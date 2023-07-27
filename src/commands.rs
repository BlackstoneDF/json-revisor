use std::{
    ffi::OsString,
    fs::{create_dir, File},
    io::{Read, Write},
    path::PathBuf,
};

use colored::Colorize;
use json_patch::Patch;
use serde_json::{from_value, to_string_pretty, Value};

use crate::{
    error::{AppError, UnwrapAppPathlessError},
    file_trio::{get_file_trios, FindFileTriosError, TrioInitError},
};

/*
original - have all the files
changes - have all the files matching changes, but not required
changed - empty
*/
pub fn build(original_path: OsString, matches: OsString, result: OsString) {
    let trios = match get_file_trios(PathBuf::from("."), original_path, matches, result) {
        Ok(it) => it,
        Err(err) => match err {
            FindFileTriosError::TrioInitError(err) => match err {
                TrioInitError::InconsistentFileTypes(err) => AppError::InconsistentFileTypes(err),
                TrioInitError::IoError(err) => AppError::IoErrorPath(err),
            },
            FindFileTriosError::IoErrorWithPath(err) => AppError::IoErrorPath(err),
            FindFileTriosError::IoError(err) => AppError::IoError(err),
        }
        .throw(),
    };
    println!("{:#?}", trios);

    let mut generate_count = 0;
    for trio in trios {
        let matching = trio.changes;
        let exists = matching.exists();

        let original = trio.original;
        let result = trio.changed;

        // TODO: Finish this
        if original.is_dir() && !(matching.exists() && result.exists()) {
            continue;
        }

        if !exists {
            if original.is_dir() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    original.to_string_lossy().yellow(),
                    " does not have a matching changes folder, creating folder...".yellow(),
                );
                create_dir(&matching).expect("All dirs required to be made should already be made");
                create_dir(&result).expect("All dirs required to be made should already be made");
                continue;
            } else if original.is_file() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    original.to_string_lossy().yellow(),
                    " does not have a matching changes file, creating file...".yellow(),
                );
                let mut file = File::create(&matching)
                    .expect("Directory creation goes first and dirs should be made first");
                file.write_all(b"[]").unwrap_app_error(&matching);
            } else {
                continue;
            }
        }

        let mut original_json = json_from_path(&original);
        let changes = json_from_path(&matching);

        let res = from_value::<Patch>(changes);
        let patch = res.unwrap_or_else(|_| {
            AppError::InvalidFileFormat {
                file_path: &matching,
                expected: "JSON patch file",
            }
            .throw();
        });

        if let Err(_) = json_patch::patch(&mut original_json, &patch) {
            AppError::PatchError {
                target_file: &original,
                patch_file: &matching,
            }
            .throw();
        };

        println!("{:?}", result);
        let mut patched_file = File::create(&result).expect("All files should have a base dir");
        patched_file
            .write_all(to_string_pretty(&original_json).unwrap().as_bytes())
            .unwrap_app_error(&result);
        generate_count += 1;
    }
    let msg = format!("Successfully generated {} files", generate_count);
    println!("{}", msg.bright_green());
}

/*
original - have all the files
changes - empty
changed - have all files but changed
*/
pub fn update(original_path: OsString, matches: OsString, result: OsString) {
    // TODO: This repeats the build function which should be fixed (later)
    let trios = match get_file_trios(PathBuf::from("."), original_path, matches, result) {
        Ok(it) => it,
        Err(err) => match err {
            FindFileTriosError::TrioInitError(err) => match err {
                TrioInitError::InconsistentFileTypes(err) => AppError::InconsistentFileTypes(err),
                TrioInitError::IoError(err) => AppError::IoErrorPath(err),
            },
            FindFileTriosError::IoErrorWithPath(err) => AppError::IoErrorPath(err),
            FindFileTriosError::IoError(err) => AppError::IoError(err),
        }
        .throw(),
    };

    let mut changes_count = 0;
    for trio in trios {
        let original = trio.original;
        let changes = trio.changes;
        let changed = trio.changed;

        if original.is_dir() {
            if !changes.exists() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    original.to_string_lossy().yellow(),
                    " does not have a matching changes folder, creating folder...".yellow(),
                );
                create_dir(&changes).expect("All dirs required to be made should already be made");
            }
            continue;
        }

        if !changed.exists() {
            continue;
        }

        let original_json: Value = json_from_path(&original);
        let changed_json: Value = json_from_path(&changed);

        let diff = json_patch::diff(&original_json, &changed_json);
        let mut changes_file = File::create(&changes).unwrap_app_error(&changes);
        File::write_all(&mut changes_file, diff.to_string().as_bytes()).unwrap_app_error(&changes);

        changes_count += 1;
    }
    let msg = format!("Successfully applied {} changes", changes_count);
    println!("{}", msg.bright_green());
}

fn json_from_path(path: &PathBuf) -> Value {
    println!("{:?}", &path);
    let mut file = File::open(&path).unwrap_app_error(&path);
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap_app_error(&path);
    serde_json::from_str(&buf).unwrap_or_else(|_| {
        AppError::InvalidFileFormat {
            file_path: path,
            expected: "JSON file",
        }
        .throw();
    })
}

pub fn init() {
    // Node inspired
    todo!()
}

pub fn print_help() {
    println!(
        r#"=== Help ===
help - Brings up this help menu
build - Build the modified changes by applying the changes in the changes folder
    {0}
update - Update the changes by modifying the changes folder - 
    {0}"#,
        "WARNING: THIS PROCESS IS IRREVERSIBLE".red()
    );
}
