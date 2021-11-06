#[derive(Debug, PartialEq)]
pub enum Expression {
    Any(char),
    Delimiter(String),
    NoirTag(String),
    StringLiteral(String),
    Term(String),
}

pub mod parser;


pub fn string_literal(s: &str) -> String {
    let mut result = "'".to_owned();
    result.push_str(&s.replace('\'', "''"));
    result.push('\'');
    return result;
}
