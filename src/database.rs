
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
    pub fn add_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        for tag in tags {
            let args = &[tag, path];
            self.connection.execute(include_str!("insert_tag.sql"), args)?;
        }
        Ok(())
    }

    pub fn clear_tags(&self, path: &str) -> AppResultU {
        self.connection.execute(include_str!("clear_tags.sql"), &[path])?;
        Ok(())
    }

    pub fn close(self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        Ok(())
    }

    pub fn flush(&self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        self.connection.execute("BEGIN;", NO_PARAMS)?;
        Ok(())
    }

    pub fn insert(&self, meta: &Meta) -> AppResultU {
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
            &meta.file.created.as_ref(),
            &meta.file.modified.as_ref(),
            &meta.file.accessed.as_ref(),
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

    pub fn select<F>(&self, where_expression: &str, mut f: F) -> AppResultU where F: FnMut(&str) -> AppResultU {
        use crate::meta::*;

        let mut stmt = self.connection.prepare(&format!("SELECT * FROM images WHERE {}", where_expression))?;
        let iter = stmt.query_map(NO_PARAMS, |row| Meta {
            animation: row.get(6),
            dimensions: Dimensions {
                width: row.get(1),
                height: row.get(2),
            },
            mime_type: "hoge",
            file: FileMeta {
                path: row.get(0),
                size: row.get(7),
                created: row.get(8),
                modified: row.get(9),
                accessed: row.get(10),
            },
        })?;

        for it in iter {
            f(&it?.file.path)?;
        }

        Ok(())
    }

    pub fn remove_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        for tag in tags {
            let args = &[tag, path];
            self.connection.execute(include_str!("remove_tag.sql"), args)?;
        }
        Ok(())
    }

    pub fn set_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        self.clear_tags(path)?;
        self.add_tags(path, tags)
    }
}

fn create_table(conn: &Connection) -> AppResultU {
    let sql: &'static str = include_str!("create_images_table.sql");
    conn.execute(sql, NO_PARAMS)?;
    let sql: &'static str = include_str!("create_tags_table.sql");
    conn.execute(sql, NO_PARAMS)?;
    let sql: &'static str = include_str!("create_tags_index.sql");
    conn.execute(sql, NO_PARAMS)?;
    Ok(())
}
