pub mod vault;

use std::{ fs::{ create_dir_all, read_to_string, File }, path::{ Path, PathBuf }, io::Write };
use dirs_next::data_local_dir;

use crate::types::VaultError;

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

pub fn get_vault_root_dir() -> Option<PathBuf> {
    match get_local_dir() {
        Some(mut path) => {
            path.push("vaults");
            Some(path)
        }

        None => None,
    }
}

/// Gets `vault_name`'s directory where `vault_name` is the name of a vault.
pub fn get_vault_dir(vault_name: String) -> Option<PathBuf> {
    match get_vault_root_dir() {
        Some(mut path) => {
            path.push(vault_name);
            Some(path)
        }

        None => None,
    }
}

/// Gets the path of the index file of the vault.
pub fn get_vault_index_path(vault_name: &String) -> Option<PathBuf> {
    if let Some(mut path) = get_vault_dir(vault_name.clone()) {
        path.push("index");
        return Some(path);
    }

    None
}

/// Checks if vault exists.
///
/// Does this by checking if a directory with the vault name exists inside the
/// "vaults" directory in the secure-notes local directory and also checks if
/// files "index" and "info" are also present inside the vault directory.
pub fn vault_exists(name: &str) -> Result<(), VaultError> {
    if let Some(mut dir_path) = get_local_dir() {
        dir_path.push("vaults");
        dir_path.push(name);

        if !dir_path.as_path().exists() {
            return Err(VaultError::DoesNotExist);
        }

        dir_path.push("index");

        if !dir_path.as_path().exists() {
            return Err(VaultError::NoIndex(name.to_owned()));
        }

        dir_path.pop();
        dir_path.push("info");

        if !dir_path.as_path().exists() {
            return Err(VaultError::NoInfo);
        }

        return Ok(());

        // TODO: Show an error that says "vault is corrupted" to the user.
    }

    return Err(VaultError::NoLocalDir);
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
pub fn get_default_vault_name() -> Result<String, VaultError> {
    let default_file_path;

    match get_default_vault_file_path() {
        Some(f) => {
            default_file_path = f;
        }

        None => {
            return Err(VaultError::NoDefault);
        }
    }

    match read_to_string(default_file_path.clone()) {
        Ok(file_name) => {
            let vault_name_lines = file_name.split("\n").collect::<Vec<&str>>();

            if vault_name_lines.is_empty() {
                return Err(VaultError::DefaultFileFirstLineEmpty);
            }

            let vault_name = vault_name_lines[0];

            match vault_exists(vault_name) {
                Ok(_) => Ok(String::from(vault_name)),
                Err(e) => Err(e),
            }
        }

        Err(e) => {
            eprintln!("Error when getting the default vault file name: {}", e);
            Err(VaultError::OSError(e.to_string()))
        }
    }
}

