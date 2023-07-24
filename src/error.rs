use std::{path::Path, io::{self}, process::exit};

use colored::Colorize;

pub enum ApplicationError<'a> {
    IoError(io::Error),
    UnexpectedArgumentSize {
        expected: usize,
        received: usize,
    },
    InvalidArgument {
        argument_pos: usize,
        message: &'a str,
    },
    InvalidFileFormat {
        file_path: &'a Path,
        expected: &'a str,
    },
    FileNotFound {
        file_name: &'a str,
    },
    PatchError {
        target_file: &'a Path,
        patch_file: &'a Path,
    },
}

impl ApplicationError<'_> {
    pub fn throw(self) -> ! {
        let message: String = match self {
            ApplicationError::UnexpectedArgumentSize { expected, received } => format!(
                "Not enough arguments, expected: {}, got: {}",
                expected, received
            ),
            ApplicationError::InvalidArgument {
                argument_pos,
                message,
            } => format!("Invalid argument at {}, {}", argument_pos, message),
            ApplicationError::InvalidFileFormat {
                file_path,
                expected,
            } => format!("File {:?} is not a {}", file_path, expected),
            ApplicationError::FileNotFound { file_name } => {
                format!("Cannot find file {}", file_name)
            }
            ApplicationError::PatchError { target_file, patch_file } => format!("Cannot patch file {:?} with {:?}", target_file, patch_file),
            ApplicationError::IoError(err) => err.to_string(),
        };
        eprintln!("{}{}", "Error: ".red(), message.red());
        panic!();
        exit(101);
    }
}

pub trait UnwrapAppError<T> {
    fn unwrap_app_error(self) -> T;
}

impl<T> UnwrapAppError<T> for io::Result<T> {
    fn unwrap_app_error(self: Result<T, std::io::Error>) -> T {
        match self {
            Ok(it) => it,
            Err(err) => {
                ApplicationError::IoError(err).throw();
            }
        }
    }
}