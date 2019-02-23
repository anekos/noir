
use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;

use regex::{Captures, Regex};

use crate::errors::{AppResult, AppResultU};



#[derive(Debug, Default)]
pub struct AliasTable {
    table: HashMap<String, String>,
}


impl AliasTable {
    pub fn alias(&mut self, from: String, to: String) {
        self.table.insert(from, to);
    }

    pub fn expand(&self, expression: &str) -> String {
        if self.table.is_empty() {
            return expression.to_owned();
        }
        let pattern = self.keywords_pattern();
        pattern.replace_all(
            expression,
            |captures: &Captures| {
                let name = captures.get(0).unwrap().as_str();
                &self.table[name]
            }).to_string()
    }

    pub fn open<T: AsRef<Path>>(path: &T) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.is_file() {
            return Ok(AliasTable { table: HashMap::default() });
        }

        let mut file = File::open(&path)?;
        let mut source = "".to_owned();
        let _ = file.read_to_string(&mut source)?;

        Ok(AliasTable { table: serde_yaml::from_str(&source)? })
    }

    pub fn save<T: AsRef<Path>>(&self, path: &T) -> AppResultU {
        if let Some(dir) = path.as_ref().parent() {
            create_dir_all(dir)?;
        }
        let source = serde_yaml::to_string(&self.table)?;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&path)?;
        write!(file, "{}", source)?;
        Ok(())
    }

    pub fn unalias(&mut self, name: &str) {
        self.table.remove(name);
    }

    fn keywords_pattern(&self) -> Regex {
        let mut result = "".to_owned();
        for key in self.table.keys() {
            let key = regex::escape(&key);
            if !result.is_empty() {
                result.push('|');
            }
            result.push_str(&key);
        }
        Regex::new(&format!("\\b(?:{})\\b", result)).unwrap()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_expandable() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned());

        assert_eq!(aliases.expand("begin hoge end"), "begin fuga end".to_owned());
        assert_eq!(aliases.expand("hoge end"), "fuga end".to_owned());
        assert_eq!(aliases.expand("begin hoge"), "begin fuga".to_owned());
        assert_eq!(aliases.expand("hoge"), "fuga".to_owned());
        assert_eq!(aliases.expand("<hoge>"), "<fuga>".to_owned());
    }

    #[test]
    fn test_non_expandable() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned());

        assert_eq!(aliases.expand("beginhogeend"), "beginhogeend".to_owned());
        assert_eq!(aliases.expand("a"), "a".to_owned());
        assert_eq!(aliases.expand("1"), "1".to_owned());
    }
}
