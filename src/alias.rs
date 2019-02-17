
use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::errors::{AppResult, AppResultU};



#[derive(Debug)]
pub struct AliasTable {
    path: PathBuf,
    table: HashMap<String, String>,
}


impl AliasTable {
    pub fn alias(&mut self, from: String, to: String) {
        self.table.insert(from, to);
    }

    pub fn close(self) -> AppResultU {
        if let Some(dir) = self.path.parent() {
            create_dir_all(dir)?;
        }
        let source = serde_yaml::to_string(&self.table)?;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
        write!(file, "{}", source)?;
        Ok(())
    }

    pub fn expand(&self, expression: &str) -> String {
        self.table.get(expression).cloned().unwrap_or_else(|| expression.to_owned())
    }

    pub fn open<T: AsRef<Path>>(path: &T) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.is_file() {
            return Ok(AliasTable {
                path,
                table: HashMap::default(),
            });
        }

        let mut file = File::open(&path)?;
        let mut source = "".to_owned();
        let _ = file.read_to_string(&mut source)?;

        Ok(AliasTable {
            path,
            table: serde_yaml::from_str(&source)?,
        })
    }

    pub fn unalias(&mut self, name: &str) {
        self.table.remove(name);
    }
}
