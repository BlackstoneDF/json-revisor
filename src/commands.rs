use std::{
    ffi::OsString,
    fs::{create_dir, File},
    io::{Read, Write},
    path::PathBuf,
};

use colored::Colorize;
use json_patch::Patch;
use serde_json::{error, from_value, to_string_pretty, Value};

use crate::{error::ApplicationError, file_pair::get_file_pairs};

pub fn build(original_path: OsString, matches: OsString, result: OsString) {
    let pairs = get_file_pairs(PathBuf::from("."), original_path, matches, result).unwrap();

    for pair in pairs {
        let matching = pair.matching;
        let exists = matching.exists();
        let key = pair.key;
        let result = pair.result;

        if key.is_dir() && exists {
            continue;
        }

        if !exists {
            if key.is_dir() {
                println!(
                    "{}{}{}",
                    "Warning: Path ".yellow(),
                    key.to_string_lossy().yellow(),
                    " does not have a matching changes folder, creating folder...".yellow(),
                );
                create_dir(&matching).expect("All dirs required to be made should already be made");
                continue;
            }
            println!(
                "{}{}{}",
                "Warning: Path ".yellow(),
                key.to_string_lossy().yellow(),
                " does not have a matching changes file, creating file...".yellow(),
            );
            File::create(&matching)
                .expect("Directory creation goes first and dirs should be made first");
        }

        let original_json = {
            let mut key_file = File::open(&key).expect("File must exist");
            let mut txt = String::new();
            key_file.read_to_string(&mut txt).unwrap();
            let json: error::Result<Value> = serde_json::from_str(&txt);
            json
        };

        let changes = {
            let mut matching_file = File::open(&matching).expect("Matching file must exist");
            let mut txt = String::new();
            matching_file.read_to_string(&mut txt).unwrap();
            let json: error::Result<Value> = serde_json::from_str(&txt);
            json
        };

        if let Ok(mut original_json) = original_json {
            let patch = if let Ok(changes) = changes {
                if let Ok(patch) = from_value::<Patch>(changes) {
                    patch
                } else {
                    ApplicationError::InvalidFileFormat {
                        file_path: &matching,
                        expected: "JSON patch file",
                    }
                    .print();
                    return;
                }
            } else {
                ApplicationError::InvalidFileFormat {
                    file_path: &matching,
                    expected: "JSON file",
                }
                .print();
                return;
            };

            if let Err(_) = json_patch::patch(&mut original_json, &patch) {
                ApplicationError::PatchError {
                    target_file: &key,
                    patch_file: &matching,
                }
                .print();
                return;
            };

            let mut patched_file = File::create(result).expect("All files should have a base dir");
            if let Err(err) =
                patched_file.write_all(to_string_pretty(&original_json).unwrap().as_bytes())
            {
                ApplicationError::IoError(err.kind());
                return;
            }
        } else {
            ApplicationError::InvalidFileFormat {
                file_path: &key,
                expected: "JSON file",
            }
            .print();
        }
    }
}

pub fn update() {
    todo!()
    // Get all files in changed and loop
    // If file doesn't exist, throw a warning and create a file in changes
    // Generate a diff and write to file
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
