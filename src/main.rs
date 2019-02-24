
use std::fs::File;
use std::io::{BufReader, BufWriter, stdin, stdout, Write};
use std::process::exit;
use std::path::{Path, PathBuf};

use app_dirs::{AppInfo, AppDataType, get_app_dir};
use if_let_return::if_let_some;

mod alias;
mod args;
mod database;
mod errors;
mod loader;
mod meta;

use crate::alias::AliasTable;
use crate::database::Database;
use crate::errors::{AppError, AppResultU};



const APP_INFO: AppInfo = AppInfo { name: "noir", author: "anekos" };


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
        let name = matches.value_of("name");
        let expressions: Option<Vec<&str>> = matches.values_of("expression").map(|it| it.collect());
        let recursive = matches.is_present("recursive");
        command_alias(&mut aliases, name, expressions, recursive)?;
    } else if let Some(ref matches) = matches.subcommand_matches("completions") {
        let shell = matches.value_of("shell").unwrap();
        args::build_cli().gen_completions_to("image-db", shell.parse().unwrap(), &mut stdout());
    } else if let Some(ref matches) = matches.subcommand_matches("load") {
        let paths: Vec<&str> = matches.values_of("path").unwrap().collect();
        let check = matches.is_present("check-extension");
        let tag_generator = matches.value_of("tag-script");
        command_load(&db, check, &paths, tag_generator)?;
    } else if let Some(ref matches) = matches.subcommand_matches("load-list") {
        let paths: Vec<&str> = matches.values_of("list-file").unwrap().collect();
        let check = matches.is_present("check-extension");
        let tag_generator = matches.value_of("tag-script");
        command_load_list(&db, check, &paths, tag_generator)?;
    } else if let Some(ref matches) = matches.subcommand_matches("select") {
        let wheres: Vec<&str> = matches.values_of("where").unwrap().collect();
        command_select(&db, &join(&wheres, Some(&aliases)))?;
    } else if let Some(ref matches) = matches.subcommand_matches("tag") {
        if let Some(ref matches) = matches.subcommand_matches("add") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(|it| it.collect()).unwrap_or_else(|| vec![]);
            command_tag_add(&db, path, &tags)?;
        } else if let Some(ref matches) = matches.subcommand_matches("clear") {
            let path: &str = matches.value_of("path").unwrap();
            command_tag_clear(&db, path)?;
        } else if let Some(ref matches) = matches.subcommand_matches("remove") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(|it| it.collect()).unwrap_or_else(|| vec![]);
            command_tag_remove(&db, path, &tags)?;
        } else if let Some(ref matches) = matches.subcommand_matches("set") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(|it| it.collect()).unwrap_or_else(|| vec![]);
            command_tag_set(&db, path, &tags)?;
        } else {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    } else if let Some(ref matches) = matches.subcommand_matches("unalias") {
        let name = matches.value_of("name").unwrap();
        command_unalias(&mut aliases, name);
    } else {
        eprintln!("{}", matches.usage());
        exit(1);
    }

    db.close()?;
    aliases.save(&aliases_file)?;

    Ok(())
}

fn command_alias(aliases: &mut AliasTable, name: Option<&str>, expressions: Option<Vec<&str>>, recursive: bool) -> AppResultU {
    if_let_some!(name = name, {
        for name in aliases.names() {
            println!("{}", name);
        }
        Ok(())
    });
    if_let_some!(expressions = expressions, {
        println!("{}", aliases.expand(name));
        Ok(())
    });
    aliases.alias(name.to_owned(), join(&expressions, None), recursive);
    Ok(())
}

fn command_load(db: &Database, check_extension: bool, paths: &[&str], tag_generator: Option<&str>) -> AppResultU {
    let loader = loader::Loader::new(db, check_extension, tag_generator);
    for path in paths {
        loader.load(&path)?;
    }
    Ok(())
}

fn command_load_list(db: &Database, check_extension: bool, mut paths: &[&str], tag_generator: Option<&str>) -> AppResultU {
    let loader = loader::Loader::new(db, check_extension, tag_generator);
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

fn command_tag_add(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    db.add_tags(path, tags)
}

fn command_tag_clear(db: &Database, path: &str) -> AppResultU {
    db.clear_tags(path)
}

fn command_tag_remove(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    db.remove_tags(path, tags)
}

fn command_tag_set(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    db.set_tags(path, tags)
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
