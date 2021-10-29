
use std::borrow::ToOwned;
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};
use std::result::Result;
use std::str::FromStr;

use walkdir::WalkDir;
use if_let_return::if_let_some;

use crate::database::Database;
use crate::errors::{AppError, AppResult, AppResultU, from_os_str, from_path, wrap_with_path};
use crate::meta::Meta;
use crate::tag::Tag;



#[derive(Debug, Default, Clone)]
pub struct Config<'a> {
    pub check_extension: bool,
    pub compute_dhash: bool,
    pub dry_run: bool,
    pub skip_errors: bool,
    pub tag_generator: Option<&'a str>,
    pub tag_source: Option<&'a str>,
    pub update: bool,
}

pub struct Loader<'a> {
    config: Config<'a>,
    count: usize,
    db: &'a Database,
}


impl<'a> Loader<'a> {
    pub fn new(db: &'a Database, config: Config<'a>) -> Self {
        Loader { config, count: 0, db }
    }

    pub fn load<T: AsRef<Path>>(&mut self, path: &T) -> AppResultU {
        log::trace!("load: {:?}", path.as_ref());

        wrap_with_path(path, {
            if path.as_ref().is_dir() {
                self.load_directory(path)
            } else if path.as_ref().is_file() {
                self.load_file(path)
            } else {
                Ok(())
            }
        })
    }

    pub fn load_file<T: AsRef<Path>>(&mut self, file: &T) -> AppResultU {
        if let Err(err) = self.load_file_inner(file) {
            if self.config.skip_errors {
                eprintln!("SKIP: {} for {:?}", err, file.as_ref());
            } else {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn load_list<T: BufRead>(&mut self, list: &mut T) -> AppResultU {
        for line in list.lines() {
            self.load(&line?)?;
        }
        Ok(())
    }

    fn load_file_inner<T: AsRef<Path>>(&mut self, file: &T) -> AppResultU {
        log::trace!("load_file: {:?}", file.as_ref());
        if self.config.check_extension && !has_image_extension(&file)? {
            log::trace!("load_file.skip.1");
            return Ok(());
        }
        let file = file.as_ref().canonicalize()?;
        if !self.config.update && self.db.path_exists(from_path(&file)?)? {
            log::trace!("load_file.skip.2");
            return Ok(());
        }
        if self.config.dry_run {
            println!("DRYRUN: {:?}", file);
            log::trace!("load_file.skip.3");
            return Ok(())
        }

        log::trace!("load_file.meta");
        let meta = Meta::from_file(&file, self.config.compute_dhash)?;

        self.count += 1;
        if self.count % 100 == 0 {
            self.db.commit()?;
            self.db.begin()?;
        }

        self.db.upsert(&meta)?;

        let tags = self.generate_tags(&file)?;
        let tags: AppResult<Vec<Tag>> = tags.iter().map(|it| Tag::from_str(it)).collect();
        let tag_source = self.config.tag_source.unwrap_or("unknown");
        self.db.set_tags(from_path(&file)?, tags?.as_slice(), tag_source)?;

        log::trace!("load_file.done");
        log::info!("Meta: {}", meta);

        Ok(())
    }

    fn load_directory<T: AsRef<Path>>(&mut self, directory: &T) -> AppResultU {
        println!("Loading: {:?}", directory.as_ref());
        log::trace!("load_directory: {:?}", directory.as_ref());
        let walker = WalkDir::new(directory).follow_links(true);
        for entry in walker.into_iter().filter_map(Result::ok).filter(|it| it.file_type().is_file()) {
            wrap_with_path(&entry.path(), self.load_file(&entry.path()))?
        }
        Ok(())
    }

    fn generate_tags<T: AsRef<Path>>(&self, file: &T) -> AppResult<Vec<String>> {
        if_let_some!(tag_generator = self.config.tag_generator, Ok(vec!()));
        let mut command = Command::new(tag_generator);
        command.args(&[file.as_ref().as_os_str()]);
        command.stdout(Stdio::piped());
        let status = command.status()?;
        if !status.success() {
            let err = command.output()?.stderr;
            let err = String::from_utf8(err)?;
            return Err(AppError::TagGeneratorFailed(err));
        }
        let result = String::from_utf8(command.output()?.stdout)?;
        Ok(result.lines().filter(|it| !it.is_empty()).map(ToOwned::to_owned).collect())
    }
}

fn has_image_extension<T: AsRef<Path>>(file: &T) -> AppResult<bool> {
    let result = if let Some(extension) = file.as_ref().extension() {
        matches!(&*from_os_str(extension)?.to_lowercase(), "png" | "jpg" | "jpeg" | "gif" | "webp")
    } else {
        false
    };
    Ok(result)
}
