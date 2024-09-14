pub mod vault;

use std::{ fs::{ create_dir_all, read_to_string, File }, path::{ Path, PathBuf }, io::Write };
use dirs_next::data_local_dir;

use crate::types::DefaultVaultFileError;

pub fn is_first_start() -> bool {
    match get_local_dir() {
        Some(path) => {
            match path.to_str() {
                Some(path_str) => !Path::new(path_str).exists(),
                None => true,
            }
        }

        None => true,
    }
}

pub fn get_local_dir() -> Option<PathBuf> {
    match data_local_dir() {
        Some(mut path) => {
            path.push("secure-notes");
            Some(path)
        }

        None => None,
    }
}

/// Checks if vault exists.
///
/// Does this by checking if a directory with the vault name exists inside the
/// "vaults" directory in the secure-notes local directory and also checks if
/// files "index" and "info" are also present inside the vault directory.
pub fn vault_exists(name: &str) -> bool {
    if let Some(mut dir_path) = get_local_dir() {
        dir_path.push("vaults");
        dir_path.push(name);

        if dir_path.as_path().exists() {
            dir_path.push("index");

            if dir_path.as_path().exists() {
                dir_path.pop();
                dir_path.push("info");

                return dir_path.as_path().exists();
            }

            // TODO: Show an error that says "vault is corrupted" to the user.
        }
    }

    false
}

pub fn create_secure_notes_directories(path: &PathBuf) -> Result<(), String> {
    match path.to_str() {
        Some(p) => {
            match create_dir_all(p) {
                Ok(_) => Ok(()),

                Err(e) => {
                    eprintln!("{}", e);
                    Err(String::from("Error creating the directories"))
                }
            }
        }

        None => Err(String::from("Error in path name"))
    }
}

// Creates the file that holds the name of the default vault.
//
// The file is named "default-vault" and is a text file that should contain
// only a single line containing the name of the default vault's directory
// inside the "vaults" directory.
pub fn create_default_vault_file(name: &str) -> Result<(), String> {
    let default_vault_file_path;

    match get_default_vault_file_path() {
        Some(def_path) => {
            default_vault_file_path = def_path;
        }

        None => {
            return Err(
                String::from("Could not find the for the default vault file.")
            );
        }
    }

    let mut file_path = PathBuf::from(default_vault_file_path.as_str());
    file_path.push("default-vault");

    match file_path.to_str() {
        Some(f_path) => {
            match File::open(f_path) {
                Ok(mut file) => {
                    match file.write(name.as_bytes()) {
                        Ok(b) => {
                            if b > 0 {
                                return Ok(());
                            }

                            Err(String::from("No bytes written"))
                        }

                        Err(e) => {
                            eprintln!("{}", e);

                            Err(String::from("Error while writing file"))
                        }
                    }
                }

                Err(e) => {
                    eprintln!("{}", e);

                    Err(String::from("Couldn't open file"))
                }
            }
        }

        None => {
            Err(String::from("Issues with file path"))
        }
    }
}

/// Gets the file path to the file that holds the name of the default vault
pub fn get_default_vault_file_path() -> Option<String> {
    match get_local_dir() {
        Some(mut path) => {
            path.push("default-vault");
            match path.to_str() {
                Some(path_str) => Some(String::from(path_str)),
                None => None,
            }
        }

        None => None,
    }
}

/// Gets the default vault's name from the "default-vault" file.
///
/// If there are more than one lines/entries in the "default-vault" file, only
/// the first line is considered.
///
/// For more information see: [`create_default_vault_file`].
pub fn get_default_vault_name() -> Result<String, DefaultVaultFileError> {
    let default_file_path;

    match get_default_vault_file_path() {
        Some(f) => {
            default_file_path = f;
        }

        None => {
            return Err(DefaultVaultFileError::FileDoesNotExist);
        }
    }

    match read_to_string(default_file_path.clone()) {
        Ok(file_name) => {
            let vault_name_lines = file_name.split("\n").collect::<Vec<&str>>();

            if vault_name_lines.is_empty() {
                return Err(DefaultVaultFileError::FirstLineEmpty);
            }

            let vault_name = vault_name_lines[0];

            if vault_exists(vault_name) {
                return Ok(String::from(vault_name))
            }

            Err(DefaultVaultFileError::VaultDoesNotExist)
        }

        Err(e) => {
            eprintln!("Error when getting the default vault file name: {}", e);
            Err(DefaultVaultFileError::OSError(e.to_string()))
        }
    }
}

