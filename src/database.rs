
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;

use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS, Row};

use crate::alias::Alias;
use crate::errors::{AppResult, AppResultU, from_path};
use crate::meta::Meta;



pub const SELECT_PREFIX: &str = "SELECT * FROM images WHERE ";

pub struct Database {
    connection: Connection,
}

macro_rules! sql {
    ($name:tt) => {
        include_str!(concat!("sql/", stringify!($name), ".sql"))
    }
}


impl Database {
    pub fn add_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        for tag in tags {
            let args = &[tag, path];
            self.connection.execute(sql!(insert_tag), args)?;
        }
        Ok(())
    }

    pub fn aliases(&self) -> AppResult<HashMap<String, Alias>> {
        let mut stmt = self.connection.prepare("SELECT * FROM aliases")?;
        let result: rusqlite::Result<HashMap<String, Alias>> = stmt.query_map(
            NO_PARAMS,
            |row: &Row|
            (
                row.get(0),
                Alias {
                    expression: row.get(1),
                    recursive: row.get(2)
                }
            ))?.collect();

        Ok(result?)
    }

    pub fn clear_tags(&self, path: &str) -> AppResultU {
        self.connection.execute(sql!(clear_tags), &[path])?;
        Ok(())
    }

    pub fn close(self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        Ok(())
    }

    fn delete_path(&self, path: &str) -> AppResultU {
        self.connection.execute("DELETE FROM images WHERE path = ?1", &[path])?;
        self.connection.execute("DELETE FROM tags WHERE path = ?1", &[path])?;
        Ok(())
    }

    pub fn flush(&self) -> AppResultU {
        self.connection.execute("COMMIT;", NO_PARAMS)?;
        self.connection.execute("BEGIN;", NO_PARAMS)?;
        Ok(())
    }

    pub fn get(&self, path: &str) -> AppResult<Option<Meta>> {
        let path = Path::new(path).canonicalize().unwrap_or_else(|_| Path::new(path).to_path_buf());
        let path = from_path(&path)?;
        let mut stmt = self.connection.prepare("SELECT * FROM images WHERE path = ?1")?;
        let mut iter = stmt.query_map(&[&path as &ToSql], from_row)?;
        if let Some(found) = iter.next() {
            Ok(Some(found?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, meta: &Meta) -> AppResultU {
        let (width, height) = &meta.dimensions.ratio();
        let args = &[
            &meta.file.path as &ToSql,
            &meta.dimensions.width,
            &meta.dimensions.height,
            &width,
            &height,
            &meta.r#type,
            &meta.animation,
            &(meta.file.size as u32),
            &meta.file.created.as_ref(),
            &meta.file.modified.as_ref(),
            &meta.file.accessed.as_ref(),
        ];
        self.connection.execute(sql!(update_image), args)?;
        self.connection.execute(sql!(insert_image), args)?;
        Ok(())
    }

    pub fn upsert_alias(&self, name: &str, original: &str, recursive: bool) -> AppResultU {
        let args = &[&name as &ToSql, &original as &ToSql, &recursive as &ToSql];
        self.connection.execute(sql!(update_alias), args)?;
        self.connection.execute(sql!(insert_alias), args)?;
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

    pub fn path_exists(&self, path: &str) -> AppResult<bool> {
        let mut stmt = self.connection.prepare("SELECT 1 FROM images WHERE path = ?;")?;
        Ok(stmt.exists(&[&path as &ToSql])?)
    }

    pub fn reset(&self) -> AppResultU {
        self.connection.execute("DELETE FROM images", NO_PARAMS)?;
        self.connection.execute("DELETE FROM tags", NO_PARAMS)?;
        self.flush()?;
        Ok(())
    }

    pub fn select<F>(&self, where_expression: &str, vacuum: bool, mut f: F) -> AppResultU where F: FnMut(&str, bool) -> AppResultU {
        let mut stmt = self.connection.prepare(&format!("{}{}", SELECT_PREFIX, where_expression))?;
        let iter = stmt.query_map(NO_PARAMS, from_row)?;

        for it in iter {
            let it = it?;
            let vacuumed = vacuum && !Path::new(&it.file.path).is_file();
            if vacuumed {
                self.delete_path(&it.file.path)?;
            }
            f(&it.file.path, vacuumed)?;
        }

        Ok(())
    }

    pub fn tags(&self) -> AppResult<Vec<String>> {
        let mut stmt = self.connection.prepare("SELECT DISTINCT tag FROM tags ORDER BY length(tag)")?;
        let result: rusqlite::Result<Vec<String>> = stmt.query_map(NO_PARAMS, |row: &Row| row.get(0))?.collect();
        Ok(result?)
    }

    pub fn delete_alias(&self, name: &str) -> AppResultU {
        self.connection.execute("DELETE FROM aliases WHERE name = ?1", &[name])?;
        Ok(())
    }

    pub fn delete_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        for tag in tags {
            let args = &[tag, path];
            self.connection.execute(sql!(delete_tag), args)?;
        }
        Ok(())
    }

    pub fn set_tags(&self, path: &str, tags: &[&str]) -> AppResultU {
        self.clear_tags(path)?;
        self.add_tags(path, tags)
    }
}

fn create_table(conn: &Connection) -> AppResultU {
    fn create(conn: &Connection, sql: &str) -> AppResultU {
        conn.execute(sql, NO_PARAMS)?;
        Ok(())
    }
    create(conn, sql!(create_images_table))?;
    create(conn, sql!(create_tags_table))?;
    create(conn, sql!(create_tags_index))?;
    create(conn, sql!(create_aliases_table))?;
    Ok(())
}

fn from_row(row: &Row) -> Meta {
    use crate::meta::*;

    Meta {
        animation: row.get(6),
        dimensions: Dimensions {
            width: row.get(1),
            height: row.get(2),
        },
        r#type: {
            let t: String = row.get(5);
            ["png", "gif", "jpeg", "webp"].iter().find(|it| **it == &*t).expect("Unknown mime type")
        },
        file: FileMeta {
            path: row.get(0),
            size: row.get(7),
            created: row.get(8),
            modified: row.get(9),
            accessed: row.get(10),
        },
    }
}
