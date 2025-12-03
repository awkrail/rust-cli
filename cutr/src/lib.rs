use crate::Extract::*;
use clap::{Arg, App};
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};
use csv::{StringRecord, WriterBuilder, ReaderBuilder};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
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

                    let mut wtr = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());

                    for record in reader.records() {
                        let record = record?;
                        wtr.write_record(extract_fields(&record, field_pos))?;
                    }
                }
                Bytes(bytes_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, bytes_pos));
                    }
                }
                Chars(chars_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, chars_pos));
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: \"{}\"", input);
    input
        .starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1) // convert Ok's content into usize
                .map_err(|_| value_error()) // convert Err's content into String
        })
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars : Vec<char> = line.chars().collect();
    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes: Vec<u8> = line.bytes().collect();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)).copied())
        .collect();
    String::from_utf8_lossy(&selected).into_owned()
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .map(String::from)
        .collect()
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n+1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!("First number in range ({}) must be lower than second number ({})", n1 + 1, n2 + 1));
                    }
                    Ok(n1..n2+1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Ken")
        .about("Rust cutr")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .multiple(true)
                .default_value("-")
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("b")
                .long("bytes")
                .takes_value(true)
                .conflicts_with_all(&["chars", "fields"])
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .takes_value(true)
                .conflicts_with_all(&["bytes", "fields"])
        )
        .arg(
            Arg::with_name("delim")
                .value_name("DELIM")
                .short("d")
                .long("delim")
                .default_value(" ")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .short("f")
                .long("fields")
                .takes_value(true)
                .conflicts_with_all(&["chars", "bytes"])
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;
    let fields = matches.value_of("fields").map(parse_pos).transpose()?;

    let delim_str = matches.value_of("delim").unwrap();
    let delimiter = match delim_str.as_bytes() {
        [b] => *b,
        _ => return Err(format!("--delim \"{}\" must be a single byte", delim_str).into()),
    };

    let extract = if let Some(p) = bytes {
        Extract::Bytes(p)
    } else if let Some(p) = chars {
        Extract::Chars(p)
    } else if let Some(p) = fields {
        Extract::Fields(p)
    } else {
        return Err("the following required arguments were not provided:\n  \
        <--fields <FIELDS>|--bytes <BYTES>|--chars <CHARS>>".into());
    };

    Ok(Config {
        files,
        delimiter,
        extract,
    })
}
