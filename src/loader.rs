
use std::io::BufRead;
use std::path::Path;

use walkdir::WalkDir;

use crate::errors::{AppResult, AppResultU, from_os_str};
use crate::meta::Meta;
use crate::database::Database;



pub struct Loader<'a> {
    db: &'a Database,
}


impl<'a> Loader<'a> {
    pub fn new(db: &'a Database) -> Self {
        Loader { db }
    }

    pub fn load<T: AsRef<Path>>(&self, path: &T) -> AppResultU {
        if path.as_ref().is_dir() {
            self.load_directory(path)?
        } else if path.as_ref().is_file() {
            self.load_file(path)?
        }
        Ok(())
    }

    pub fn load_list<T: BufRead>(&self, list: &mut T) -> AppResultU {
        for line in list.lines() {
            self.load(&line?)?;
        }
        Ok(())
    }

    fn load_file<T: AsRef<Path>>(&self, file: &T) -> AppResultU {
        if !has_image_extension(file)? {
            return Ok(());
        }
        if let Ok(meta) = Meta::from_file(&file) {
            self.db.insert(&meta)?;
            println!("{}", meta);
        }
        Ok(())
    }

    fn load_directory<T: AsRef<Path>>(&self, directory: &T) -> AppResultU {
        println!("Loading: {:?}", directory.as_ref());
        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            self.load_file(&entry.path())?;
        }
        Ok(())
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
