use serde::{ Serialize, Deserialize };

use super::vault_index_entry::VaultIndexEntry;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultIndex {
    folders: Vec<VaultIndexEntry>,
    notes: Vec<VaultIndexEntry>,
}

