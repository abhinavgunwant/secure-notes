use iced::{ widget::{ column, text_editor, text_editor::{ Action, Content } }, Element, Fill, Task };

#[derive(Debug, Default, Clone)]
pub enum EditorState {
    #[default]
    Uninitialized,
    ActionPerformed(Action),
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

            EditorState::ActionPerformed(action) => {
                self.content.perform(action);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<EditorState> {
        column![
            text_editor(&self.content)
                .on_action(EditorState::ActionPerformed)
                .height(Fill)
        ]
        .into()
    }
}

