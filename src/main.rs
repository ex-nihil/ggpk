mod ggpk;
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::convert::TryFrom;

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::*;
use clap::{Arg, App};
use memmap::Mmap;

mod shared;
use shared::FileRecord;

fn main() {
    let matches = App::new("GGPK Reader")
        .version("1.0")
        .author("Daniel D. <daniel.k.dimovski@gmail.com>")
        .about("Reads the GGPK fileformat from the game Path of Exile")
        .arg(Arg::with_name("path")
            .short("p")
            .long("path")
            .value_name("DIRECTORY")
            .help("Specify location of Path of Exile installation")
            .required_unless("file")
            .takes_value(true))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Specify location of GGPK file")
            .required_unless("path")
            .takes_value(true))
        .arg(Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("QUERY")
            .help("Filter output to include provided substring")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("DIRECTORY")
            .help("Write files to location")
            .takes_value(true))
        .arg(Arg::with_name("silent")
            .short("s")
            .long("silent")
            .help("Prevent writing to stdout"))
        .arg(Arg::with_name("binary")
            .short("b")
            .long("binary")
            .help("Output file contents to stdout"))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();

    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed)
        ]
    ).unwrap();

    let query = matches.value_of("query").unwrap_or("");
    let silent = matches.is_present("silent");

    let file = if matches.is_present("file") {
        matches.value_of("file").unwrap().to_string()
    } else {
        format!("{}/Content.ggpk", matches.value_of("path").unwrap())
    };

    ggpk::read(file.as_str(), |ggpk| {
        ggpk.files.iter()
            .filter(|(filepath, _)| query.is_empty() || filepath.contains(query) )
            .map(|tuple| tuple.1)
            .for_each(|record| {

                if let Some(output) = matches.value_of("output") {
                    let path = format!("{}/{}/{}", output, record.path, record.name);

                    if !silent { println!("Writing {}", path) }
                    create_path(path.as_str());
                    read_record(record, &ggpk.mmap, |bytes| {
                        File::create(path.as_str()).unwrap().write(bytes).unwrap();
                    });

                } else if matches.is_present("binary") {
                    read_record(record, &ggpk.mmap, |bytes| {
                        io::stdout().write_all(bytes).unwrap();
                    });

                } else {
                    println!("{}/{}", record.path, record.name)
                }
            });
    });
}

fn create_path(path: &str) {
    match Path::new(path).parent() {
        Some(directory) => {
            let _ = create_dir_all(directory);
        },
        None => {},
    }
}

pub fn read_record<F>(record: &FileRecord, mmap: &Mmap, callback: F) where F: Fn(&[u8]) {
    let file_end = record.begin + usize::try_from(record.bytes).unwrap();
    match mmap.get(record.begin..file_end) {
        Some(bytes) => {
            callback(bytes);
        },
        None => error!("Failed reading bytes for file {}/{}", record.path, record.name),
    }
}
