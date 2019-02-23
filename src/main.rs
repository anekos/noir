
use std::fs::File;
use std::io::{BufReader, BufWriter, stdin, stdout, Write};
use std::process::exit;
use std::path::{Path, PathBuf};

use app_dirs::{AppInfo, AppDataType, get_app_dir};

mod alias;
mod args;
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
    let matches = crate::args::build_cli().get_matches();
    let db = {
        let path: PathBuf = if let Some(path) = matches.value_of("database-path") {
            Path::new(path).to_owned()
        } else {
            let mut path = get_app_dir(AppDataType::UserData, &APP_INFO, "db")?;
            let name = matches.value_of("database-name").unwrap_or("default");
            path.push(format!("{}.sqlite", name));
            path
        };
        Database::open(&path)?
    };
    let aliases_file = get_app_dir(AppDataType::UserConfig, &APP_INFO, "aliases.yaml").unwrap();
    let mut aliases = AliasTable::open(&aliases_file)?;

    if let Some(ref matches) = matches.subcommand_matches("alias") {
        let name = matches.value_of("name").unwrap().to_owned();
        let expressions: Vec<&str> = matches.values_of("expression").unwrap().collect();
        command_alias(&mut aliases, name, join(&expressions, None));
    } else if let Some(ref matches) = matches.subcommand_matches("completions") {
        let shell = matches.value_of("shell").unwrap();
        args::build_cli().gen_completions_to("image-db", shell.parse().unwrap(), &mut stdout());
    } else if let Some(ref matches) = matches.subcommand_matches("load") {
        let paths: Vec<&str> = matches.values_of("path").unwrap().collect();
        let check = matches.is_present("check-extension");
        command_load(&db, check, &paths)?;
    } else if let Some(ref matches) = matches.subcommand_matches("load-list") {
        let paths: Vec<&str> = matches.values_of("list-file").unwrap().collect();
        let check = matches.is_present("check-extension");
        command_load_list(&db, check, &paths)?;
    } else if let Some(ref matches) = matches.subcommand_matches("select") {
        let wheres: Vec<&str> = matches.values_of("where").unwrap().collect();
        command_select(&db, &join(&wheres, Some(&aliases)))?;
    } else if let Some(ref matches) = matches.subcommand_matches("unalias") {
        let name = matches.value_of("name").unwrap();
        command_unalias(&mut aliases, name);
    }

    db.close()?;
    aliases.save(&aliases_file)?;

    Ok(())
}

fn command_alias(aliases: &mut AliasTable, name: String, expression: String) {
    aliases.alias(name, expression);
}

fn command_load(db: &Database, check_extension: bool, paths: &[&str]) -> AppResultU {
    let loader = loader::Loader::new(db, check_extension);
    for path in paths {
        loader.load(&path)?;
    }
    Ok(())
}

fn command_load_list(db: &Database, check_extension: bool, mut paths: &[&str]) -> AppResultU {
    let loader = loader::Loader::new(db, check_extension);
    if paths.is_empty() {
        paths = &["-"];
    }
    for path in paths {
        if &"-" == path {
            let input = stdin();
            let mut input = input.lock();
            loader.load_list(&mut input)?;
        } else {
            let file = File::open(path)?;
            let mut file = BufReader::new(file);
            loader.load_list(&mut file)?;
        }
    }
    Ok(())
}

fn command_select(db: &Database, expression: &str) -> AppResultU {
    let output = stdout();
    let output = output.lock();
    let mut output = BufWriter::new(output);
    db.select(expression, |path| {
        writeln!(output, "{}", path)?;
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
