use serde::{ Serialize, Deserialize };

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VaultIndexEntry {
    pub id: u32,
    pub name: String,
    pub parent_folder: Option<u32>,
}

