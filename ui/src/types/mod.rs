pub mod vault_info;
pub mod vault_index;
pub mod vault_index_entry;

#[derive(Debug, PartialEq)]
pub enum VaultError {
    /// No default vault or default vault file exists
    NoDefault,

    /// The first line of default vault file is empty
    DefaultFileFirstLineEmpty,

    /// Vault was not found
    DoesNotExist,

    /// No index found for the vault
    NoIndex(String),

    /// No info file found
    NoInfo,

    /// Index for the vault is corrupt
    CorruptIndex,

    /// No local directory was found
    NoLocalDir,

    OSError(String),
}

