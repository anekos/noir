
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};

use walkdir::WalkDir;
use if_let_return::if_let_some;

use crate::errors::{AppResult, AppResultU, from_os_str, from_path};
use crate::meta::Meta;
use crate::database::Database;



pub struct Loader<'a> {
    check_extension: bool,
    db: &'a Database,
    tag_generator: Option<&'a str>,
    count: usize,
}


impl<'a> Loader<'a> {
    pub fn new(db: &'a Database, check_extension: bool, tag_generator: Option<&'a str>) -> Self {
        Loader { db, check_extension, tag_generator, count: 0 }
    }

    pub fn load<T: AsRef<Path>>(&mut self, path: &T) -> AppResultU {
        if path.as_ref().is_dir() {
            self.load_directory(path)?;
        } else if path.as_ref().is_file() {
            self.load_file(path)?;
        }
        Ok(())
    }

    pub fn load_list<T: BufRead>(&mut self, list: &mut T) -> AppResultU {
        for line in list.lines() {
            self.load(&line?)?;
        }
        Ok(())
    }

    fn load_file<T: AsRef<Path>>(&mut self, file: &T) -> AppResultU {
        if self.check_extension && !has_image_extension(file)? {
            return Ok(());
        }
        let file = file.as_ref().canonicalize()?;
        if let Ok(meta) = Meta::from_file(&file) {
            self.count += 1;
            if self.count % 100 == 0 {
                self.db.flush()?;
            }
            let tags = self.generate_tags(&file)?;
            let tags: Vec<&str> = tags.iter().map(|it| it.as_ref()).collect();
            self.db.set_tags(from_path(&file)?, &tags)?;
            self.db.insert(&meta)?;
            println!("{}", meta);
        }
        Ok(())
    }

    fn load_directory<T: AsRef<Path>>(&mut self, directory: &T) -> AppResultU {
        println!("Loading: {:?}", directory.as_ref());
        let walker = WalkDir::new(directory).follow_links(true);
        for entry in walker.into_iter().filter_map(|it| it.ok()).filter(|it| it.file_type().is_file()) {
            self.load_file(&entry.path())?;
        }
        Ok(())
    }

    fn generate_tags<T: AsRef<Path>>(&self, file: &T) -> AppResult<Vec<String>> {
        if_let_some!(tag_generator = self.tag_generator, Ok(vec!()));
        let mut command = Command::new(tag_generator);
        command.args(&[file.as_ref().as_os_str()]);
        command.stdout(Stdio::piped());
        let result = String::from_utf8(command.output()?.stdout)?;
        Ok(result.lines().filter(|it| !it.is_empty()).map(|it| it.to_owned()).collect())
    }
}

fn has_image_extension<T: AsRef<Path>>(file: &T) -> AppResult<bool> {
    let result = if let Some(extension) = file.as_ref().extension() {
        match &*from_os_str(&extension)?.to_lowercase() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" => true,
            _ => false,
        }
    } else {
        false
    };
    Ok(result)
}
