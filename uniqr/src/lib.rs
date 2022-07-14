use clap::{Arg, Command};
use std::{
    //importing multiple crates with the same prefix
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("uniqr")
        .version("0.1.0")
        .author("Lev")
        .about("Uniq Rust")
        .arg(
            Arg::new("in_file")
                .value_name("FILE")
                .help("Input file")
                .default_value("-"),
        )
        .arg(Arg::new("out_file").value_name("FILE").help("Output file"))
        .arg(
            Arg::new("count")
                .help("Display unique word count")
                .takes_value(false)
                .short('c')
                .long("count"),
        )
        .get_matches();

    Ok(Config {
        //value_of => single value
        in_file: matches.value_of("in_file").map(str::to_string).unwrap(),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;

    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(out_name) => Box::new(File::create(&out_name)?),

        _ => Box::new(io::stdout()),
    };

    /*
    Example of a closure
    A closure is essentially a function
     */
    let mut print = |count: &u64, line: &String| -> MyResult<()> {
        if count > &0 {
            if config.count {
                write!(out_file, "{:>4} {}", &count, &line)?;
            } else {
                write!(out_file, "{}", &line)?;
            }
        }
        Ok(())
    };

    //iterate and count the unique lines
    let mut line = String::new();
    let mut count: u64 = 0;
    let mut last_line = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != last_line.trim_end() {
            //trim_end removes trailing whitespace
            if count > 0 {
                print(&count, &last_line)?;
            }
            //example on how to clone a string
            last_line = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    //DOUBLE CHECK!
    //Violates DRY principle -> Do Not Repeat Yourself
    // if count > 0 {
    //     print!("{:>4} {}", count, last_line);
    // }
    //replaced by a closure call
    print(&count, &last_line)?;

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    //match the filename
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
