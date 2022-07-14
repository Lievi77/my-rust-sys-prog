//library
use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

//Generic type, can be anything
type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)] //derive macro adds the Debug trait so the strct can be printed
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_files: bool,
    //Challenge: implement -E (display a $ at the end of each line)
    show_end_line: bool
}

//by default, all variables and functions in a module are private
// to allow access, use the pub keyword
pub fn run(config: Config) -> MyResult<()> {
    //dbg!(config); //debug macro to print config

    for filename in config.files {
        //for loop with in construct
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(e) => {
                let mut last_non_blank = 0;
                let ending = if config.show_end_line {"$"} else {""};

                for (index, line) in e.lines().enumerate() {
                    let line = line?; //unpack line

                    if config.number_lines {
                        println!("{:>6}\t{}{}", index + 1, line, ending);
                    } else if config.number_nonblank_files {
                        if !line.is_empty() {
                            last_non_blank += 1;
                            println!("{:>6}\t{}{}", last_non_blank, line, ending);
                        } else {
                            println!("{}", ending);
                        }
                    }
                    else {
                        println!("{}{}", line, ending);
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        //box is a smart pointer to a memory location in the heap
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

//Function that gets user arguments
pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("catr")
        .version("0.1.0")
        .author("Lev")
        .about("Rust cat")
        // implement the options -b|-- number-nonblank and -n|--number
        .arg(
            Arg::new("number")
                .takes_value(false)
                .help("Enumerates all file lines")
                .short('n')
                .long("number")
                //To show that two arguments are mutually exclusive, use .conflicts_with
                .conflicts_with("number-nonblank"),
        )
        .arg(
            Arg::new("number-nonblank")
                .takes_value(false)
                .help("Enumerates all file lines")
                .long("number-nonblank")
                .short('b'),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-") //for stdin
                .allow_invalid_utf8(true)
                .min_values(1),
        )
        .arg(
            Arg::new("show-ends")
            .takes_value(false)
            .short('e')
            .long("show-ends")
            .help("display a $ at the end of each line")
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_files: matches.is_present("number-nonblank"),
        show_end_line : matches.is_present("show-ends")
    })
}
