mod editor;
mod utils;
mod first_start;

use crate::{ editor::Editor, first_start::FirstStart, utils::is_first_start };

use iced::{
    Result as IcedResult, application,
};

fn main() -> IcedResult{
    if is_first_start() {
        return application("Secure Notes", FirstStart::update, FirstStart::view)
            .run();
    }

    application("Secure Notes", Editor::update, Editor::view)
    .subscription(Editor::subscription)
    .run()
}

