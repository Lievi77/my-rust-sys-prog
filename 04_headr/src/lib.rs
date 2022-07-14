use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize, //pointer-sized unsigned integer
    //usize varies from 4 bytes on 32-bit OS and 8 bytes on a 64-bit
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .author("Lev")
        .about("Rust Head")
        .arg(
            Arg::new("lines")
                .help("The number of lines to print")
                .short('n')
                .long("lines")
                .value_name("LINES") //does not necessarily need a value
                .default_value("10"), //since we have a default
        )
        .arg(
            Arg::new("bytes")
                .help("The number of bytes to print")
                .value_name("BYTES")
                .short('c')
                .takes_value(true) //indicate that it actually takes a value
                .long("bytes")
                .conflicts_with("lines"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE(S)")
                .help("Input file(s)")
                .takes_value(true)
                .allow_invalid_utf8(true)
                .default_value("-")
                .min_values(1),
        )
        .get_matches();

    //parsing here takes a bit more work
    let lines = matches
        .value_of("lines")
        .map(parse_int)
        //transpose maps converts from Option to Result
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    Ok(Config {
        //get_many =
        files: matches.values_of_lossy("files").unwrap(),
        //lines and bytes take a
        lines: lines.unwrap(),
        bytes,
    })
}

// function to parse strings into positive integers
fn parse_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        //match statements can have guards, an additional check condition
        Ok(n) if n > 0 => Ok(n), //rust infers usize type from the return annotation
        _ => Err(From::from(val)), // &str does not implement Box<>
                                  // MyResult is of type Result<T, Box<dyn Errors>>
    }
}

//unit test for parse_positive_int
#[test]
fn test_parse_positive_int() {
    //3 is ok
    let res = parse_int("3");
    assert!(res.is_ok()); //check that function returns OK()
    assert_eq!(res.unwrap(), 3);

    //any string is an error
    let res = parse_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    //A zero is an error
    let res = parse_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

pub fn run(config: Config) -> MyResult<()> {
    //dbg!(config);

    // _ indicates a wildcard in a match statement
    // _ indicates a variable not being used when used as a variable name
    // _ indicates the compiler to infer the type when used in a type annotation

    let num_files = config.files.len();

    //for each filename give, try to open it
    for (file_num, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(mut file) => {
                if num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }

                if let Some(num_bytes) = config.bytes {
                    //if there are num_bytes
                    //take first num_bytes of a file
                    let mut handle = file.take(num_bytes as u64); //cast to unsigned 64 bits integer
                    let mut buffer = vec![0; num_bytes]; // creates a vector of zeroes of length num_bytes
                    let n = handle.read(&mut buffer)?; //n signalizes the number of bytes read, may be lower than the number requested
                    print!("{}", String::from_utf8_lossy(&buffer[..n]));
                } else {
                    //lines() does not append line delimiters to the buffer
                    //take(n) only grabs the first n lines of a file
                    // for line in file.lines().take(config.lines) {
                    //     println!("{}", line?);
                    // }
                    let mut line = String::new(); //string buffer
                    for _ in 0..config.lines {
                        let bytes = file.read_line(&mut line)?; //read_line appends the delimiter to the buffer
                        if bytes == 0 {
                            break; //if the bytes is 0, we are done
                        }
                        print!("{}", line);
                        line.clear(); //clear the buffer
                    }
                }
            }
        };
    }

    Ok(())
}

//function that opens a file given a filename
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        //if '-' is given, open the standard input
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        //else open the new file
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
