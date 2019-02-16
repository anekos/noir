
use std::process::exit;

#[macro_use] extern crate clap;
use clap::{Arg, SubCommand};
use failure::Fail;
use app_dirs::{AppInfo, AppDataType, get_app_dir};

mod database;
mod errors;
mod loader;
mod meta;
mod search;

use crate::errors::AppResultU;
use crate::database::Database;



const APP_INFO: AppInfo = AppInfo { name: "image-db", author: "anekos" };


fn main() {
    if let Err(err) = app() {
        let mut fail: &Fail = &err;
        let mut message = err.to_string();
        while let Some(cause) = fail.cause() {
            message.push_str(&format!("\n\tcaused by: {}", cause));
            fail = cause;
        }
        eprintln!("{}\n", message);
        exit(1);
    }
}


fn app() -> AppResultU {
    let app = app_from_crate!()
        .subcommand(SubCommand::with_name("load")
                    .alias("l")
                    .about("Load directory")
                    .arg(Arg::with_name("directory")
                         .required(true)
                         .min_values(1)))
        .subcommand(SubCommand::with_name("select")
                    .alias("s")
                    .about("Select SQL")
                    .arg(Arg::with_name("where")
                         .required(true)
                         .min_values(1)));

    let matches = app.get_matches();
    let db = {
        let mut db = get_app_dir(AppDataType::UserData, &APP_INFO, "db").unwrap();
        db.push("default.sqlite");
        Database::open(&db)?
    };

    if let Some(ref matches) = matches.subcommand_matches("load") {
        let directories: Vec<&str> = matches.values_of("directory").unwrap().collect();
        for directory in directories {
            command_load(&db, directory)?;
        }
    } else if let Some(ref matches) = matches.subcommand_matches("select") {
        let wheres: Vec<&str> = matches.values_of("where").unwrap().collect();
        command_select(&db, &wheres)?;
    }

    db.close()?;

    Ok(())
}

fn command_load(db: &Database, directory: &str) -> AppResultU {
    println!("Load directory: {}", directory);
    crate::loader::load(&db, &directory)?;
    Ok(())
}

fn command_select(db: &Database, wheres: &[&str]) -> AppResultU {
    let mut joined = "".to_owned();
    for (index, it) in wheres.iter().enumerate() {
        if 0 < index {
            joined.push(' ');
        }
        joined.push_str(it);
    }
    for it in crate::search::select(db, &joined)? {
        println!("it: {:?}", it);
    }
    Ok(())
}
