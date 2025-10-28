use std::error::Error;
use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        println!("{}", filename);
    }
    Ok(())
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

