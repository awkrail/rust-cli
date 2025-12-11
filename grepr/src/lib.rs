use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::path::Path;
use std::fs;

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
        let path = Path::new(path);
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

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    find_files(&config.files, false);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

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
}


