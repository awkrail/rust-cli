use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Ken")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .required(true)
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .multiple(true)
                .default_value("-")
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
        )
        .arg(
            Arg::with_name("invert-match")
                .short("v")
                .long("invert-match")
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
        )
        .get_matches();
    
    let pattern = matches.value_of_lossy("pattern").unwrap();
    let pattern = RegexBuilder::new(&pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", pattern))?;

    Ok(Config {
        pattern: pattern,
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert-match"),
    })

}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    unimplemented!();
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

    #[test]
    fn test_find_files() {
        let files = find_files(&["./test/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // recursive







    }























}


