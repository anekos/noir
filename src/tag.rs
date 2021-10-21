
use std::convert::From;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::types::{ToSql, ToSqlOutput, Value, ValueRef};
use rusqlite::{Result as QResult};

use crate::errors::{AppError, AppResult};



lazy_static! {
    static ref TAG_NAME: Regex = Regex::new(r"\S+").unwrap();
}

pub struct Tag(String);



impl From<Tag> for Value {
    fn from(tag: Tag) -> Self {
        Value::Text(tag.0)
    }
}

impl ToSql for Tag {
    fn to_sql(&self) -> QResult<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(self.0.as_bytes())))
    }
}

impl FromStr for Tag {
    type Err = AppError;
    fn from_str(tag: &str) -> AppResult<Self> {
        let tag = tag.to_owned();
        if (*TAG_NAME).is_match(&tag) {
            Ok(Tag(tag))
        } else {
            Err(AppError::InvalidTagFormat(tag))
        }
    }
}
