use serde::{ Serialize, Deserialize };

use super::vault_index_entry::VaultIndexEntry;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VaultIndex {
    /// The id of the last entry. New entry's id is `last_id + 1`.
    pub last_id: u32,
    pub entries: Vec<VaultIndexEntry>,
}

