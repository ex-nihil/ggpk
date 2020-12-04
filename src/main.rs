#[macro_use]
extern crate log;
extern crate simplelog;
use clap::{App, Arg};
use simplelog::*;
use regex::Regex;
use std::io;

mod ggpk;
mod file;
mod util;
use crate::ggpk::{GGPK, GGPKRead};
use crate::file::{GGPKFileFn};

fn main() {
    let matches = App::new("GGPK Reader")
        .version("1.1.0")
        .author("Daniel D. <daniel.k.dimovski@gmail.com>")
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
            Arg::with_name("silent")
                .short("s")
                .long("silent")
                .help("Prevent writing to stdout"),
        )
        .arg(
            Arg::with_name("binary")
                .short("b")
                .long("binary")
                .help("Output file contents to stdout"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
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

    let ggpk = if let Some(file) = matches.value_of("file") {
        GGPK::from_file(file)
    } else {
        GGPK::from_install(matches.value_of("path").unwrap())
    };

    let files = ggpk.list_files();

    if matches.is_present("binary") {
        files
            .iter()
            .filter(|filepath| is_included(filepath.as_str(), &query))
            .take(1)
            .for_each(|filepath| {
                let file = ggpk.get_file(filepath);
                //let size = (file.record.bytes as u32).try_into().unwrap();
                //let mut dst: Vec<u8> = Vec::with_capacity(size);
                //file.write_into(&mut dst).unwrap();
                file.write_into(&mut io::stdout()).unwrap();
            });
    } else if let Some(output) = matches.value_of("output") {
        files
            .iter()
            .filter(|filepath| is_included(filepath.as_str(), &query))
            .for_each(|filepath| {
                let file = ggpk.get_file(filepath);
                let path = format!("{}/{}", output, filepath);
                println!("Writing {}", path);
                file.write_file(path.as_str()).expect("Write failed");
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
