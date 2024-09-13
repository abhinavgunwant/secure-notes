use serde::{ Serialize, Deserialize };

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultIndexEntry {
    id: u32,
    name: String,
    parent_folder: Option<u32>,
}

