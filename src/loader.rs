
use std::path::Path;

use walkdir::WalkDir;

use crate::errors::{AppResult, AppResultU, from_os_str};
use crate::meta::Meta;
use crate::database::Database;



pub fn load<T: AsRef<Path>, U: AsRef<Path>>(directory: &T, db: &U) -> AppResultU {
    println!("Loading: {:?}", directory.as_ref());

    let db = Database::open(db)?;

    for entry in WalkDir::new(directory) {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_dir() || !has_image_extension(&path)? {
            continue;
        }
        if let Ok(meta) = Meta::from_file(&path) {
            db.insert(&meta)?;
            println!("{} → {:?}", path.display(), meta);
        }
    }

    db.close()?;

    Ok(())
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
