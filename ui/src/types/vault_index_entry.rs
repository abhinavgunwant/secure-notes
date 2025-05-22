use serde::{ Serialize, Deserialize };

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum VaultIndexEntryType {
    #[default]
    Note,
    Folder,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VaultIndexEntry {
    pub id: u32,
    pub name: String,
    pub entry_type: VaultIndexEntryType,
    pub parent_folder: Option<u32>,
}

impl VaultIndexEntry {
    pub fn new_note(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            entry_type: VaultIndexEntryType::Note,
            parent_folder: None,
        }
    }

    pub fn new_folder(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            entry_type: VaultIndexEntryType::Folder,
            parent_folder: None,
        }
    }
}

