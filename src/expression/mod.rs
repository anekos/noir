

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Any(char),
    Delimiter(String),
    NoirTag(String),
    PathSegment(String),
    StringLiteral(String),
    Term(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoirQuery {
    pub elements: Vec<Expression>
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawQuery(String);


pub mod modifier;
pub mod parser;


impl ToString for NoirQuery {
    fn to_string(&self) -> String {
        use Expression::*;

        let mut result = "".to_owned();
        for e in &self.elements {
            match e {
                Any(c) => result.push(*c),
                Delimiter(ref s) => result.push_str(s),
                NoirTag(ref tag) => result.push_str(&format!("#{}", tag)),
                PathSegment(ref s) => result.push_str(&format!("`{}`", s)),
                StringLiteral(ref s) => result.push_str(&string_literal(s)),
                Term(ref s) => result.push_str(s),
            }

        }
        result
    }
}

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
