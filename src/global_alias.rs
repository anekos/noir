
use std::collections::HashMap;
use std::collections::hash_map::IntoIter;
use std::convert::AsRef;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::{AppResult, AppResultU};



#[derive(Debug, Default)]
pub struct GlobalAliasTable {
    path: PathBuf,
    table: HashMap<String, Alias>,
    tags: Vec<String>,
}

pub struct Iter {
    iter: IntoIter<String, Alias>
}


impl GlobalAliasTable {
    pub fn add(&mut self, from: String, to: String, recursive: bool) {
        self.table.insert(from, Alias { expression: to, recursive });
    }

    pub fn delete(&mut self, name: &str) {
        self.table.remove(name);
    }

    // pub fn into_iter(self) -> IntoIter<String, Alias> {
    //     self.table.into_iter()
    // }
    //
    pub fn names(&self) -> Vec<&str> {
        let mut result: Vec<&str> = self.table.keys().map(AsRef::as_ref).collect();
        result.sort();
        result
    }

    pub fn open<T: AsRef<Path>>(path: &T, db: &Database) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.is_file() {
            return Ok(Self {
                path,
                table: HashMap::default(),
                tags: db.tags()?,
            });
        }

        let mut file = File::open(&path)?;
        let mut source = "".to_owned();
        let _ = file.read_to_string(&mut source)?;
        Ok(Self {
            path,
            table: serde_yaml::from_str(&source)?,
            tags: db.tags()?,
        })
    }

    pub fn save(&self) -> AppResultU {
        if let Some(dir) = self.path.parent() {
            create_dir_all(dir)?;
        }
        let source = serde_yaml::to_string(&self.table)?;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
        write!(file, "{}", source)?;
        Ok(())
    }
}


impl std::iter::Iterator for Iter {
    type Item = (String, Alias);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl std::iter::IntoIterator for GlobalAliasTable {
    type Item = (String, Alias);
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter { iter: self.table.into_iter() }
    }
}
