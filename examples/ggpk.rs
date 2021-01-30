#[macro_use]
extern crate log;
extern crate simplelog;
use clap::{App, Arg};
use regex::Regex;
use simplelog::*;
use std::io;

use ggpk::GGPK;

fn main() {
    let matches = App::new("GGPK Reader")
        .version("1.2.1")
        .author("Daniel D. <daniel@timeloop.se>")
        .about("Reads the GGPK fileformat from the game Path of Exile")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("DIRECTORY")
                .help("Specify location of Path of Exile installation")
                .required_unless("file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Specify location of GGPK file")
                .required_unless("path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("query")
                .short("q")
                .long("query")
                .value_name("QUERY")
                .help("Filter output to include provided substring")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("DIRECTORY")
                .help("Write files to location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("binary")
                .short("b")
                .long("binary")
                .help("Output file contents to stdout"),
        )
        .get_matches();

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
    )])
    .unwrap();

    let query = matches
        .value_of("query")
        .and_then(|re| Regex::new(&re).ok());

    let filepath = if matches.is_present("file") {
        matches.value_of("file").unwrap().to_string()
    } else {
        let path = matches.value_of("path").unwrap();
        format!("{}/Content.ggpk", path)
    };

    let ggpk = open_ggpk(filepath.as_str());
    let files = ggpk.list_files();
    if matches.is_present("binary") {
        files
            .iter()
            .filter(|filepath| is_included(filepath.as_str(), &query))
            .take(1)
            .for_each(|filepath| {
                let file = ggpk.get_file(filepath);
                file.write_into(&mut io::stdout()).unwrap();
            });
    } else if let Some(output) = matches.value_of("output") {
        files
            .iter()
            .filter(|filepath| is_included(filepath.as_str(), &query))
            .for_each(|filepath| {
                let file = ggpk.get_file(filepath);
                let path = format!("{}/{}", output, filepath);
                match file.write_file(path.as_str()) {
                    Ok(_) => info!("Wrote file '{}'", path),
                    Err(e) => error!("Write failed. '{}'. Error: {}", path, e),
                }
            });
    } else {
        files
            .iter()
            .filter(|filepath| is_included(filepath.as_str(), &query))
            .for_each(|filepath| println!("{}", filepath));
    }
}

fn is_included(file: &str, query: &Option<Regex>) -> bool {
    query.as_ref().map(|re| re.is_match(file)).unwrap_or(true)
}

fn open_ggpk(filepath: &str) -> GGPK {
    match GGPK::from_file(filepath) {
        Ok(ggpk) => ggpk,
        Err(e) => {
            error!("Failed reading '{}' into mmap. {}", filepath, e);
            std::process::exit(-1);
        }
    }
}
