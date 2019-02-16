
use std::fs::create_dir_all;
use std::path::Path;

use chrono::DateTime;
use chrono::offset::Utc;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

use crate::errors::{AppResult, AppResultU};
use crate::meta::Meta;



const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";


pub struct Database {
    connection: Connection,
}


impl Database {
    pub fn close(self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        Ok(())
    }

    pub fn insert(&self, meta: &Meta) -> AppResultU {
        fn from_datetime(t: &DateTime<Utc>) -> String {
            t.format(DATETIME_FORMAT).to_string()
        }

        let (width, height) = &meta.dimensions.ratio();
        let args = &[
            &meta.file.path as &ToSql,
            &meta.dimensions.width,
            &meta.dimensions.height,
            &width,
            &height,
            &meta.mime_type,
            &meta.animation,
            &(meta.file.size as u32),
            &meta.file.created.as_ref().map(from_datetime),
            &meta.file.modified.as_ref().map(from_datetime),
            &meta.file.accessed.as_ref().map(from_datetime),
        ];
        self.connection.execute(include_str!("update.sql"), args)?;
        self.connection.execute(include_str!("insert.sql"), args)?;
        Ok(())
    }

    pub fn open<T: AsRef<Path>>(file: &T) -> AppResult<Self> {
        if let Some(dir) = file.as_ref().parent() {
            create_dir_all(dir)?;
        }
        let connection = Connection::open(file.as_ref())?;
        create_table(&connection)?;
        connection.execute("BEGIN;", NO_PARAMS)?;
        Ok(Database { connection })
    }
}

fn create_table(conn: &Connection) -> AppResultU {
    let sql: &'static str = include_str!("create_table.sql");
    conn.execute(sql, NO_PARAMS)?;
    Ok(())
}

