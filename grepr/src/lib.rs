use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::path::Path;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader},
};

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
    let mut results: Vec<MyResult<String>> = vec![];
    for path in paths {
        if path == "-" {
            results.push(Ok(path.to_string()));
            continue;
        }

        let path = Path::new(path);
        if !path.exists() {
            results.push(Err(format!("{}: No such file or directory", path.to_str().unwrap().to_string()).into()));
            continue;
        }

        if path.is_file() {
            results.push(Ok(path.to_str().unwrap().to_string()));
        } else {
            if !recursive {
                results.push(Err(format!("{} is a directory", path.to_str().unwrap().to_string()).into()));
            }
            else {
                let entries = fs::read_dir(path)
                    .unwrap()
                    .map(|res| {
                        let entry = res.unwrap();
                        entry.path()
                           .to_string_lossy()
                           .into_owned()
                    })
                    .collect::<Vec<String>>();
                
                for entry in entries {
                    results.push(Ok(entry));
                }
            }
        }
    }
    results
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn find_lines<T: BufRead>(mut file: T, pattern: &Regex, invert_match: bool) -> MyResult<Vec<String>> {
    let mut results: Vec<String> = vec![];
    for line in file.lines() {
        let line = line?;
        if (!invert_match && pattern.is_match(&line)) || (invert_match && !pattern.is_match(&line)) {
            results.push(line);
        }
    }
    Ok(results)
}


pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let matches = find_lines(
                        file,
                        &config.pattern,
                        config.invert_match,
                    )?;
                    if config.files.len() == 1 {
                        if config.count {
                            println!("{}", matches.len());
                        } else {
                            for matched_line in matches {
                                println!("{}", matched_line);
                            }
                        }
                    } else {
                        if config.count {
                            println!("{}:{}", filename, matches.len());
                        } else {
                            for matched_line in matches {
                                println!("{}:{}", filename, matched_line);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // recursive
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();

        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        let bad : String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}


