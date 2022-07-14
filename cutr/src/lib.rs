use crate::Extract::*;
use clap::{App, Arg};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<usize>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8, //delimiter is represented as a byte (8bits)
    extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Lev")
        .about("rust cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input File(s)")
                .required(true)
                .default_value("-")
                .min_values(1),
        )
        .arg(
            Arg::with_name("delimeter")
                .value_name("DELIMITER")
                .help("Field delimiter")
                .short("d")
                .long("delim")
                //a tab is the default value
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .help("Selected fields")
                .short("f")
                .long("fields")
                .conflicts_with_all(&["chars", "bytes"]),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .help("Selected bytes")
                .short("b")
                .long("bytes")
                .conflicts_with_all(&["fields", "chars"]),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .help("Selected characters")
                .short("c")
                .long("chars")
                .conflicts_with_all(&["fields", "bytes"]),
        )
        .get_matches();

    //first, get the delimeter
    let delimeter = matches.value_of("delimeter").unwrap_or("\t");
    let delim_bytes = delimeter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            delimeter
        )));
    }

    //then parse fields, chars, bytes
    let fields = matches.value_of("fields").map(parse_pos).transpose()?; //Transpose allows us to transform an Option of a result to a result of an option
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;

    let extract = if let Some(field_pos) = fields {
        Fields(field_pos)
    } else if let Some(byte_pos) = bytes {
        Bytes(byte_pos)
    } else if let Some(char_pos) = chars {
        Chars(char_pos)
    } else {
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    };

    //correct arguments at this point
    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        delimiter: delim_bytes[0],
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &config.extract {
                Fields(field_pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .has_headers(false)
                        .from_reader(file);

                    let mut wrt = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());

                    for record in reader.records() {
                        let record = record?;
                        wrt.write_record(extract_fields(&record, field_pos))?;
                    }
                }
                Bytes(byte_pos) => {
                    for line in file.lines() {
                        let line = line?;
                        println!("{}", extract_bytes(&line, &byte_pos));
                    }
                }
                Chars(char_pos) => {
                    for line in file.lines() {
                        let line = line?;
                        println!("{}", extract_chars(&line, &char_pos))
                    }
                }
            },
        }
    }
    Ok(())
}

pub fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut fields: Vec<usize> = vec![];
    //use a regular expression to parse the range {start-end}
    let range_re = Regex::new(r"(\d+)?-(\d+)?").unwrap();
    for val in range.split(",") {
        if let Some(cap) = range_re.captures(val) {
            //if we capture a value within the regex
            let n1: &usize = &cap[1].parse()?;
            let n2: &usize = &cap[2].parse()?;

            //a range is valid iff start < end
            if n1 < n2 {
                for n in *n1..=*n2 {
                    fields.push(n);
                }
            } else {
                return Err(From::from(format!(
                    "First number in range ({}) must be lower than second number ({})",
                    n1, n2
                )));
            }
        } else {
            //otherwise value is a single value, parse and push
            match val.parse() {
                Ok(n) if n > 0 => fields.push(n),
                _ => return Err(From::from(format!("illegal list value: \"{}\"", val))),
            }
        }
    }

    //substract 1 to every index to account for 0-index
    Ok(fields.into_iter().map(|i| i - 1).collect())
}

//Unit tests for parse_pos
#[cfg(test)]
mod tests {
    use super::parse_pos;
    #[test]
    fn test_parse_pos() {
        assert!(parse_pos("").is_err());
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0]);
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0, 2]);
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0, 1, 2]);
        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0, 6, 2, 3, 4]);
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[usize]) -> String {
    let chars: Vec<_> = line.chars().collect();
    char_pos.iter().filter_map(|i| chars.get(*i)).collect()
}
//unit test for extract_chars
#[test]
fn test_extract_chars() {
    assert_eq!(extract_chars("", &[0]), "".to_string());
    assert_eq!(extract_chars("ábc", &[0]), "á".to_string());
    assert_eq!(extract_chars("ábc", &[0, 2]), "ác".to_string());
    assert_eq!(extract_chars("ábc", &[0, 1, 2]), "ábc".to_string());
    assert_eq!(extract_chars("ábc", &[2, 1]), "cb".to_string());
    assert_eq!(extract_chars("ábc", &[0, 1, 4]), "áb".to_string());
}

fn extract_bytes(line: &str, byte_pos: &[usize]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<u8> = byte_pos
        .iter()
        .filter_map(|i| bytes.get(*i))
        .cloned() // cloned dereferences a Vec<&u8> to Vec<u8>
        .collect();

    String::from_utf8_lossy(&selected).into_owned()
}

fn extract_fields<'a>(record: &'a StringRecord, field_pos: &'a [usize]) -> Vec<&'a str> {
    field_pos.iter().filter_map(|i| record.get(*i)).collect()
}
