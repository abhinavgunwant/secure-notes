use serde::{ Deserialize, Serialize };

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Note {
    /// The id of the note
    pub id: u32,

    /// The version of the note (default: 0)
    pub version: u16,

    /// The title of the note (stored encrypted)
    pub title: String,

    /// The text of the note (stored encrypted)
    pub text: String,
}

impl Note {
    pub fn new(id: u32, title: String, text: String) -> Self {
        Self { id, version: 0, title, text }
    }
}

