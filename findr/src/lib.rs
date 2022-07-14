use crate::EntryType::*;
use clap::{Arg, Command};
use regex::Regex;
use std::error::Error;
use std::fs;
use walkdir::DirEntry;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    dirs: Vec<String>,
    names: Option<Vec<Regex>>,
    entry_types: Option<Vec<EntryType>>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("findr")
        .version("0.1.0")
        .author("Lev")
        .about("Rust Find")
        .arg(
            Arg::new("dirs")
                .value_name("DIR")
                .help("Search directory")
                .takes_value(true)
                .default_value(".")
                .allow_invalid_utf8(true)
                .min_values(1),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("name")
                .value_name("NAME")
                .allow_invalid_utf8(true)
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::new("type")
                .short('t')
                .long("type")
                .value_name("TYPE")
                .help("Entry type")
                .takes_value(true)
                .possible_values(["f", "d", "l"])
                .multiple(true)
                .allow_invalid_utf8(true),
        )
        .get_matches();

    //handling the names option, which is a vector of regex
    let mut names = vec![];

    if let Some(values) = matches.values_of_lossy("name") {
        for name in values {
            //create a new regex
            match Regex::new(&name) {
                Ok(exp) => names.push(exp),
                _ => return Err(From::from(format!("Invalid --name \"{}\"", name))),
            }
        }
    }

    let entry_types = matches.values_of_lossy("type").map(|vals| {
        vals.iter()
            .filter_map(|val| match val.as_str() {
                "d" => Some(Dir),
                "f" => Some(File),
                "l" => Some(Link),
                _ => None,
            })
            .collect()
    });

    Ok(Config {
        dirs: matches.values_of_lossy("dirs").unwrap(),
        names: if names.is_empty() { None } else { Some(names) },
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    //Approach: use closures alonside with filter

    //filter file types
    let type_filter = |entry: &DirEntry| match &config.entry_types {
        Some(types) => types.iter().any(|t| match t {
            Link => entry.path_is_symlink(),
            Dir => entry.file_type().is_dir(),
            File => entry.file_type().is_file(),
        }),
        _ => true,
    };

    let name_filter = |entry: &DirEntry| match &config.names {
        //config.names can be null -> need a match statement
        //
        Some(names) => names
            .iter()
            .any(|re| re.is_match(&entry.file_name().to_string_lossy())),
        _ => true,
    };

    //iterate through all input directory paths
    for dirname in config.dirs {
        //Walkdir creates an iterator to iterate recursively through directories
        //to check if a string is a directory, use fs::read_dir
        match fs::read_dir(&dirname) {
            Err(e) => eprintln!("{}: {}", dirname, e),
            _ => {
                let entries = WalkDir::new(dirname)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(type_filter)
                    .filter(name_filter)
                    .map(|entry| entry.path().display().to_string())
                    .collect::<Vec<String>>();

                println!("{}", entries.join("\n"));
            }
        }
    }

    Ok(())
}
