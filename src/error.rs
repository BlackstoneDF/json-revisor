use std::{
    io::{self},
    path::Path,
    process::exit,
    rc::Rc,
};

use colored::Colorize;

use thiserror::Error;

use crate::file_trio::InconsistentFileTypes;

// Make it easy to replace when threads are maybe needed
pub type ErrorPath = Rc<Path>;

pub enum AppError<'a> {
    IoError(io::Error),
    IoErrorPath(IoErrorWithPath),
    UnexpectedArgumentSize {
        expected: usize,
        received: usize,
    },
    InvalidArgument {
        argument_pos: usize,
        message: &'a str,
    },
    InvalidFileFormat {
        file_path: ErrorPath,
        expected: &'a str,
    },
    FileNotFound {
        file_name: &'a str,
    },
    PatchError {
        target_file: ErrorPath,
        patch_file: ErrorPath,
    },
    InconsistentFileTypes(InconsistentFileTypes),
}

impl AppError<'_> {
    pub fn throw(self) -> ! {
        let message: String = match self {
            AppError::IoErrorPath(err) => err.to_string(),
            AppError::IoError(err) => err.to_string(),
            AppError::UnexpectedArgumentSize { expected, received } => format!(
                "Not enough arguments, expected: {}, got: {}",
                expected, received
            ),
            AppError::InvalidArgument {
                argument_pos,
                message,
            } => format!("Invalid argument at {}, {}", argument_pos, message),
            AppError::InvalidFileFormat {
                file_path,
                expected,
            } => format!("File {:?} is not a {}", file_path, expected),
            AppError::FileNotFound { file_name } => {
                format!("Cannot find file {}", file_name)
            }
            AppError::PatchError {
                target_file,
                patch_file,
            } => format!("Cannot patch file {:?} with {:?}", target_file, patch_file),
            AppError::InconsistentFileTypes(err) => err.to_string(),
        };
        eprintln!("{}{}", "Error: ".red(), message.red());
        // panic!();
        exit(101);
    }
}

#[derive(Debug, Error)]
#[error("{} at {:?}", error, path)]
pub struct IoErrorWithPath {
    error: io::Error,
    path: ErrorPath,
}

impl<'a> IoErrorWithPath {
    pub fn new(error: io::Error, path: ErrorPath) -> Self {
        Self { error, path }
    }
    pub fn throw(self) -> ! {
        AppError::IoErrorPath(self).throw()
    }
}

pub trait UnwrapAppPathlessError<T> {
    fn unwrap_app(self) -> T;
}

impl<T> UnwrapAppPathlessError<T> for io::Result<T> {
    fn unwrap_app(self: Result<T, io::Error>) -> T {
        match self {
            Ok(it) => it,
            Err(err) => {
                err.throw()
            }
        }
    }
}

pub trait AppErrorIo {
    fn attach_path(self, path: ErrorPath) -> IoErrorWithPath;
    fn throw(self) -> !;
}

impl AppErrorIo for io::Error {
    fn attach_path(self: io::Error, path: ErrorPath) -> IoErrorWithPath {
        return IoErrorWithPath::new(self, path);
    }
    fn throw(self: io::Error) -> ! {
        AppError::IoError(self).throw()
    }
}
