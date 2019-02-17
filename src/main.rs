
use std::process::exit;

#[macro_use] extern crate clap;
use clap::{Arg, SubCommand};
use failure::Fail;
use app_dirs::{AppInfo, AppDataType, get_app_dir};

mod alias;
mod database;
mod errors;
mod loader;
mod meta;
mod search;

use crate::alias::AliasTable;
use crate::database::Database;
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
        .subcommand(SubCommand::with_name("alias")
                    .alias("a")
                    .about("Define expression alias")
                    .arg(Arg::with_name("name")
                         .required(true))
                    .arg(Arg::with_name("expression")
                         .required(true)
                         .min_values(1)))
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
                         .min_values(1)))
        .subcommand(SubCommand::with_name("unalias")
                    .alias("s")
                    .about("Unalias")
                    .arg(Arg::with_name("name")
                         .required(true)));

    let matches = app.get_matches();
    let db = {
        let mut db = get_app_dir(AppDataType::UserData, &APP_INFO, "db")?;
        db.push("default.sqlite");
        Database::open(&db)?
    };
    let mut aliases = {
        let aliases = get_app_dir(AppDataType::UserConfig, &APP_INFO, "aliases.yaml").unwrap();
        AliasTable::open(&aliases)?
    };

    if let Some(ref matches) = matches.subcommand_matches("alias") {
        let name = matches.value_of("name").unwrap().to_owned();
        let expressions: Vec<&str> = matches.values_of("expression").unwrap().collect();
        command_alias(&mut aliases, name, join(&expressions, None));
    } else if let Some(ref matches) = matches.subcommand_matches("load") {
        let directories: Vec<&str> = matches.values_of("directory").unwrap().collect();
        for directory in directories {
            command_load(&db, directory)?;
        }
    } else if let Some(ref matches) = matches.subcommand_matches("select") {
        let wheres: Vec<&str> = matches.values_of("where").unwrap().collect();
        command_select(&db, &join(&wheres, Some(&aliases)))?;
    } else if let Some(ref matches) = matches.subcommand_matches("unalias") {
        let name = matches.value_of("name").unwrap();
        command_unalias(&mut aliases, name);
    }

    db.close()?;
    aliases.close()?;

    Ok(())
}

fn command_alias(aliases: &mut AliasTable, name: String, expression: String) {
    aliases.alias(name, expression);
}

fn command_load(db: &Database, directory: &str) -> AppResultU {
    println!("Load directory: {}", directory);
    crate::loader::load(&db, &directory)?;
    Ok(())
}

fn command_select(db: &Database, expression: &str) -> AppResultU {
    for it in crate::search::select(db, expression)? {
        println!("it: {:?}", it);
    }
    Ok(())
}

fn command_unalias(aliases: &mut AliasTable, name: &str) {
    aliases.unalias(name);
}

fn join(strings: &[&str], aliases: Option<&AliasTable>) -> String {
    let mut joined = "".to_owned();
    for (index, it) in strings.iter().enumerate() {
        if 0 < index {
            joined.push(' ');
        }
        if let Some(aliases) = aliases {
            joined.push_str(&aliases.expand(&it));
        } else {
            joined.push_str(it);
        }
    }
    joined
}
