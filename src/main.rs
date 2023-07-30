use std::{
    env::{self, args},
    ffi::OsString,
    fs::File,
    io::Read,
    path::PathBuf,
    sync::Arc,
};

use config::ProjectConfig;
use error::{AppError, AppErrorIo};

use serde_json::from_str;

pub mod commands;
pub mod config;
pub mod error;
pub mod file_trio;

pub type ImmutableString = Arc<str>;

pub const CONFIG_FILE: &str = "project.json";

fn main() {
    // env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        commands::print_help();
        AppError::UnexpectedArgumentSize {
            expected: 1,
            received: args.len() - 1,
        }
        .throw();
    }

    match args
        .get(1)
        .expect("Args length has already been checked to be a length of 1")
        .as_str()
    {
        "init" => commands::init(),
        "init-default" => commands::init_default(),
        "build" => {
            let config = get_config();
            commands::build(
                OsString::from(config.paths.original),
                OsString::from(config.paths.changes),
                OsString::from(config.paths.revise),
            );
        }
        "update" => {
            let config = get_config();
            commands::update(
                OsString::from(config.paths.original),
                OsString::from(config.paths.changes),
                OsString::from(config.paths.revise),
            );
        }
        "help" => {
            commands::print_help();
        }
        _ => {
            commands::print_help();
            AppError::InvalidArgument {
                argument_pos: 1,
                message: "not in the list of args",
            }
            .throw();
        }
    }
}

fn get_config() -> ProjectConfig {
    let path = PathBuf::from("./project.json");
    if let Ok(mut data) = File::open(&path) {
        let mut buf = String::new();
        match data.read_to_string(&mut buf) {
            Ok(it) => it,
            Err(err) => AppError::IoErrorPath(err.attach_path(path.into())).throw(),
        };
        
        if let Ok(config) = from_str::<ProjectConfig>(&buf) {
            config
        } else {
            AppError::InvalidFileFormat {
                file_path: path.into(),
                expected: "JSON format",
            }
            .throw();
        }
    } else {
        AppError::FileNotFound {
            file_name: CONFIG_FILE,
        }
        .throw();
    }
}
