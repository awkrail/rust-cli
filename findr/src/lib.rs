use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if (config.entry_types.is_empty()
                        || config.entry_types.iter().any(|entry_type| {
                            match entry_type {
                                Link => entry.file_type().is_symlink(),
                                Dir => entry.file_type().is_dir(),
                                File => entry.file_type().is_file(),
                            }
                        }))
                        && (config.names.is_empty()
                        || config.names.iter().any(|re| {
                            re.is_match(&entry.file_name().to_string_lossy())
                        }))
                    {
                        println!("{}", entry.path().display());
                    }
                },
            }
        }
    }
    Ok(())
}


pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("Ken")
        .about("Rust find")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .multiple(true)
                .default_value(".")
        )
        .arg(
            Arg::with_name("names")
                .value_name("NAME")
                .short("n")
                .long("name")
                .multiple(true)
        )
        .arg(
            Arg::with_name("types")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .possible_values(&["f", "d", "l"])
                .multiple(true)
        )
        .get_matches();

    let paths = matches.values_of_lossy("paths").unwrap();

    let names = matches
        .values_of_lossy("names")
        .map(|vals| {
            vals.into_iter()
                .map(|name| {
                    Regex::new(&name)
                        .map_err(|_| format!("error: invalid value \'{}\'", name))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let entry_types = matches
        .values_of_lossy("types")
        .map(|vals| {
            vals.iter()
                .map(|val| match val.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("error: invalid type \'{}\' for \'--type [<TYPE>...]\'", val),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names,
        entry_types,
    })
}
