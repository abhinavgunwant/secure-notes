use std::{ fs::{ File, create_dir_all }, io::Write, path::{ Path, PathBuf } };

use dirs_next::data_local_dir;
use serde::{ Serialize, Deserialize };
use flexbuffers::FlexbufferSerializer;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct VaultInfo {
    name: String,
    password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultIndexEntry {
    id: u32,
    name: String,
    parent_folder: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct VaultIndex {
    folders: Vec<VaultIndexEntry>,
    notes: Vec<VaultIndexEntry>,
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
pub fn create_vault(name: String, password: String, first_start: bool) -> Result<(), String> {
    println!("getting local dir");

    match get_local_dir() {
        Some(mut path) => {
            path.push("vaults");
            path.push(&name);
            println!("got local dir: {}", path.to_str().unwrap());

            if first_start {
                match create_secure_notes_directories(&path) {
                    Ok(()) => {
                        println!("Created secure notes directories on the first start");
                    }
                    Err(e) => { return Err(e); }
                }
            }

            match create_vault_info_file(&path, name.clone(), password.clone()) {
                Ok(()) => {
                    println!("Info file created");
                }
                Err(e) => {
                    return Err(format!("Could not create info file: {}", e));
                }
            }

            match create_vault_index_file(&path) {
                Ok(()) => {
                    println!("Index file created");
                }
                Err(e) => {
                    return Err(format!("Could not create index file: {}", e));
                }
            }

            match create_vault_notes_directory(&path) {
                Ok(()) => {
                    println!("vault ntoes directory created");
                }

                Err(e) => {
                    return Err(format!("Erro while creating notes directory: {}", e));
                }
            }

            let def_v_fpath = get_default_vault_file_path();

            if !def_v_fpath.is_empty() {
                println!("info file created");
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
        return match File::create(info_path) {
            Ok(mut file) => {
                let info = VaultInfo { name, password };
                let mut serializer = FlexbufferSerializer::new();

                match info.serialize(&mut serializer) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }

                match file.write(serializer.view()) {
                    Ok(_) => Ok(()),

                    Err(e) => {
                        eprintln!("{}", e);
                        Err(String::from(""))
                    }
                }
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

pub fn create_vault_notes_directory(path: &PathBuf) -> Result<(), String> {
    let mut dir_path_buf = path.clone();
    dir_path_buf.push("notes");

    match dir_path_buf.to_str() {
        Some(p) => {
            match create_dir_all(p) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("{}", e);
                    Err(String::from("Couldn't create notes directory"))
                }
            }
        }

        None => Err(String::from("Invalid path string"))
    }
}

