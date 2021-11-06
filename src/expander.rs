
use std::collections::HashMap;

use log::info;

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::global_alias::GlobalAliasTable;
use crate::expression::{Expression, parser, string_literal};



pub struct Expander {
    aliases: HashMap<String, Alias>,
}


impl Expander {
    pub fn expand(&self, expression: &str) -> AppResult<String> {
        self.expand_n(expression, 0)
    }

    fn expand_n(&self, expression: &str, n: usize) -> AppResult<String> {
        use Expression::*;

        if 30 < n {
            return Err(AppError::Standard("Too deep recursively alias"));
        }

        info!("expanding({}): {:?}", n, expression);

        let (_rest, es) = parser::parse(expression)?;

        let mut result = "".to_owned();
        for e in es {
            match e {
                Any(c) => result.push(c),
                Delimiter(ref s) => result.push_str(s),
                NoirTag(ref tag) => {
                    result.push_str(&format!("(path in (SELECT path FROM tags WHERE tag = '{}'))", tag));
                },
                StringLiteral(ref s) => result.push_str(&string_literal(s)),
                Term(ref s) => {
                    if let Some(alias) = self.aliases.get(s) {
                        if alias.recursive {
                            let e = self.expand_n(&alias.expression, n + 1)?;
                            result.push_str(&e);
                        } else {
                            result.push_str(&alias.expression);
                        }
                    } else {
                        result.push_str(s);
                    }
                },
            }

        }
        info!("expand: {:?} → {:?}", expression, result);
        Ok(result)
    }

    pub fn generate(database: &Database, global_alias_table: &GlobalAliasTable) -> AppResult<Self> {
        let local = database.aliases()?;
        // FIXME remove clone
        let global = global_alias_table.clone().into_iter().collect();
        Ok(Self::new(local, global))
    }

    pub fn new(local: HashMap<String, Alias>, global: HashMap<String, Alias>) -> Self {
        let mut aliases = global;
        for (k, v) in local.into_iter() {
            aliases.insert(k, v);
        }
        let mut names: Vec<&String> = aliases.keys().collect();
        names.sort_by_key(|it| usize::MAX - it.len());

        Self {
            aliases,
        }
    }

    pub fn get_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    pub fn get_alias_names(&self) -> Vec<&str> {
        self.aliases.keys().map(String::as_ref).collect()
    }
}
