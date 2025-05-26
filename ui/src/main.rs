mod editor;
mod utils;
mod first_start;
mod types;
mod styles;

use crate::{
    editor::Editor, first_start::FirstStart,
    utils::is_first_start,
};

use iced::{
    application, Font, Result as IcedResult
};

fn main() -> IcedResult{
    if is_first_start() {
        return application("Secure Notes", FirstStart::update, FirstStart::view)
            .run();
    }

    application("Secure Notes", Editor::update, Editor::view)
    .font(include_bytes!("../../assets/fonts/inter/Inter-VariableFont_opsz,wght.ttf").as_slice())
    .default_font(Font::with_name("Inter"))
    .subscription(Editor::subscription)
    .run()
}

