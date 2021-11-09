

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
pub struct RawQuery(String);


pub mod modifier;
pub mod parser;


impl RawQuery {
    pub fn new(q: String) -> Self {
        RawQuery(q)
    }
}

impl ToString for RawQuery {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl AsRef<str> for RawQuery {
    fn as_ref(&self) -> &str {
        &self.0
    }
}



pub fn string_literal(s: &str) -> String {
    let mut result = "'".to_owned();
    result.push_str(&s.replace('\'', "''"));
    result.push('\'');
    return result;
}
