mod editor;

use crate::editor::Editor;

use iced::{
    Result as IcedResult, application,
};

fn main() -> IcedResult{
    application("Secure Notes", Editor::update, Editor::view)
    .run_with(Editor::new)
}

