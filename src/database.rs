
use std::fs::create_dir_all;
use std::path::Path;

use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

use crate::errors::{AppResult, AppResultU};
use crate::meta::Meta;



pub struct Database {
    connection: Connection,
}


impl Database {
    pub fn close(self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        Ok(())
    }

    pub fn insert<T: AsRef<Path>>(&self, path: &T, meta: &Meta) -> AppResultU {
        let path = path.as_ref().to_str().unwrap();
        let (width, height) = &meta.dimensions.ratio();
        let args = &[
            &path as &ToSql,
            &meta.dimensions.width,
            &meta.dimensions.height,
            &width,
            &height,
            &meta.mime_type,
            &meta.animation,
            &(meta.file_size as u32),
            &meta.file_extension as &ToSql,
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

