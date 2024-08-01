use iced::{ Task, Element, Fill, widget::{ text_editor, text_editor::Content, Column, column } };

#[derive(Debug, Default, Copy, Clone)]
pub enum EditorState {
    #[default]
    Uninitialized,
    StartupComplete,
}

#[derive(Debug, Default)]
pub struct Editor {
    pub content: Content,
}

impl Editor {
    pub fn new() -> (Self, Task<EditorState>) {
        (Editor::default(), Task::none())
    }

    pub fn update(&mut self, editor_state: EditorState) -> Task<EditorState> {
        match editor_state {
            EditorState::Uninitialized => {
                Task::none()
            }

            EditorState::StartupComplete => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<EditorState> {
        column![
            text_editor(&self.content)
                .height(Fill)
        ]
        .into()
    }
}

