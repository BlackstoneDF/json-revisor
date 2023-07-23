use std::{env::args, ffi::OsString, fs::File, io::Read, path::PathBuf, sync::Arc};

use config::ProjectConfig;
use error::ApplicationError;

use serde_json::from_str;

pub mod commands;
pub mod config;
pub mod error;
pub mod file_pair;

pub type ImmutableString = Arc<str>;

fn main() {
    let path = PathBuf::from("./project.json");
    let config = if let Ok(mut data) = File::open(&path) {
        let mut buf = String::new();
        data.read_to_string(&mut buf).unwrap();
        if let Ok(config) = from_str::<ProjectConfig>(&buf) {
            config
        } else {
            ApplicationError::InvalidFileFormat {
                file_path: &path,
                expected: "JSON format",
            }
            .print();
            return;
        }
    } else {
        ApplicationError::FileNotFound {
            file_name: "project.json",
        }
        .print();
        return;
    };

    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        ApplicationError::UnexpectedArgumentSize {
            expected: 1,
            received: args.len(),
        }
        .print();
        commands::print_help();
        return;
    }

    let original = OsString::from(config.paths.original);
    let changes = OsString::from(config.paths.changes);
    let output = OsString::from(config.paths.output);

    match args
        .get(1)
        .expect("Args length has already been checked to be a length of 1")
        .as_str()
    {
        "init" => {
            commands::init()
        }
        "build" => {
            commands::build(original, changes, output);
        }
        "update" => {
            commands::update();
        }
        "help" => {
            commands::print_help();
        }
        _ => {
            eprintln!("Error: Invalid arguments!");
            commands::print_help();
        }
    }
}
