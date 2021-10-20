
use std::borrow::ToOwned;
use std::fs::File;
use std::io::{BufReader, BufWriter, stderr, stdin, stdout, Write};
use std::iter::Iterator;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use app_dirs::{AppInfo, AppDataType, get_app_dir};
use clap::ArgMatches;
use if_let_return::if_let_some;
use indicatif::{ProgressBar, ProgressStyle};

use crate::args;
use crate::database::Database;
use crate::server::download::Manager;
use crate::errors::{AppResult, AppResultU, from_path};
use crate::expander::Expander;
use crate::global_alias::GlobalAliasTable;
use crate::loader::Config;
use crate::loader;
use crate::output_format::OutputFormat;
use crate::server::start as start_server;
use crate::tag::Tag;



const APP_INFO: AppInfo = AppInfo { name: "noir", author: "anekos" };


pub fn run(matches: &ArgMatches) -> AppResultU {
    let db_file = {
        if let Some(path) = matches.value_of("database-path") {
            Path::new(path).to_owned()
        } else {
            let mut path = get_app_dir(AppDataType::UserData, &APP_INFO, "db")?;
            let name = matches.value_of("database-name").unwrap_or("default");
            path.push(format!("{}.sqlite", name));
            path
        }
    };
    let aliases_file = {
        if let Some(path) = matches.value_of("alias-file") {
            Path::new(path).to_owned()
        } else {
            get_app_dir(AppDataType::UserConfig, &APP_INFO, "aliases.yaml").unwrap()
        }
    };
    let db = Database::open(&db_file)?;
    let mut aliases = GlobalAliasTable::open(&aliases_file, &db)?;

    if let Some(matches) = matches.subcommand_matches("alias") {
        let name = matches.value_of("name");
        let expressions: Option<Vec<&str>> = matches.values_of("expression").map(Iterator::collect);
        let recursive = matches.is_present("recursive");
        let local = matches.is_present("local");
        command_alias(&db, aliases, name, expressions, recursive, local)?;
    } else if let Some(matches) = matches.subcommand_matches("completions") {
        let shell = matches.value_of("shell").unwrap();
        args::build_cli().gen_completions_to("noir", shell.parse().unwrap(), &mut stdout());
    } else if let Some(matches) = matches.subcommand_matches("expand") {
        let expression = matches.value_of("expression").unwrap();
        let full = matches.is_present("full");
        command_expand(&db, aliases, expression, full)?;
    } else if let Some(matches) = matches.subcommand_matches("get") {
        let path = matches.value_of("path").unwrap();
        let format = matches.value_of("format").map(OutputFormat::from_str).unwrap_or(Ok(OutputFormat::Simple))?;
        command_get(&db, path, format)?;
    } else if matches.subcommand_matches("history").is_some() {
        command_history(&db)?;
    } else if let Some(matches) = matches.subcommand_matches("load") {
        let paths: Vec<&str> = matches.values_of("path").unwrap().collect();
        command_load(&db, &paths, extract_loader_config(matches))?;
    } else if let Some(matches) = matches.subcommand_matches("load-list") {
        let paths: Vec<&str> = matches.values_of("list-file").unwrap().collect();
        command_load_list(&db, &paths, extract_loader_config(matches))?;
    } else if matches.is_present("path") {
        println!("{}", from_path(&db_file)?);
    } else if matches.is_present("reset") {
        command_reset(&db)?;
    } else if let Some(matches) = matches.subcommand_matches("search") {
        let wheres: Vec<&str> = matches.values_of("where").unwrap().collect();
        let vacuum = matches.is_present("vacuum");
        let format = matches.value_of("format").map(OutputFormat::from_str).unwrap_or(Ok(OutputFormat::Simple))?;
        command_search(&db, aliases, &join(&wheres), vacuum, format)?;
    } else if let Some(matches) = matches.subcommand_matches("server") {
        let port: u16 = matches.value_of("port").unwrap_or("9696").parse()?;
        let root: &str = matches.value_of("root").unwrap_or("static");
        let download_to: Option<&str> = matches.value_of("download-to");
        return command_server(db, &db_file, aliases, port, root, download_to);
    } else if let Some(matches) = matches.subcommand_matches("tag") {
        if let Some(matches) = matches.subcommand_matches("add") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(Iterator::collect).unwrap_or_default();
            command_tag_add(&db, path, &tags)?;
        } else if let Some(matches) = matches.subcommand_matches("clear") {
            let path: &str = matches.value_of("path").unwrap();
            command_tag_clear(&db, path)?;
        } else if let Some(matches) = matches.subcommand_matches("remove") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(Iterator::collect).unwrap_or_default();
            command_tag_remove(&db, path, &tags)?;
        } else if let Some(matches) = matches.subcommand_matches("set") {
            let path: &str = matches.value_of("path").unwrap();
            let tags: Vec<&str> = matches.values_of("tag").map(Iterator::collect).unwrap_or_default();
            command_tag_set(&db, path, &tags)?;
        } else if let Some(matches) = matches.subcommand_matches("show") {
            let path: Option<&str> = matches.value_of("path");
            command_tag_show(&db, path)?;
        } else {
            eprintln!("{}", matches.usage());
            exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("unalias") {
        let name = matches.value_of("name").unwrap();
        let local = matches.is_present("local");
        command_unalias(&db, &mut aliases, name, local)?;
    } else if let Some(matches) = matches.subcommand_matches("vacuum") {
        let prefix: Option<&str> = matches.value_of("prefix");
        command_vacuum(&db, prefix)?;
    } else {
        eprintln!("{}", matches.usage());
        exit(1);
    }

    // aliases.save(&aliases_file)?;

    Ok(())
}

fn command_alias(db: &Database, mut aliases: GlobalAliasTable, name: Option<&str>, expressions: Option<Vec<&str>>, recursive: bool, local: bool) -> AppResultU {
    if_let_some!(name = name, {
        for name in aliases.names() {
            println!("{}", name);
        }
        Ok(())
    });
    if_let_some!(expressions = expressions, {
        let expander = Expander::generate(db, &aliases)?;
        println!("{}", expander.expand(name));
        Ok(())
    });
    let expression = join(&expressions);
    if local {
        db.upsert_alias(name, &expression, recursive)?;
    } else {
        aliases.add(name.to_owned(), expression, recursive);
        aliases.save()?;
    }
    Ok(())
}

fn command_expand(db: &Database, aliases: GlobalAliasTable, expression: &str, full: bool) -> AppResultU {
    let expander = Expander::generate(db, &aliases)?;
    let expanded = expander.expand(expression);
    if full {
        println!("{}{}", crate::database::SELECT_PREFIX, expanded);
    } else {
        println!("{}", expanded);
    }
    Ok(())
}

fn command_get(db: &Database, path: &str, format: OutputFormat) -> AppResultU {
    if let Some(meta) = db.get(path)? {
        let output = stdout();
        let mut output = output.lock();
        format.write(&mut output, &meta)?;
    } else {
        eprintln!("Entry Not found");
        exit(1);
    }
    Ok(())
}

fn command_history(db: &Database) -> AppResultU {
    for entry in db.search_history()? {
        println!("{}", entry.expression);
    }
    Ok(())
}

fn command_load(db: &Database, paths: &[&str], config: Config) -> AppResultU {
    let _tx = db.transaction()?;
    let mut loader = loader::Loader::new(db, config);
    for path in paths {
        loader.load(&path)?;
    }
    Ok(())
}

fn command_load_list(db: &Database, mut paths: &[&str], config: Config) -> AppResultU {
    let mut loader = loader::Loader::new(db, config);
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

fn command_search(db: &Database, aliases: GlobalAliasTable, expression: &str, vacuum: bool, format: OutputFormat) -> AppResultU {
    let error = stderr();
    let error = error.lock();
    let output = stdout();
    let output = output.lock();

    let mut error = BufWriter::new(error);
    let mut output = BufWriter::new(output);

    let expander = Expander::generate(db, &aliases)?;
    let expanded_expression = expander.expand(expression);

    db.select(&expanded_expression, vacuum, |meta, vacuumed| {
        if vacuumed {
            writeln!(error, "Vacuumed: {}", meta.file.path)?;
        } else {
            format.write(&mut output, meta)?;
        }
        Ok(())
    })?;

    db.add_search_history(&expression)
}

fn command_server<T: AsRef<Path>>(db: Database, db_file: &T, aliases: GlobalAliasTable, port: u16, root: &str, download_to: Option<&str>) -> AppResultU {
    let manager = Manager::new(Database::open(db_file)?);
    start_server(db, manager, aliases, port, root.to_owned(), download_to.map(ToOwned::to_owned))?;
    Ok(())
}

fn command_tag_add(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    let tags = to_tags(tags)?;
    db.add_tags(path, tags.as_slice())
}

fn command_tag_clear(db: &Database, path: &str) -> AppResultU {
    db.clear_tags(path)
}

fn command_tag_remove(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    let tags = to_tags(tags)?;
    db.delete_tags(path, tags.as_slice())
}

fn command_tag_set(db: &Database, path: &str, tags: &[&str]) -> AppResultU {
    let tags = to_tags(tags)?;
    db.set_tags(path, tags.as_slice())
}

fn command_tag_show(db: &Database, path: Option<&str>) -> AppResultU {
    if let Some(path) = path {
        let path = Path::new(path).canonicalize()?;
        for tag in db.tags_by_path(from_path(&path)?)? {
            println!("{}", tag);
        }
    } else {
        for tag in db.tags()? {
            println!("{}", tag);
        }
    }
    Ok(())
}

fn command_reset(db: &Database) -> AppResultU {
    let stdin = stdin();
    let mut input = "".to_owned();
    print!("Are you sure? (yes/NO): ");
    stdout().flush()?;
    stdin.read_line(&mut input)?;
    if input.to_lowercase() == "yes\n" {
        db.reset()?;
        println!("All data have been deleted.")
    } else {
        println!("Canceled")
    }
    Ok(())
}

fn command_unalias(db: &Database, aliases: &mut GlobalAliasTable, name: &str, local: bool) -> AppResultU {
    if local {
        db.delete_alias(name)?;
    } else {
        aliases.delete(name);
        aliases.save()?;
    }
    Ok(())
}

fn command_vacuum(db: &Database, prefix: Option<&str>) -> AppResultU {
    let total_images = db.get_total_images(prefix)?;
    let mut vacummed_images: u64 = 0;
    let bar = ProgressBar::new(total_images);
    let sty = ProgressStyle::default_bar()
                .template("{elapsed_precise} {bar:40.cyan/blue} {pos:>7}/{len:7} (ETA {eta}) {msg} images")
                        .progress_chars("██░");
    bar.set_style(sty);
    db._vacuum(prefix, |_meta, _current, vacuumed| {
        if vacuumed {
            vacummed_images += 1;
        }
        bar.set_message(format!("{}", vacummed_images));
        bar.inc(1);
        Ok(())
    })?;
    Ok(())
}

fn extract_loader_config<'a>(matches: &'a ArgMatches) -> Config<'a> {
    let check_extension = matches.is_present("check-extension");
    let compute_dhash = matches.is_present("dhash");
    let dry_run = matches.is_present("dry-run");
    let skip_errors = matches.is_present("skip-errors");
    let tag_generator = matches.value_of("tag-script");
    let update = matches.is_present("update");
    Config { check_extension, compute_dhash, dry_run, skip_errors, tag_generator, update }
}

fn join(strings: &[&str]) -> String {
    let mut joined = "".to_owned();
    for (index, it) in strings.iter().enumerate() {
        if 0 < index {
            joined.push(' ');
        }
        joined.push_str(it);
    }
    joined
}

fn to_tags(tags: &[&str]) -> AppResult<Vec<Tag>> {
    tags.iter().map(|it| Tag::from_str(it)).collect()
}
