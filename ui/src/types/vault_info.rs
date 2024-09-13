use serde::{ Serialize, Deserialize };

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultInfo {
    pub name: String,
    pub password: String,
}

