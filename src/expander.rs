
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
        let result = self.unalias(expression);
        self.untag(&result).to_string()
    }

    pub fn generate(database: &Database, global_alias_table: GlobalAliasTable) -> AppResult<Self> {
        let local = database.aliases()?;
        let global = global_alias_table.into_iter().collect();
        let tags = database.tags()?;
        Ok(Self::new(local, global, tags))
    }

    pub fn new(local: HashMap<String, Alias>, global: HashMap<String, Alias>, tags: Vec<String>) -> Self {
        let mut aliases = global;
        for (k, v) in local.into_iter() {
            aliases.insert(k, v);
        }
        let names: Vec<&String> = aliases.keys().collect();
        let alias_pattern = word_pattern(&names, "");

        let tags_pattern = word_pattern(&tags, "#");

        Self {
            alias_pattern,
            aliases,
            tags,
            tags_pattern,
        }
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
            &expression,
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



#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use crate::alias::Alias;
    use crate::expander::Expander;

    #[test]
    fn test_expandable() {
        let e = Expander::new(
            hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
            hashmap!{},
            vec![]);

        assert_eq!(e.expand("begin hoge end"), "begin fuga end".to_owned());
        assert_eq!(e.expand("hoge end"), "fuga end".to_owned());
        assert_eq!(e.expand("begin hoge"), "begin fuga".to_owned());
        assert_eq!(e.expand("hoge"), "fuga".to_owned());
        assert_eq!(e.expand("<hoge>"), "<fuga>".to_owned());
    }

    #[test]
    fn test_tag_expandable() {
        let e = Expander::new(
            hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
            hashmap!{},
            vec!["moge".to_owned()]);

        assert_eq!(
            e.expand("begin #moge end"),
            "begin (path in (SELECT path FROM tags WHERE tag = 'moge')) end".to_owned());
        assert_eq!(
            e.expand("begin #moge"),
            "begin (path in (SELECT path FROM tags WHERE tag = 'moge'))".to_owned());
    }

    #[test]
    fn test_non_expandable() {
        let e = Expander::new(
            hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
            hashmap!{},
            vec![]);

        assert_eq!(e.expand("beginhogeend"), "beginhogeend".to_owned());
        assert_eq!(e.expand("a"), "a".to_owned());
        assert_eq!(e.expand("1"), "1".to_owned());
    }

    #[test]
    fn test_tag_non_expandable() {
        let e = Expander::new(
            hashmap!{},
            hashmap!{},
            vec![]);

        assert_eq!(e.expand("begin #hoge end"), "begin #hoge end".to_owned());
    }

    #[test]
    fn test_recursive() {
        let e = Expander::new(
            hashmap!{
                "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: true },
                "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
            },
            hashmap!{},
            vec![]);

        assert_eq!(e.expand("begin hoge end"), "begin meow end".to_owned());
    }

    #[test]
    fn test_nonrecursive() {
        let e = Expander::new(
            hashmap!{
                "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false },
                "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
            },
            hashmap!{},
            vec![]);

        assert_eq!(e.expand("begin hoge end"), "begin fuga end".to_owned());
    }
}
