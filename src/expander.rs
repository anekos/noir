
use std::borrow::Cow;
use std::collections::HashMap;

use regex::{Captures, Regex};

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::AppResult;
use crate::global_alias::GlobalAliasTable;



pub struct Expander {
    alias_pattern: Regex,
    aliases: HashMap<String, Alias>,
    tags: Vec<String>,
    tags_pattern: Regex,
}


impl Expander {
    pub fn expand(&self, expression: &str) -> String {
        println!("expand: {:?}", expression);
        let result = self.unalias(expression);
        self.untag(&result).to_string()
    }

    pub fn generate(database: &Database, global_alias_table: &GlobalAliasTable) -> AppResult<Self> {
        let local = database.aliases()?;
        // FIXME remove clone
        let global = global_alias_table.clone().into_iter().collect();
        let tags = database.tags()?;
        Ok(Self::new(local, global, tags))
    }

    pub fn new(local: HashMap<String, Alias>, global: HashMap<String, Alias>, tags: Vec<String>) -> Self {
        let mut aliases = global;
        for (k, v) in local.into_iter() {
            aliases.insert(k, v);
        }
        let mut names: Vec<&String> = aliases.keys().collect();
        names.sort_by_key(|it| usize::MAX - it.len());
        let alias_pattern = word_pattern(&names, "");

        let tags_pattern = word_pattern(&tags, "#");

        Self {
            alias_pattern,
            aliases,
            tags,
            tags_pattern,
        }
    }

    pub fn get_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    pub fn get_alias_names(&self) -> Vec<&str> {
        self.aliases.keys().map(String::as_ref).collect()
    }

    pub fn get_tag_names(&self) -> Vec<&str> {
        self.tags.iter().map(String::as_ref).collect()
    }

    fn unalias<'b>(&self, expression: &'b str) -> Cow<'b, str> {
        if self.aliases.is_empty() {
            return expression.into()
        }
        self.alias_pattern.replace_all(
            expression,
            |captures: &Captures| {
                let name = captures.get(0).unwrap().as_str();
                let alias = &self.aliases[name];
                if alias.recursive {
                    self.expand(&alias.expression)
                } else {
                    alias.expression.clone()
                }
            })
    }

    fn untag<'b>(&self, expression: &'b str) -> Cow<'b, str> {
        if self.tags.is_empty() {
            return expression.into();
        }
        self.tags_pattern.replace_all(
            expression,
            |captures: &Captures| {
                let tag = captures.get(1).unwrap().as_str();
                format!("(path in (SELECT path FROM tags WHERE tag = '{}'))", tag)
            })
    }
}


fn word_pattern<T: AsRef<str>>(words: &[T], prefix: &str) -> Regex {
    let mut result = "".to_owned();
    for word in words {
        let word = regex::escape(word.as_ref());
        if !result.is_empty() {
            result.push('|');
        }
        result.push_str(&word);
    }
    Regex::new(&format!("{}({})\\b", prefix, result)).unwrap()
}
