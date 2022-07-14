use clap::{Arg, Command};
use std::fs::File;
use std::io::BufReader;
use std::{
    error::Error,
    io::{self, BufRead},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    char: bool,
}

//custom structure to help us keep track of a file's metrics
#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("wcr")
        .version("0.1.0")
        .author("Lev")
        .about("Rust wc")
        .arg(
            Arg::new("lines")
                .help("Show line count")
                .short('l')
                .long("lines")
                .takes_value(false),
        )
        .arg(
            Arg::new("bytes")
                .help("Show byte count")
                .short('c')
                .long("bytes")
                .takes_value(false)
                //Mimics BSD version of the utility
                .conflicts_with("chars"),
        )
        .arg(
            Arg::new("words")
                .help("Show word count")
                .short('w')
                .long("words")
                .takes_value(false),
        )
        .arg(
            Arg::new("chars")
                .help("Show character count")
                .short('m')
                .long("chars")
                .takes_value(false), // .conflicts_with("bytes"),
        )
        .arg(
            Arg::new("files")
                .help("Input file(s)")
                .value_name("FILE(S)")
                .takes_value(true)
                .default_value("-")
                .allow_invalid_utf8(true)
                .min_values(1),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let mut char = matches.is_present("chars");

    //clever way to check if all options are absent
    if [lines, words, bytes, char].iter().all(|v| v == &false) {
        //tip: read the iterator documentation

        //all are false, thus go with default values
        lines = true;
        words = true;
        bytes = true;
        char = false;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        char,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        match open(filename) {
            Err(err) => eprint!("{}: {}", filename, err),
            Ok(file) => {
                if let Ok(info) = count(file) {
                    println!(
                        "{}{}{}{}{}",
                        format_field(info.num_lines, config.lines),
                        format_field(info.num_words, config.words),
                        format_field(info.num_bytes, config.bytes),
                        format_field(info.num_chars, config.char),
                        if filename == "-" {
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    );
                    total_lines += info.num_lines;
                    total_words += info.num_words;
                    total_bytes += info.num_bytes;
                    total_chars += info.num_chars;
                }
            }
        }
    }

    if config.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.char)
        )
    }

    Ok(())
}

fn format_field(value: usize, show: bool) -> String {
    //str is immutable
    //String is growable, heap allocated structure
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

//opening a file
pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

//impl indicates that a variable must implement the mentioned Trait
pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new(); //buffer

    loop {
        //remember that read_line does not eliminate any endings
        let line_bytes = file.read_line(&mut line)?;

        if line_bytes == 0 {
            break; //we are done
        }
        num_bytes += line_bytes;
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        line.clear() //clear buffer for next line
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

//Unit test for the count function
#[cfg(test)] // cfg enables conditional compilation, module will only be compiled during testing
             //creating a module for unit tests, a tidy way to group them
mod test {
    use super::{count, format_field, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
