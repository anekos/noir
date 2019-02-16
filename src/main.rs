
use std::process::exit;

#[macro_use] extern crate clap;
use clap::{Arg, SubCommand};
use failure::Fail;
use app_dirs::{AppInfo, AppDataType, get_app_dir};

mod database;
mod errors;
mod loader;
mod meta;

use crate::errors::AppResultU;


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
                         .required(true)));

    let matches = app.get_matches();

    if let Some(ref matches) = matches.subcommand_matches("load") {
       command_load(matches.value_of("directory").unwrap())?; // Required
    }

    Ok(())
}

fn command_load(directory: &str) -> AppResultU {
    let mut db = get_app_dir(AppDataType::UserData, &APP_INFO, "db").unwrap();
    db.push("default.sqlite");
    println!("Load directory: {} with {:?}", directory, db);
    crate::loader::load(&directory, &db)?;
    Ok(())
}
