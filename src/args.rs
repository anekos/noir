
use clap::*;



pub fn build_cli() -> App<'static, 'static> {
    let format = Arg::with_name("format")
        .help("Output format")
        .short("f")
        .long("format")
        .takes_value(true);

    app_from_crate!()
        .arg(Arg::with_name("database-name")
             .help("Database name")
             .short("n")
             .long("name")
             .takes_value(true))
        .arg(Arg::with_name("database-path")
             .help("Path to *.sqlite")
             .short("p")
             .long("path")
             .takes_value(true))
        .arg(Arg::with_name("alias-file")
             .help("Path to *.yaml")
             .short("a")
             .long("alias")
             .takes_value(true))
        .arg(Arg::with_name("max-retry")
             .help("Maximum retry")
             .long("max-retry")
             .takes_value(true))
        .subcommand(SubCommand::with_name("alias")
                    .alias("a")
                    .about("Define expression alias")
                    .arg(Arg::with_name("local")
                         .help("Database local alias")
                         .short("l")
                         .long("local")
                         .takes_value(false))
                    .arg(Arg::with_name("recursive")
                         .help("Recursive")
                         .short("r")
                         .long("recursive")
                         .takes_value(false))
                    .arg(Arg::with_name("name"))
                    .arg(Arg::with_name("expression")
                         .min_values(0)))
        .subcommand(SubCommand::with_name("completions")
                    .about("Generates completion scripts for your shell")
                    .arg(Arg::with_name("shell")
                         .required(true)
                         .possible_values(&["bash", "fish", "zsh"])
                         .help("The shell to generate the script for")))
        .subcommand(SubCommand::with_name("compute")
                    .about("Compute hashes")
                    .arg(format.clone())
                    .arg(Arg::with_name("where")
                         .help("SQL Where clause")
                         .required(true)
                         .min_values(1))
                    .arg(Arg::with_name("chunk")
                         .help("Chunk size")
                         .long("chunk")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("expand")
                    .about("Show alias expanded expression")
                    .arg(Arg::with_name("full")
                         .help("Full")
                         .short("f")
                         .long("full")
                         .takes_value(false))
                    .arg(Arg::with_name("expression")
                         .required(true)))
        .subcommand(SubCommand::with_name("get")
                    .about("Get image information")
                    .arg(format.clone())
                    .arg(Arg::with_name("path")
                         .required(true)))
        .subcommand(SubCommand::with_name("history")
                    .about("Search expression history"))
        .subcommand(
            load_args(
                SubCommand::with_name("load")
                .alias("l")
                .about("Load directory or file")
                .arg(Arg::with_name("update")
                     .help("Update exising files")
                     .short("u")
                     .long("update")
                     .takes_value(false))
                .arg(Arg::with_name("path")
                     .required(true)
                     .min_values(1))))
        .subcommand(
            load_args(
                SubCommand::with_name("load-list")
                    .alias("l")
                    .about("Load from list file")
                    .arg(Arg::with_name("update")
                         .help("Update exising files")
                         .short("u")
                         .long("update")
                         .takes_value(false))
                    .arg(Arg::with_name("list-file")
                         .required(true)
                         .min_values(0))))
        .subcommand(SubCommand::with_name("meta")
                    .about("Compute metaformation")
                    .arg(Arg::with_name("path")
                         .required(true))
                    .arg(format.clone()))
        .subcommand(SubCommand::with_name("path")
                    .about("Show database path"))
        .subcommand(SubCommand::with_name("reset")
                    .about("Clear all data"))
        .subcommand(SubCommand::with_name("search")
                    .alias("s")
                    .alias("select")
                    .about("Search images")
                    .arg(format)
                    .arg(Arg::with_name("vacuum")
                         .help("Remove entries that do not exist")
                         .short("v")
                         .long("vacuum")
                         .takes_value(false))
                    .arg(Arg::with_name("where")
                         .help("SQL Where clause")
                         .required(true)
                         .min_values(1)))
        .subcommand(SubCommand::with_name("server")
                    .about("Web App")
                    .arg(Arg::with_name("download-to")
                         .help("Download to this directory")
                         .short("d")
                         .long("download-to")
                         .takes_value(true))
                    .arg(Arg::with_name("port")
                         .help("Server port")
                         .short("p")
                         .long("port")
                         .takes_value(true))
                    .arg(Arg::with_name("root")
                         .help("Static file root")
                         .short("r")
                         .long("root")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("tag")
                    .alias("t")
                    .about("Manage tags")
                    .subcommand(SubCommand::with_name("add")
                                .alias("a")
                                .about("Add tags")
                                .arg(Arg::with_name("path")
                                     .required(true))
                                .arg(Arg::with_name("source")
                                    .required(true))
                                .arg(Arg::with_name("tag")
                                     .required(true)
                                     .min_values(1)))
                    .subcommand(SubCommand::with_name("clear")
                                .alias("c")
                                .about("Clear tags")
                                .arg(Arg::with_name("path")
                                     .required(true))
                                .arg(Arg::with_name("source")
                                     .required(true)))
                    .subcommand(SubCommand::with_name("remove")
                                .alias("r")
                                .about("Remove tags")
                                .arg(Arg::with_name("path")
                                     .required(true))
                                .arg(Arg::with_name("source")
                                     .required(true))
                                .arg(Arg::with_name("tag")
                                     .required(true)
                                     .min_values(1)))
                    .subcommand(SubCommand::with_name("set")
                                .alias("s")
                                .about("Set tags")
                                .arg(Arg::with_name("path")
                                     .required(true))
                                .arg(Arg::with_name("source")
                                     .required(true))
                                .arg(Arg::with_name("tag")
                                     .min_values(0)))
                    .subcommand(SubCommand::with_name("show")
                                .alias("S")
                                .about("Show tags")
                                .arg(Arg::with_name("path")
                                     .required(false))))
        .subcommand(SubCommand::with_name("unalias")
                    .alias("s")
                    .about("Unalias")
                    .arg(Arg::with_name("local")
                         .help("Database local alias")
                         .short("l")
                         .long("local")
                         .takes_value(false))
                    .arg(Arg::with_name("name")
                         .required(true)))
        .subcommand(SubCommand::with_name("vacuum")
                    .about("Remove deleted files")
                    .arg(Arg::with_name("prefix")
                         .help("Path prefix")
                         .short("p")
                         .long("prefix")
                         .takes_value(true)))
}

fn load_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(Arg::with_name("tag-script")
            .help("Tag generator script")
            .short("t")
            .long("tag-script")
            .takes_value(true))
        .arg(Arg::with_name("tag-source")
            .help("Tag source")
            .long("tag-source")
            .takes_value(true))
        .arg(Arg::with_name("check-extension")
             .help("Check file extension before load")
             .short("c")
             .long("check-extension"))
        .arg(Arg::with_name("dhash")
             .help("Compute dhash")
             .short("d")
             .long("dhash"))
        .arg(Arg::with_name("dry-run")
             .help("Dry run")
             .long("dry-run")
             .takes_value(false))
        .arg(Arg::with_name("skip-errors")
             .help("Skip errors")
             .short("s")
             .long("skip-errors")
             .takes_value(false))
}
