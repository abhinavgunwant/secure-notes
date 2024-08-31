use std::{ fs::File, io::Write, path::{ Path, PathBuf } };

use dirs_next::data_local_dir;
use serde::{ Serialize, Deserialize };
use flexbuffers::FlexbufferSerializer;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct VaultInfo {
    name: String,
    password: String,
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

// Creates the file that holds the name of the default vault
pub fn create_default_vault_file(name: &str) -> Result<(), String> {
    let mut file_path = PathBuf::from(get_default_vault_file_path().as_str());
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
pub fn get_default_vault_file_path() -> String {
    match get_local_dir() {
        Some(mut path) => {
            path.push("default-vault");
            match path.to_str() {
                Some(path_str) => String::from(path_str),
                None => String::default(),
            }
        }

        None => String::default(),
    }
}

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

/// Creates vault
///
/// Typical vault structure:
///
/// C:\Users\<user>\AppData\Local\secure-notes\
/// + default-vault
/// + vaults\
///   + <vault-name>\
///     + info
///     + index
///     + notes\
pub fn create_vault(name: String, password: String) -> Result<(), String> {
    match get_local_dir() {
        Some(mut path) => {
            path.push("vaults");
            path.push(&name);

            match create_vault_info_file(&path, name.clone(), password.clone()) {
                Ok(()) => {}
                Err(e) => {
                    return Err(format!("Could not create info file: {}", e));
                }
            }

            match create_vault_index_file(&path) {
                Ok(()) => {}
                Err(e) => {
                    return Err(format!("Could not create index file: {}", e));
                }
            }

            let def_v_fpath = get_default_vault_file_path();

            if !def_v_fpath.is_empty() {
                // TODO: change the default to the current vault
                Ok(())
            } else {
                match create_default_vault_file(&name) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                }
            }
        }

        None => Err(String::from(
            "Could not find local directory! Please edit the config manually!"
        ))
    }
}

pub fn create_vault_info_file(path: &PathBuf, name: String, password: String) -> Result<(), String> {
    let mut info_path_buf = path.clone();
    info_path_buf.push("info");

    let info_path = if let Some(p) = info_path_buf.to_str() { p } else { "" };

    if !info_path.is_empty() {
        return match File::open(info_path) {
            Ok(mut file) => {
                let info = VaultInfo { name, password };
                let mut serializer = FlexbufferSerializer::new();

                match info.serialize(&mut serializer) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }

                file.write(serializer.view());

                Ok(())
            }

            Err(e) => {
                eprintln!("{}", e);
                Err(String::from("Couldn't open file"))
            }
        };
    }

    Err(String::from("Invalid path"))
}

pub fn create_vault_index_file(path: &PathBuf) -> Result<(), String> {
    let mut index_path_buf = path.clone();
    index_path_buf.push("index");

    let index_path = if let Some(p) = index_path_buf.to_str() { p } else { "" };

    if !index_path.is_empty() {
        return match File::create(index_path) {
            Ok(_) => Ok(()),

            Err(e) => {
                eprintln!("{}", e);
                Err(String::from("Couldn't create file"))
            }
        };
    }

    Err(String::from("Invalid path"))
}

