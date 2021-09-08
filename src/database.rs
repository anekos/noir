
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;

use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS, Row};

use crate::alias::Alias;
use crate::errors::{AppResult, AppResultU, from_path};
use crate::meta::Meta;
use crate::tag::Tag;



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
    pub fn add_tags(&self, path: &str, tags: &[Tag]) -> AppResultU {
        for tag in tags {
            let args = &[&tag as &dyn ToSql, &path as &dyn ToSql];
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

    pub fn delete_alias(&self, name: &str) -> AppResultU {
        self.connection.execute("DELETE FROM aliases WHERE name = ?1", &[name])?;
        Ok(())
    }

    pub fn delete_tags(&self, path: &str, tags: &[Tag]) -> AppResultU {
        for tag in tags {
            let args = &[&tag as &dyn ToSql, &path as &dyn ToSql];
            self.connection.execute(sql!(delete_tag), args)?;
        }
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
        let mut iter = stmt.query_and_then(&[&path as &dyn ToSql], from_row)?;
        iter.next().transpose()
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
        Ok(stmt.exists(&[&path as &dyn ToSql])?)
    }

    pub fn reset(&self) -> AppResultU {
        self.connection.execute("DROP TABLE images", NO_PARAMS)?;
        self.connection.execute("DROP TABLE tags", NO_PARAMS)?;
        self.flush()?;
        create_table(&self.connection)?;
        Ok(())
    }

    pub fn select<F>(&self, where_expression: &str, vacuum: bool, mut f: F) -> AppResultU where F: FnMut(&Meta, bool) -> AppResultU {
        let mut stmt = self.connection.prepare(&format!("{}{}", SELECT_PREFIX, where_expression))?;
        let iter = stmt.query_and_then(NO_PARAMS, from_row)?;

        for it in iter {
            let it = it?;
            let vacuumed = vacuum && !Path::new(&it.file.path).is_file();
            if vacuumed {
                self.delete_path(&it.file.path)?;
            }
            f(&it, vacuumed)?;
        }

        Ok(())
    }

    pub fn set_tags(&self, path: &str, tags: &[Tag]) -> AppResultU {
        self.clear_tags(path)?;
        self.add_tags(path, tags)
    }
    pub fn tags(&self) -> AppResult<Vec<String>> {
        let mut stmt = self.connection.prepare("SELECT DISTINCT tag FROM tags ORDER BY length(tag)")?;
        let result: rusqlite::Result<Vec<String>> = stmt.query_map(NO_PARAMS, |row: &Row| row.get(0))?.collect();
        Ok(result?)
    }

    pub fn tags_by_path(&self, path: &str) -> AppResult<Vec<String>> {
        let mut stmt = self.connection.prepare("SELECT tag FROM tags WHERE path = ?1")?;
        let result: rusqlite::Result<Vec<String>> = stmt.query_map(&[path], |row: &Row| row.get(0))?.collect();
        Ok(result?)
    }

    pub fn upsert(&self, meta: &Meta) -> AppResultU {
        let (width, height) = &meta.dimensions.ratio();
        let args = &[
            &meta.file.path as &dyn ToSql,
            &meta.dimensions.width,
            &meta.dimensions.height,
            &width,
            &height,
            &meta.format,
            &meta.animation,
            &(meta.file.size as u32),
            &(meta.dhash as i64) as &dyn ToSql,
            &meta.file.created.as_ref(),
            &meta.file.modified.as_ref(),
            &meta.file.accessed.as_ref(),
        ];
        self.connection.execute(sql!(update_image), args)?;
        self.connection.execute(sql!(insert_image), args)?;
        Ok(())
    }

    pub fn upsert_alias(&self, name: &str, original: &str, recursive: bool) -> AppResultU {
        let args = &[&name as &dyn ToSql, &original as &dyn ToSql, &recursive as &dyn ToSql];
        self.connection.execute(sql!(update_alias), args)?;
        self.connection.execute(sql!(insert_alias), args)?;
        Ok(())
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

fn from_row(row: &Row) -> AppResult<Meta> {
    use crate::image_format::{from_raw, ImageFormatExt};
    use crate::meta::*;

    let result = Meta {
        animation: row.get_checked(6)?,
        dhash: {
            let dhash: i64 = row.get_checked(8)?;
            dhash as u64
        },
        dimensions: Dimensions {
            width: row.get_checked(1)?,
            height: row.get_checked(2)?,
        },
        format: from_raw(row.get_raw(5))?.to_str(),
        file: FileMeta {
            path: row.get_checked(0)?,
            size: row.get_checked(7)?,
            created: row.get_checked(9)?,
            modified: row.get_checked(10)?,
            accessed: row.get_checked(11)?,
        },
    };
    Ok(result)
}
