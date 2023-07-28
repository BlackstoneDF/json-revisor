use std::{
    ffi::OsString,
    fs::{read_dir, FileType},
    io::{self},
    path::{Component, Path, PathBuf},
    sync::Arc,
    vec,
};

use thiserror::Error;

use crate::error::{AddMessage, ErrorPath, IoErrorWithPath};

// TODO: This function is bad, it is riddled with clones and bad solutions
// AND error handling that looks like it has been written by a person who smokes crack
//
// Planning to rewrite this sometime, just not now, if you have a solution,
// please let me know or shoot me a pull request
/// Recursively gets a pair of paths
pub fn get_file_trios(
    mut root: PathBuf,
    original: OsString,
    matches: OsString,
    result: OsString,
) -> Result<Vec<FilePathTrio<'static>>, FindFileTriosError> {
    root.push(original);
    _get_file_trios(root, matches, result)
}

fn _get_file_trios(
    path: PathBuf,
    matches: OsString,
    results: OsString,
) -> Result<Vec<FilePathTrio<'static>>, FindFileTriosError> {
    let current_file = {
        let matching_path =
            replace_item(&path, 1, &matches).expect("First index should always exist");
        let result_path =
            replace_item(&path, 1, &results).expect("First index should always exist");
        FilePathTrio::new(&path, &matching_path, &result_path)
    }
    .map_err(|err| FindFileTriosError::TrioInitError(err))?;

    if path.is_dir() {
        let mut res: Vec<FilePathTrio> = Vec::new();
        res.push(current_file);
        let read_dir = match read_dir(&path) {
            Ok(it) => it,
            Err(err) => {
                return Err(FindFileTriosError::IoErrorWithPath(
                    err.attach_message(path.into()),
                ))
            }
        };
        for entry in read_dir {
            let entry = entry.map_err(|err| {
                FindFileTriosError::IoError(err) // Should never happen?
            })?;
            let mut path = path.clone();
            path.push(entry.file_name());
            let mut pairs = _get_file_trios(path, matches.clone(), results.clone())?;
            res.append(&mut pairs);
        }
        Ok(res)
    } else if path.is_file() {
        Ok::<Vec<FilePathTrio>, FindFileTriosError>(vec![current_file])
    } else {
        panic!("File somehow isn't a dir or file")
    }
}

pub enum FindFileTriosError {
    TrioInitError(TrioInitError),
    IoErrorWithPath(IoErrorWithPath),
    IoError(io::Error),
}

pub fn replace_item(
    path: &Path,
    index: usize,
    value: &OsString,
) -> Result<PathBuf, IndexOutOfBoundsError> {
    let mut vec: Vec<_> = path.components().collect();
    if vec.len() < (index + 1) {
        return Err(IndexOutOfBoundsError);
    }
    vec[index] = Component::Normal(value);
    Ok(vec.iter().collect())
}

#[derive(Debug)]
pub struct FilePathTrio<'a> {
    pub file_type: FileType,
    pub original: &'a Path,
    pub changes: &'a Path,
    pub changed: &'a Path,
}

impl<'a> FilePathTrio<'a> {
    fn new(
        original: &'a Path,
        matching: &'a Path,
        result: &'a Path,
    ) -> Result<Self, TrioInitError> {
        let file_type = original
            .metadata()
            .map_err(|err: io::Error| TrioInitError::IoError(err.attach_message(original.into())))?
            .file_type();

        let matching_type = matching
            .metadata()
            .map_err(|err| TrioInitError::IoError(err.attach_message(matching.into())))?
            .file_type();

        if file_type != matching_type {
            return Err(TrioInitError::InconsistentFileTypes(
                InconsistentFileTypes {
                    file_type_a: file_type,
                    path_a: original.into(),
                    file_type_b: matching_type,
                    path_b: matching.into(),
                },
            ));
        }

        let result_type = result
            .metadata()
            .map_err(|err| TrioInitError::IoError(err.attach_message(result.into())))?
            .file_type();

        if file_type != result_type {
            return Err(TrioInitError::InconsistentFileTypes(
                InconsistentFileTypes {
                    file_type_a: file_type,
                    path_a: original.into(),
                    file_type_b: result_type,
                    path_b: result.into(),
                },
            ));
        }

        Ok(Self {
            file_type,
            original,
            changes: matching,
            changed: result,
        })
    }
}

pub enum TrioInitError {
    InconsistentFileTypes(InconsistentFileTypes),
    IoError(IoErrorWithPath),
}

#[derive(Debug, Error)]
#[error(
    "File {:?}'s type {:?} is not consistent with {:?}'s file type {:?}",
    file_type_b,
    path_b,
    file_type_a,
    path_a
)]
pub struct InconsistentFileTypes {
    file_type_a: FileType,
    path_a: ErrorPath,
    file_type_b: FileType,
    path_b: ErrorPath,
}

#[derive(Debug, Error)]
#[error("Index out of bounds")]
pub struct IndexOutOfBoundsError;
