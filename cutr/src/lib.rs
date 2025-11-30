use crate::Extract::*;
use clap::{Arg, App};
use std::{error::Error, ops::Range};
use regex::Regex;

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Field(PositionList),
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
    println!("{:#?}", &config);
    Ok(())
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut ranges : PositionList = vec![];
    for part in range.split(',') {
        let re_hyphen = Regex::new(r"^([0-9]+)-([0-9]+)$").unwrap();
        let re_number = Regex::new(r"^[0-9]+$").unwrap();

        if let Some(cap) = re_hyphen.captures(part) {
            let left : i32 = cap[1].parse::<i32>().unwrap() - 1;
            let right : i32 = cap[2].parse::<i32>().unwrap();
            let left = usize::try_from(left)?;
            let right = usize::try_from(right)?;
            if left >= right {
                return Err(format!("First number in range ({}) must be lower than second number ({})", left, right).into());
            }
            let r = left..right;
            ranges.push(r);
            continue;
        }

        if let Some(cap) = re_number.find(part) {
            let n: usize = cap.as_str().parse().map_err(|_| "The number should be larger than 0")?;
            if n == 0 {
                return Err("The number should be larger than 0".into());
            }
            continue;
        }

        // illegal match
        return Err(format!("illegal list value: \"{}\"", part).into());
    }

    Ok(ranges)
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
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .takes_value(true)
                .conflicts_with("bytes")
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
                .conflicts_with("chars")
                .conflicts_with("bytes")
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
        Extract::Field(p)
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
