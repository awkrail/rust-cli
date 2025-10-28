use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn print_file(reader: Box<dyn BufRead>, number_lines: bool, number_nonblank_lines: bool) -> MyResult<()> {
    let mut line_idx = 1;
    for line in reader.lines() {
        let line = line?;
        if number_lines {
            println!("{:>6}\t{}", line_idx, line);
            line_idx += 1;
        } else if number_nonblank_lines {
            if line != "" {
                println!("{:>6}\t{}", line_idx, line);
                line_idx += 1;
            } else {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(reader) => print_file(reader, config.number_lines, config.number_nonblank_lines)?,
        }
    }
    Ok(())
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))), 
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Ken")
        .about("Rust cat")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .multiple(true)
                .default_value("-")
                .help("Input files"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .takes_value(false)
                .conflicts_with("number_nonblank")
                .help("Print line number"),
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .takes_value(false)
                .help("Print line number for non-blank lines"),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_lines: matches.is_present("number_nonblank"),
    })
}

