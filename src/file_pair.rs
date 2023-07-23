use std::{
    ffi::OsString,
    fs::read_dir,
    io::{self},
    path::{Component, Path, PathBuf},
    vec,
};

use thiserror::Error;

// TODO: This function is bad, it is riddled with clones and bad solutions.
// Planning to rewrite this sometime, just not now, if you have a solution,
// please let me know or shoot me a pull request
/// Recursively gets a pair of paths
/// # Example
/// ```
/// get_file_pairs(
///     Path::new("./").unwrap(),
///     OsString::from("original"),
///     OsString::from("changes")
/// )
/// ```
pub fn get_file_pairs(
    mut root: PathBuf,
    original: OsString,
    matches: OsString,
    result: OsString,
) -> io::Result<Vec<FilePathTrio>> {
    root.push(original);
    _get_file_pairs(root, matches, result)
}

fn _get_file_pairs(
    path: PathBuf,
    matches: OsString,
    results: OsString,
) -> io::Result<Vec<FilePathTrio>> {
    let current_file = {
        let matching_path =
            replace_item(&path, 1, &matches).expect("First index should always exist");
        let result_path =
            replace_item(&path, 1, &results).expect("First index should always exist");
        FilePathTrio::new(path.clone(), matching_path, result_path)
    };
    if path.is_dir() {
        let mut res: Vec<FilePathTrio> = Vec::new();
        res.push(current_file);
        for entry in read_dir(&path)? {
            let entry = entry?;
            let mut path = path.clone();
            path.push(entry.file_name());
            let mut pairs = _get_file_pairs(path, matches.clone(), results.clone())?;
            res.append(&mut pairs);
        }
        Ok(res)
    } else if path.is_file() {
        Ok::<Vec<FilePathTrio>, io::Error>(vec![current_file])
    } else {
        panic!("File somehow isn't a dir or file")
    }
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

pub struct FilePathTrio {
    pub key: PathBuf,
    pub matching: PathBuf,
    pub result: PathBuf,
}

impl FilePathTrio {
    fn new(key: PathBuf, matching: PathBuf, result: PathBuf) -> Self {
        Self {
            key,
            matching,
            result,
        }
    }
}

#[derive(Debug, Error)]
#[error("Index out of bounds")]
pub struct IndexOutOfBoundsError;
