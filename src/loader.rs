
use std::path::Path;

use walkdir::WalkDir;

use crate::errors::AppResultU;
use crate::meta::Meta;
use crate::database::Database;



pub fn load<T: AsRef<Path>, U: AsRef<Path>>(directory: &T, db: &U) -> AppResultU {
    println!("Loading: {:?}", directory.as_ref());

    let db = Database::open(db)?;

    for entry in WalkDir::new(directory) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        if let Ok(meta) = Meta::from_file(&entry.path()) {
            db.insert(&entry.path(), &meta)?;
            println!("{} → {:?}", entry.path().display(), meta);
        }
    }

    db.close()?;

    Ok(())
}


