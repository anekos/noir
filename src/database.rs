
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;

use chrono::DateTime;
use chrono::offset::Utc;
use log::info;
use rusqlite::types::ToSql;
use rusqlite::{Connection, Row};

use crate::alias::Alias;
use crate::errors::{AppError, AppResult, AppResultU, from_path};
use crate::meta::Meta;
use crate::search_history::SearchHistory;
use crate::tag::Tag;
use crate::defun::{add_distance_function, add_match_functions, add_recent_function};



pub const SELECT_PREFIX: &str = "SELECT * FROM images WHERE ";

pub struct Database {
    connection: Connection,
}

pub struct Tx<'a> {
    database: &'a Database,
}


macro_rules! sql {
    ($name:tt) => {
        include_str!(concat!("sql/", stringify!($name), ".sql"))
    }
}


impl Database {
    pub fn add_search_history(&self, where_expression: &str) -> AppResultU {
        let now: DateTime<Utc> = Utc::now();
        let exp = where_expression.trim();
        let args = &[&exp as &dyn ToSql, &now as &dyn ToSql];
        self.connection.execute(sql!(update_search_history), args)?;
        self.connection.execute(sql!(insert_search_history), args)?;
        Ok(())
    }

    pub fn add_tags(&self, path: &str, tags: &[Tag], source: &str) -> AppResultU {
        self.check_path_existence(path)?;
        for tag in tags {
            let args = &[&tag as &dyn ToSql, &path as &dyn ToSql, &source as &dyn ToSql];
            self.connection.execute(sql!(insert_tag), args)?;
        }
        Ok(())
    }

    pub fn aliases(&self) -> AppResult<HashMap<String, Alias>> {
        let mut stmt = self.connection.prepare("SELECT * FROM aliases")?;
        let result: rusqlite::Result<HashMap<String, Alias>> = stmt.query_map(
            [],
            |row: &Row|
            Ok((
                row.get(0)?,
                Alias {
                    expression: row.get(1)?,
                    recursive: row.get(2)?
                }
            )))?.collect();

        Ok(result?)
    }

    pub fn begin(&self) -> AppResultU {
        info!("BEGIN");
        self.connection.execute("BEGIN;", [])?;
        Ok(())
    }

    pub fn clear_tags(&self, path: &str, source: &str) -> AppResultU {
        self.check_path_existence(path)?;
        self.connection.execute(sql!(clear_tags), &[path, &source])?;
        Ok(())
    }

    pub fn commit(&self) -> AppResultU {
        info!("COMMIT");
        self.connection.execute("COMMIT;", [])?;
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

    pub fn delete_tags(&self, path: &str, tags: &[Tag], source: &str) -> AppResultU {
        for tag in tags {
            let args = &[&tag as &dyn ToSql, &path as &dyn ToSql, &source as &dyn ToSql];
            self.connection.execute(sql!(delete_tag), args)?;
        }
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
        add_distance_function(&connection)?;
        add_match_functions(&connection)?;
        add_recent_function(&connection)?;
        create_table(&connection)?;
        Ok(Database { connection })
    }

    pub fn path_exists(&self, path: &str) -> AppResult<bool> {
        let mut stmt = self.connection.prepare("SELECT 1 FROM images WHERE path = ?;")?;
        Ok(stmt.exists(&[&path as &dyn ToSql])?)
    }

    pub fn reset(&self) -> AppResultU {
        self.connection.execute("DROP TABLE images", [])?;
        self.connection.execute("DROP TABLE tags", [])?;
        create_table(&self.connection)?;
        Ok(())
    }

    pub fn search_history(&self) -> AppResult<Vec<SearchHistory>> {
        let mut stmt = self.connection.prepare("SELECT expression, uses FROM search_history ORDER BY modified DESC")?;
        let result: rusqlite::Result<Vec<SearchHistory>> = stmt.query_map(
            [],
            |row: &Row|
            Ok(
                SearchHistory {
                    expression: row.get(0)?,
                    uses: row.get(1)?
                }
            )
        )?.collect();

        Ok(result?)
    }

    pub fn select<F>(&self, where_expression: &str, vacuum: bool, mut f: F) -> AppResultU where F: FnMut(&Meta, bool) -> AppResultU {
        let mut stmt = self.connection.prepare(&format!("{}{}", SELECT_PREFIX, where_expression))?;
        let iter = stmt.query_and_then([], from_row)?;

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

    pub fn set_tags(&self, path: &str, tags: &[Tag], source: &str) -> AppResultU {
        self.clear_tags(path, source)?;
        self.add_tags(path, tags, source)?;
        Ok(())
    }

    pub fn tags(&self) -> AppResult<Vec<String>> {
        let mut stmt = self.connection.prepare("SELECT DISTINCT tag FROM tags ORDER BY length(tag)")?;
        let result: rusqlite::Result<Vec<String>> = stmt.query_map([], |row: &Row| row.get(0))?.collect();
        Ok(result?)
    }

    pub fn tags_by_path(&self, path: &str) -> AppResult<Vec<String>> {
        let mut stmt = self.connection.prepare("SELECT tag FROM tags WHERE path = ?1")?;
        let result: rusqlite::Result<Vec<String>> = stmt.query_map(&[path], |row: &Row| row.get(0))?.collect();
        Ok(result?)
    }

    pub fn get_total_images(&self, prefix: Option<&str>) -> AppResult<u64> {
        let sql = format!("SELECT COUNT(*) FROM images {}", maybe_prefixed_where_clause(prefix));
        let mut stmt = self.connection.prepare(&sql)?;
        let mut iter = if let Some(path) = prefix {
            stmt.query(&[&path as &dyn ToSql])?
        } else {
            stmt.query([])?
        };
        let r = iter.next()?.ok_or(AppError::Standard("No records"));
        let r: i64 = r?.get(0)?;
        Ok(r as u64)
    }

    pub fn transaction(&self) -> AppResult<Tx> {
        self.begin()?;
        let tx = Tx { database: self };
        Ok(tx)
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
            &meta.dhash as &dyn ToSql,
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

    pub fn _vacuum<F>(&self, prefix: Option<&str>, mut f: F) -> AppResultU where F: FnMut(&Meta, u64, bool) -> AppResultU {
        let mut current: u64 = 0;
        let sql = format!("SELECT * FROM images {}", maybe_prefixed_where_clause(prefix));
        let mut stmt = self.connection.prepare(&sql)?;

        let iter = if let Some(path) = prefix {
            stmt.query_and_then(&[&path as &dyn ToSql], from_row)?
        } else {
            stmt.query_and_then([], from_row)?
        };

        for it in iter {
            let it = it?;
            current += 1;
            let is_target = !Path::new(&it.file.path).is_file();
            if is_target {
                self.delete_path(&it.file.path)?;
            }
            f(&it, current, is_target)?;
        }
        Ok(())
    }

    pub fn check_path_existence(&self, path: &str) -> AppResultU {
        if self.path_exists(path)? {
            return Ok(())
        }
        Err(AppError::PathNotFound(path.to_owned()))
    }
}


impl<'a> Drop for Tx<'a> {
    fn drop(&mut self) {
        self.database.commit().expect("commit");
    }
}


fn create_table(conn: &Connection) -> AppResultU {
    fn create(conn: &Connection, sql: &str) -> AppResultU {
        conn.execute(sql, [])?;
        Ok(())
    }
    create(conn, sql!(create_images_table))?;
    create(conn, sql!(create_tags_table))?;
    create(conn, sql!(create_tags_index))?;
    create(conn, sql!(create_aliases_table))?;
    create(conn, sql!(create_search_history_table))?;
    Ok(())
}

fn from_row(row: &Row) -> AppResult<Meta> {
    use crate::image_format::{from_raw, ImageFormatExt};
    use crate::meta::*;

    let result = Meta {
        animation: row.get(6)?,
        dhash: row.get(8)?,
        dimensions: Dimensions {
            width: row.get(1)?,
            height: row.get(2)?,
        },
        format: from_raw(row.get_ref_unwrap(5))?.to_str(),
        file: FileMeta {
            path: row.get(0)?,
            size: row.get(7)?,
            created: row.get(9)?,
            modified: row.get(10)?,
            accessed: row.get(11)?,
        },
    };
    Ok(result)
}

fn maybe_prefixed_where_clause(prefix: Option<&str>) -> &'static str {
    if prefix.is_some() {
        "WHERE path like (?1 || '%')"
    } else {
        ""
    }
}
