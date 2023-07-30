use std::{
    ffi::OsString,
    fs::{create_dir, File},
    io::{self, Read, Write},
    path::PathBuf,
};

use colored::Colorize;
use json_patch::Patch;
use serde_json::{from_value, to_string_pretty, Value};

use crate::{
    config::{ProjectConfig, ProjectPaths},
    error::{AppError, AppErrorIo, UnwrapAppPathlessError},
    file_trio::{get_file_trios, FilePath, FindFileTriosError, TrioInitError},
    CONFIG_FILE,
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

    let mut generate_count = 0;
    for trio in trios {
        let file_type = trio.file_type;
        let original = trio.original;
        let matching = trio.changes;
        let result = trio.changed;

        if !matching.exists() {
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
                    Err(err) => err.attach_path(matching).throw(),
                };
            }
            continue;
        }

        if !result.exists() {
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
            Err(err) => err.attach_path(result.into()).throw(),
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
            Err(err) => err.attach_path(changes.into()).throw(),
        };

        match File::write_all(&mut changes_file, diff.to_string().as_bytes()) {
            Ok(_) => (),
            Err(err) => err.attach_path(changes.into()).throw(),
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
        Err(err) => err.attach_path(path.into()).throw(),
    };
    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(err) => err.attach_path(path.into()).throw(),
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
    println!(
        "This utility walks you through in creating a project.json file by asking some questions.\nYou can use Ctrl+C to exit any time"
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let name = {
        let path = std::env::current_dir().expect("This program must be a binary");

        let default = path
            .file_name()
            .expect("Program directory name always has a name")
            .to_string_lossy()
            .to_string();

        print!("Name ({}): ", default);
        stdout.flush().unwrap_app();

        let mut name = String::new();
        stdin.read_line(&mut name).unwrap_app();
        let name = name.trim();
        if name.is_empty() {
            default
        } else {
            name.to_string()
        }
    };

    let description = {
        print!("Description: ");
        stdout.flush().unwrap_app();

        let mut description = String::new();
        stdin.read_line(&mut description).unwrap_app();
        description.trim().to_string()
    };

    let version = {
        const DEFAULT_VERSION: &str = "1.0.0";
        print!("Version ({}): ", DEFAULT_VERSION);
        stdout.flush().unwrap_app();

        let mut version = String::new();
        stdin.read_line(&mut version).unwrap_app();
        let version = version.trim();
        if version.is_empty() {
            DEFAULT_VERSION.to_string()
        } else {
            version.to_string()
        }
    };

    let authors = {
        print!("Author: ");
        stdout.flush().unwrap_app();

        let mut author = String::new();
        stdin.read_line(&mut author).unwrap_app();
        
        vec![author.trim().to_string()]
    };

    let license = {
        const DEFAULT_LICENSE: &str = "MIT";
        print!("License: ({}): ", DEFAULT_LICENSE);
        stdout.flush().unwrap_app();

        let mut license = String::new();
        stdin.read_line(&mut license).unwrap_app();
        let license = license.trim();
        if license.is_empty() {
            DEFAULT_LICENSE.to_string()
        } else {
            license.to_string()
        }
    };

    let config = ProjectConfig {
        name,
        description,
        version,
        authors,
        license,
        paths: ProjectPaths {
            original: "original".to_string(),
            changes: "changes".to_string(),
            revise: "revise".to_string(),
        },
    };

    let config =
        serde_json::to_string_pretty(&config).expect("Correct steps taken to create config");

    println!("Config: \n{}", config);
    print!("Is this OK? (yes)");
    stdout.flush().unwrap_app();

    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap_app();
    let buf = buf.trim();

    if !buf.is_empty() && buf != "y" && buf != "yes" {
        println!("Aborted");
    } else {
        let mut file = match File::create(CONFIG_FILE) {
            Ok(it) => it,
            // Unsure of a good way to construct a Rc<Path>
            Err(err) => err.attach_path(PathBuf::from(CONFIG_FILE).into()).throw(),
        };

        match file.write(config.as_bytes()) {
            Ok(_) => (),
            Err(err) => err.attach_path(PathBuf::from(CONFIG_FILE).into()).throw(),
        };
    }
}

pub fn init_default() {
    let mut file = match File::create(CONFIG_FILE) {
        Ok(it) => it,
        // Unsure of a good way to construct a Rc<Path>
        Err(err) => err.attach_path(PathBuf::from(CONFIG_FILE).into()).throw(),
    };

    match file.write(include_bytes!("static/project.json")) {
        Ok(_) => (),
        Err(err) => err.attach_path(PathBuf::from(CONFIG_FILE).into()).throw(),
    };
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
