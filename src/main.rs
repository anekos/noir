
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

mod alias;
mod app;
mod args;
mod database;
mod errors;
mod expander;
mod global_alias;
mod image_format;
mod loader;
mod meta;
mod output_format;
mod tag;

use crate::errors::AppError;



const MAX_RETRY: usize = 10;


fn main() {
    env_logger::init();
    run(1);
}

fn run(tries: usize) {
    use rusqlite::{Error as RE};
    use libsqlite3_sys::ErrorCode;

    match app::run() {
        Err(AppError::Void) | Ok(()) => (),
        Err(AppError::Sqlite(RE::SqliteFailure(error, _))) if tries <= MAX_RETRY => {
            match error.code {
                ErrorCode::DatabaseBusy => {
                    eprintln!("{}", error);
                    eprintln!("Retrying: {}", tries);
                    sleep(Duration::from_millis(1000));
                    run(tries + 1)
                },
                _ => {
                    eprintln!("{}", error);
                    exit(1);
                },
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
    }

}
