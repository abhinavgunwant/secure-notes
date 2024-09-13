pub mod vault;

use std::{ fs::create_dir_all, path::{ Path, PathBuf } };
use dirs_next::data_local_dir;

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

