use std::{env::{args, self}, ffi::OsString, fs::File, io::Read, path::PathBuf, sync::Arc, process::exit};

use config::ProjectConfig;
use error::{ApplicationError, UnwrapAppError};

use serde_json::from_str;

pub mod commands;
pub mod config;
pub mod error;
pub mod file_pair;

pub type ImmutableString = Arc<str>;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let path = PathBuf::from("./project.json");
    let config = if let Ok(mut data) = File::open(&path) {
        let mut buf = String::new();
        data.read_to_string(&mut buf).unwrap_app_error();
        if let Ok(config) = from_str::<ProjectConfig>(&buf) {
            config
        } else {
            ApplicationError::InvalidFileFormat {
                file_path: &path,
                expected: "JSON format",
            }
            .throw();
        }
    } else {
        ApplicationError::FileNotFound {
            file_name: "project.json",
        }
        .throw();
    };

    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        commands::print_help();
        ApplicationError::UnexpectedArgumentSize {
            expected: 1,
            received: args.len() - 1,
        }
        .throw();
    }

    let original = OsString::from(config.paths.original);
    let changes = OsString::from(config.paths.changes);
    let output = OsString::from(config.paths.output);

    match args
        .get(1)
        .expect("Args length has already been checked to be a length of 1")
        .as_str()
    {
        "init" => commands::init(),
        "build" => {
            commands::build(original, changes, output);
        }
        "update" => {
            commands::update(original, changes, output);
        }
        "help" => {
            commands::print_help();
        }
        _ => {
            commands::print_help();
            ApplicationError::InvalidArgument {
                argument_pos: 1,
                message: "Not in the list of args",
            }
            .throw();
        }
    }
}
