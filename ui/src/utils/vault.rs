///
/// Contains all the utilities related to vault.
///
/// A vault is directory a inside the "vaults" directory in the secure notes
/// local directory.
///
/// It contains:
/// - And index file named "index". It contains entries that map a note's name
///     with it's file inside the notes directory.
/// - An info file named "info". This contains all the information necessary to
///     decrypt the notes.
/// - A directory named "notes" that contains all the encrypted notes.
///
use std::{ fs::{ File, create_dir_all }, io::Write, path::PathBuf };
use serde::{ Serialize, Deserialize };
use flexbuffers::FlexbufferSerializer;

use crate::{
    types::{
        vault_index::VaultIndex, vault_index_entry::VaultIndexEntry,
        vault_info::VaultInfo
    },
    utils::{
        create_secure_notes_directories, get_local_dir,
        get_default_vault_file_path, create_default_vault_file,
    },
};

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

            match get_default_vault_file_path() {
                Some(def_v_fpath) => {
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

                None => {
                    Err(String::from("Could not find the default path"))
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

