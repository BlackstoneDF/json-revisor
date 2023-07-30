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
    error::{AppError, AppErrorIo},
    file_trio::{get_file_trios, FilePath, FindFileTriosError, TrioInitError},
};

/*
original - have all the files
changes - have all the files matching changes, but not required
changed - empty
*/
pub fn build(original_path: OsString, matches: OsString, result: OsString) {
    let trios = match get_file_trios(PathBuf::from("."), original_path, matches, result) {
        Ok(it) => it,
        Err(err) => {
            match err {
                FindFileTriosError::TrioInitError(err) => match err {
                    TrioInitError::InconsistentFileTypes(err) => {
                        AppError::InconsistentFileTypes(err)
                    }
                    TrioInitError::IoError(err) => AppError::IoErrorPath(err),
                },
                FindFileTriosError::IoErrorWithPath(err) => AppError::IoErrorPath(err),
                FindFileTriosError::IoError(err) => AppError::IoError(err),
            }
            .throw()
        }
    };

    let mut generate_count = 0;
    for trio in trios {

        let file_type = trio.file_type;
        let original = trio.original;
        let matching = trio.changes;
        let result = trio.changed;

        if !matching.exists()  {
            if file_type.is_dir() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    original.to_string_lossy().yellow(),
                    " does not have a matching changes folder, creating folder...".yellow(),
                );
                create_dir(&result).expect("All dirs required to be made should already be made");
            } else if file_type.is_file() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    original.to_string_lossy().yellow(),
                    " does not have a matching changes file, creating file...".yellow(),
                );
                let mut file = File::create(&matching)
                    .expect("Directory creation goes first and dirs should be made first");
                match file.write_all(b"[]") {
                    Ok(it) => it,
                    Err(err) => err.attach_message(matching).throw(),
                };
            }
            continue;
        }

        if !result.exists()  {
            if file_type.is_dir() {
                create_dir(&result).expect("All dirs required to be made should already be made");
                continue;
            }  
        }

        if file_type.is_dir() {
            continue;
        }

        let mut original_json = json_from_path(original.clone());
        let changes = json_from_path(matching.clone());

        let res = from_value::<Patch>(changes);
        let patch = match res {
            Ok(it) => it,
            Err(_err) => AppError::InvalidFileFormat {
                file_path: matching.into(),
                expected: "JSON patch file",
            }
            .throw(),
        };

        if let Err(_) = json_patch::patch(&mut original_json, &patch) {
            AppError::PatchError {
                target_file: original.into(),
                patch_file: matching.into(),
            }
            .throw();
        };

        let mut patched_file = File::create(&result).expect("All files should have a base dir");
        match patched_file.write_all(to_string_pretty(&original_json).unwrap().as_bytes()) {
            Ok(it) => it,
            Err(err) => err.attach_message(result.into()).throw(),
        }
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

        let original_json: Value = json_from_path(original);
        let changed_json: Value = json_from_path(changed);

        let diff = json_patch::diff(&original_json, &changed_json);

        let mut changes_file = match File::create(&changes) {
            Ok(it) => it,
            Err(err) => err.attach_message(changes.into()).throw(),
        };

        match File::write_all(&mut changes_file, diff.to_string().as_bytes()) {
            Ok(_) => (),
            Err(err) => err.attach_message(changes.into()).throw(),
        };

        changes_count += 1;
    }
    let msg = format!("Successfully applied {} changes", changes_count);
    println!("{}", msg.bright_green());
}

// Too lazy to return error
fn json_from_path(path: FilePath) -> Value {
    let mut file = match File::open(&path) {
        Ok(it) => it,
        Err(err) => err.attach_message(path.into()).throw(),
    };
    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(err) => err.attach_message(path.into()).throw(),
    }
    serde_json::from_str(&buf).unwrap_or_else(|_| {
        AppError::InvalidFileFormat {
            file_path: path.into(),
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
