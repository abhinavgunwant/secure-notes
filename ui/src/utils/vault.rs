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
use std::{
    fs::{ File, create_dir_all, read, write }, io::Write, path::{ Path, PathBuf },
};
use serde::{ Serialize, Deserialize };
use flexbuffers::{ FlexbufferSerializer, Reader };
use argon2:: {
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2, Algorithm, Version, Params,
};

use crate::{
    types::{ note::Note, vault_index::VaultIndex, vault_info::VaultInfo, VaultError },
    utils::{
        create_default_vault_file, create_secure_notes_directories,
        get_default_vault_file_path, get_local_dir, get_vault_dir,
        get_vault_index_path, vault_exists,
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
                match generate_password_hash(&password) {
                    Ok(pwd) => {
                        let info = VaultInfo { name, password: pwd };
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

                    Err(e) => Err(e),
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

fn generate_password_hash(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => {
            eprintln!("Error while generating password hash: {}", e);

            Err(String::from("Some error occured while storing password."))
        }
    }
}

fn get_argon<'a>() -> Argon2<'a> {
    Argon2::new(
        Algorithm::Argon2id,// Algorithm: Argon2id
        Version::V0x13,     // Version: 19
        Params::new(
            16384,      // m = 16MB
            8,          // t = 2
            1,          // p = 1
            Some(64)    // Output size in bytes
        ).unwrap()
    )
}

/// Authenticates access to the vault by verifying the password
pub fn authenticate_vault(name: &str, password: &str) -> bool {
    if let Err(e) = vault_exists(name) {
        match e {
            VaultError::NoIndex(_) => { }
            _ => { return false; }
        }
    }

    match get_local_dir() {
        Some(mut local_dir) => {
            local_dir.push("vaults");
            local_dir.push(name);
            local_dir.push("info");

            let info_file_path;

            if let Some(path) = local_dir.as_path().to_str() {
                info_file_path = path;
            } else {
                return false;
            }

            match read(info_file_path) {
                Ok(bytes) => {
                    match Reader::get_root(bytes.as_slice()) {
                        Ok(reader) => {
                            match VaultInfo::deserialize(reader) {
                                Ok(vault_info) => {
                                    match PasswordHash::new(&vault_info.password) {
                                        Ok(parsed_hash) => {
                                            return get_argon()
                                                .verify_password(
                                                    password.as_bytes(),
                                                    &parsed_hash
                                                )
                                                .is_ok();
                                        }

                                        Err(e) => {
                                            eprintln!("Error when verifying password: {}", e);
                                        }
                                    }
                                }

                                Err(e) => {
                                    eprintln!("Error when de-serialising info file: {}", e);
                                }
                            }
                        }

                        Err(e) => {
                            eprintln!("Error when getting de-serializer: {}", e);
                        }
                    }
                }

                Err(e) => {
                    eprintln!("Error when reading info file: {}", e);
                }
            }
        }

        None => {}
    }

    return false;
}

fn get_relative_note_path(note_id: u32) -> Vec<String> {
    let mut path_str_vec: Vec<String> = Vec::with_capacity(4);

    let raw_str = format!("{:08x}", note_id);
    let mut str_chars = raw_str.chars();

    for _ in 0..4 {
        let mut s = String::with_capacity(2);

        if let Some(c) = str_chars.next() { s.push(c); }
        if let Some(c) = str_chars.next() { s.push(c); }

        path_str_vec.push(s);
    }

    path_str_vec
}

fn get_note_path(vault_name: &String, note_id: u32) -> Result<String, std::io::Error>{
    match get_vault_dir(vault_name.clone()) {
        Some(mut path) => {
            path.push("notes");

            println!("got id: {}", note_id);

            for p in get_relative_note_path(note_id).iter() {
                path.push(p);
            }

            println!("note path: {:?}", path);

            if let Some(file_path) = path.to_str() {
                return Ok(file_path.to_owned());
            }

            Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
        }

        None => Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }
}

pub fn save_note_to_vault(
    vault_name: &String,
    note: &Note,
) -> Result<(), std::io::Error> {
    match get_note_path(vault_name, note.id) {
        Ok(path) => {
            let mut p = PathBuf::from(&path);
            p.pop();

            if !p.exists() {
                match create_dir_all(p) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error creating dirs: {}", e);
                        return Err(e);
                    }
                }
            }
            
            let mut serializer = FlexbufferSerializer::new();

            if let Err(e) = note.serialize(&mut serializer) {
                eprintln!("{}", e);
                return Err(std::io::Error::from(std::io::ErrorKind::Other));
            }

            match write(path.as_str(), serializer.view()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }

        Err(e) => Err(e),
    }
}

pub fn open_note(vault_name: &String, id: u32) -> Result<Note, std::io::Error> {
    if let Some(mut path) = get_vault_dir(vault_name.clone()) {
        path.push("notes");

        println!("got id: {}", id);

        for p in get_relative_note_path(id).iter() {
            path.push(p);
        }

        println!("note path: {:?}", path);

        if let Some(file_path) = path.to_str() {
            match read(file_path) {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
                    }

                    if let Ok(reader) = Reader::get_root(bytes.as_slice()) {
                        if let Ok(note) = Note::deserialize(reader) {
                            return Ok(note);
                        }
                    }

                    return Err(std::io::Error::from(std::io::ErrorKind::Other));
                }

                Err(e) => {
                    eprintln!("Error reading note: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

/// Reads the vault index file, and returns the deserialized object
pub fn get_vault_index(vault_name: &String) -> Result<VaultIndex, std::io::Error> {
    if let Some(path) = get_vault_index_path(vault_name) {
        if let Some(file_path) = path.to_str() {
            match read(file_path) {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        println!("Vault index is empty!");
                        return Ok(VaultIndex::default());
                    }

                    if let Ok(reader) = Reader::get_root(bytes.as_slice()) {
                        if let Ok(vault_index) = VaultIndex::deserialize(reader) {
                            return Ok(vault_index);
                        }
                    }

                    println!("Other Index error");

                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other, "Other Error"
                    ));
                }

                Err(e) => {
                    eprintln!("Other Index error: {}", e);
                    return Err(e);
                }
            };
        }

        println!("Invalid Index data");
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }

    println!("Index not found");

    Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

/// Serializes `vault_index` into vault index file.
pub fn set_vault_index(
    vault_name: &String,
    vault_index: &VaultIndex,
) -> Result<(), std::io::Error> {
    if let Some(path) = get_vault_index_path(&vault_name) {
        let mut serializer = FlexbufferSerializer::new();

        match vault_index.serialize(&mut serializer) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        match write(path, serializer.view()) {
            Ok(_) => {
                println!("Index written");
                return Ok(());
            }

            Err(e) => {
                eprintln!("{}", e);
                return Err(e);
            }
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

/// Builds entire vault index from scratch
///
/// TODO: Implement it.
/// For now it just builds an empty index
pub fn build_vault_index(vault_name: &String) -> Result<(), std::io::Error> {
    if let Some(_path) = get_vault_index_path(vault_name) {
        return set_vault_index(vault_name, &VaultIndex::default());
    }

    Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

