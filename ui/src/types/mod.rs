pub mod vault_info;
pub mod vault_index;
pub mod vault_index_entry;

pub enum DefaultVaultFileError {
    FileDoesNotExist,
    FirstLineEmpty,
    VaultDoesNotExist,
    OSError(String),
}

