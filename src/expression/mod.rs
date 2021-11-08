

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Any(char),
    Delimiter(String),
    NoirTag(String),
    PathSegment(String),
    StringLiteral(String),
    Term(String),
}

#[derive(Clone, Debug)]
pub struct NoirQuery {
    pub elements: Vec<Expression>
}

#[derive(Clone, Debug)]
pub struct RawQuery(pub String);


pub mod parser;


pub fn string_literal(s: &str) -> String {
    let mut result = "'".to_owned();
    result.push_str(&s.replace('\'', "''"));
    result.push('\'');
    return result;
}
