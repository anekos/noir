
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;

use regex::{Captures, Regex};
use serde_derive::{Deserialize, Serialize};

use crate::database::Database;
use crate::errors::{AppResult, AppResultU};



#[derive(Debug, Default)]
pub struct AliasTable {
    table: HashMap<String, Alias>,
    tags: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Alias {
    expression: String,
    recursive: bool,
}

impl AliasTable {
    pub fn alias(&mut self, from: String, to: String, recursive: bool) {
        self.table.insert(from, Alias { expression: to, recursive });
    }

    pub fn expand(&self, expression: &str) -> String {
        let result = self.unkeyword(expression);
        self.untag(&result).to_string()
    }

    pub fn names(&self) -> Vec<&str> {
        let mut result: Vec<&str> = self.table.keys().map(|it| it.as_ref()).collect();
        result.sort();
        result
    }

    pub fn open<T: AsRef<Path>>(path: &T, db: &Database) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.is_file() {
            return Ok(AliasTable {
                table: HashMap::default(),
                tags: db.tags()?,
            });
        }

        let mut file = File::open(&path)?;
        let mut source = "".to_owned();
        let _ = file.read_to_string(&mut source)?;
        Ok(AliasTable {
            table: serde_yaml::from_str(&source)?,
            tags: db.tags()?,
        })
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

    fn tags_pattern(&self) -> Regex {
        let tags: Vec<&str> = self.tags.iter().map(|it| it.as_ref()).collect();
        word_pattern(&tags, "#")
    }

    pub fn unalias(&mut self, name: &str) {
        self.table.remove(name);
    }

    fn unkeyword<'a>(&self, expression: &'a str) -> Cow<'a, str> {
        if self.table.is_empty() {
            return expression.into()
        }
        let pattern = self.keywords_pattern();
        pattern.replace_all(
            expression,
            |captures: &Captures| {
                let name = captures.get(0).unwrap().as_str();
                let alias = &self.table[name];
                if alias.recursive {
                    self.expand(&alias.expression)
                } else {
                    alias.expression.clone()
                }
            })
    }

    fn untag<'a>(&self, expression: &'a str) -> Cow<'a, str> {
        if self.tags.is_empty() {
            return expression.into();
        }
        let pattern = self.tags_pattern();
        pattern.replace_all(
            &expression,
            |captures: &Captures| {
                let tag = captures.get(1).unwrap().as_str();
                format!("(path in (SELECT path FROM tags WHERE tag = '{}'))", tag)
            })
    }

    fn keywords_pattern(&self) -> Regex {
        let keys: Vec<&str> = self.table.keys().map(|it| it.as_str()).collect();
        word_pattern(&keys, "\\b")
    }
}

fn word_pattern(words: &[&str], prefix: &str) -> Regex {
    let mut result = "".to_owned();
    for word in words {
        let word = regex::escape(&word);
        if !result.is_empty() {
            result.push('|');
        }
        result.push_str(&word);
    }
    Regex::new(&format!("{}({})\\b", prefix, result)).unwrap()
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_expandable() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned(), false);

        assert_eq!(aliases.expand("begin hoge end"), "begin fuga end".to_owned());
        assert_eq!(aliases.expand("hoge end"), "fuga end".to_owned());
        assert_eq!(aliases.expand("begin hoge"), "begin fuga".to_owned());
        assert_eq!(aliases.expand("hoge"), "fuga".to_owned());
        assert_eq!(aliases.expand("<hoge>"), "<fuga>".to_owned());
    }

    #[test]
    fn test_tag_expandable() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned(), false);
        aliases.tags.push("moge".to_owned());

        assert_eq!(
            aliases.expand("begin #moge end"),
            "begin (path in (SELECT path FROM tags WHERE tag = 'moge')) end".to_owned());
        assert_eq!(
            aliases.expand("begin #moge"),
            "begin (path in (SELECT path FROM tags WHERE tag = 'moge'))".to_owned());
    }

    #[test]
    fn test_non_expandable() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned(), false);

        assert_eq!(aliases.expand("beginhogeend"), "beginhogeend".to_owned());
        assert_eq!(aliases.expand("a"), "a".to_owned());
        assert_eq!(aliases.expand("1"), "1".to_owned());
    }

    #[test]
    fn test_tag_non_expandable() {
        let aliases = crate::alias::AliasTable::default();

        assert_eq!(
            aliases.expand("begin #hoge end"),
            "begin #hoge end".to_owned());
    }

    #[test]
    fn test_recursive() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned(), true);
        aliases.alias("fuga".to_owned(), "meow".to_owned(), false);

        assert_eq!(aliases.expand("begin hoge end"), "begin meow end".to_owned());
    }

    #[test]
    fn test_nonrecursive() {
        let mut aliases = crate::alias::AliasTable::default();
        aliases.alias("hoge".to_owned(), "fuga".to_owned(), false);
        aliases.alias("fuga".to_owned(), "meow".to_owned(), false);

        assert_eq!(aliases.expand("begin hoge end"), "begin fuga end".to_owned());
    }
}
