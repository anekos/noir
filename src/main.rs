
use std::io::Write;
use std::process::exit;

#[macro_use] extern crate clap;
use clap::{Arg, SubCommand};
use app_dirs::{AppInfo, AppDataType, get_app_dir};

mod alias;
mod database;
mod errors;
mod loader;
mod meta;

use crate::alias::AliasTable;
use crate::database::Database;
use crate::errors::{AppError, AppResultU};



const APP_INFO: AppInfo = AppInfo { name: "image-db", author: "anekos" };


fn main() {
    match app() {
        Err(AppError::Void) | Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
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
                    .about("Load directory or file")
                    .arg(Arg::with_name("path")
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
        let paths: Vec<&str> = matches.values_of("path").unwrap().collect();
        command_load(&db, &paths)?;
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

fn command_load(db: &Database, paths: &[&str]) -> AppResultU {
    let loader = loader::Loader::new(db);
    for path in paths {
        loader.load(&path)?;
    }
    Ok(())
}

fn command_select(db: &Database, expression: &str) -> AppResultU {
    let out = std::io::stdout();
    let mut out = out.lock();
    db.select(expression, |path| {
        writeln!(out, "{}", path)?;
        Ok(())
    })
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
