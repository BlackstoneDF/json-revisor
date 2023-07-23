use std::{path::Path, io::ErrorKind};

use colored::Colorize;

pub enum ApplicationError<'a> {
    IoError(ErrorKind),
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
    pub fn print(self) {
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
    }
}
