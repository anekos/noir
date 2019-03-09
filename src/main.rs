
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use clap::ArgMatches;

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

use crate::errors::{AppError, AppResult, AppResultU};



const MAX_RETRY_DEFAULT: &str = "10";


fn main() {
    env_logger::init();
    if let Err(error) = _main() {
        eprintln!("{}", error);
        exit(1);
    }
}

fn _main() -> AppResultU {
    let (matches, max_retry) = parse_args()?;
    run(matches, 1, max_retry)
}

fn parse_args() -> AppResult<(ArgMatches<'static>, usize)> {
    let matches = crate::args::build_cli().get_matches();
    let max_retry = matches.value_of("max-retry").unwrap_or(MAX_RETRY_DEFAULT).parse()?;
    Ok((matches, max_retry))
}

fn run(matches: ArgMatches, tries: usize, max_retry: usize) -> AppResultU {
    use rusqlite::{Error as RE};
    use libsqlite3_sys::ErrorCode;

    match app::run(&matches) {
        Err(AppError::Void) => Ok(()),
        Err(AppError::Sqlite(RE::SqliteFailure(error, _))) if tries <= max_retry && error.code == ErrorCode::DatabaseBusy => {
            eprintln!("{}", error);
            eprintln!("Retrying: {}", tries);
            sleep(Duration::from_millis(1000));
            run(matches, tries + 1, max_retry)
        },
        result => result,
    }
}
