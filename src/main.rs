use std::ffi::OsStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "lineman")]
struct LinemanArgs {
    /// The root path from which to begin processing
    #[structopt(short, long)]
    path: PathBuf,

    /// A list of file extensions that dictates which files are processed
    #[structopt(short, long)]
    extensions: Vec<String>,
}

fn main() -> Result<(), String> {
    // let untouched_files: Vec<String> = Vec::new();
    // let cleaned_files: Vec<String> = Vec::new();
    // let files_with_errors: Vec<String> = Vec::new();

    let args = LinemanArgs::from_args();

    for dir_entry_result in WalkDir::new(args.path) {
        match dir_entry_result {
            Ok(dir_entry) => {
                let path = dir_entry.path();

                if !path.is_file() {
                    continue;
                }

                if let Some(extension) = path.extension() {
                    if args
                        .extensions
                        .iter()
                        .map(|extension| OsStr::new(extension))
                        .any(|xtension| xtension == extension)
                    {
                        let path_display = path.display();

                        match clean_file(path) {
                            Ok(_) => println!("Cleaned: {}", path_display),
                            Err(_) => println!("Not cleaned: {}", path_display),
                        }
                    }
                }
            }
            Err(_) => return Err("Bad Path".to_string()),
        }
    }

    Ok(())
}

fn clean_file(path: &Path) -> Result<bool, String> {
    let cleaned_lines: Vec<String>;
    let mut file_was_cleaned: bool = false;

    {
        let file = File::open(path).map_err(|_| format!("Cannot open file {}", path.display()))?;
        let buf_reader = BufReader::new(file);

        cleaned_lines = buf_reader
            .lines()
            .map(|line_result| {
                line_result.map(|line| {
                    let (cleaned_line, line_was_cleaned) = clean_line(&line);

                    if line_was_cleaned {
                        file_was_cleaned = true
                    }

                    cleaned_line
                })
            })
            .collect::<Result<Vec<String>, _>>()
            .map_err(|_| "Can't read line".to_string())?;
    }

    let mut file = File::create(path).map_err(|_| "Cannot open file".to_string())?;

    for line in cleaned_lines {
        file.write_all(line.as_bytes()).unwrap();
    }

    Ok(file_was_cleaned)
}

fn clean_line(line: &str) -> (String, bool) {
    let cleaned_line = format!("{}\n", line.trim_end());
    let line_was_cleaned = cleaned_line == line;
    (cleaned_line, line_was_cleaned)
}

#[test]
fn clean_bad_lines() {
    let input_output_lines_array = [
        // Remove spaces
        ("some code    \n", "some code\n"),
        // Keep indentation, remove spaces
        ("    some code    \n", "    some code\n"),
        // Remove tab
        ("some code\t\n", "some code\n"),
        // Keep indentation, remove tab
        ("    some code\t\n", "    some code\n"),
        // Add newline
        ("some code", "some code\n"),
        // Remove spaces, add newline
        ("some code    ", "some code\n"),
        // Remove spaces
        ("    \n", "\n"),
        // Remove spaces, add newline
        ("    ", "\n"),
    ];

    test_runner(&input_output_lines_array);
}

#[test]
fn skip_clean_good_lines() {
    let input_output_lines_array = [("some code\n", "some code\n"), ("\n", "\n")];

    test_runner(&input_output_lines_array);
}

#[allow(dead_code)]
fn test_runner(input_output_lines_array: &[(&str, &str)]) {
    for (input, output) in input_output_lines_array {
        assert_eq!(clean_line(*input).0, *output);
    }
}

// TODO:

// Fix all bad error handling - don't use string errors and don't use unwraps - some errors might be killing the program when the program could just continue on
// Better logging - Log what has been checked, what has actually been changed, and what couldn't be changed, for whatever reason
// Show numerical stats on how many files were looked at, how many were changed, duration of run, etc
// Tweak command line argument parsing (help, info, etc)
