use iced::{
    keyboard::{ on_key_press, key::{ Named, Key }, Modifiers, },
    widget::{
        column, container, pane_grid, responsive, text, text_editor, text_editor::{Action, Content},
        text_input,
    }, Element, Fill, Subscription, Center, Background, Color,
};

use crate::utils::VaultIndexEntry;

#[derive(Debug, Default, Clone)]
pub enum EditorMessage {
    #[default]
    Uninitialized,
    /// Action performed on the text editor
    ActionPerformed(Action),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    CloseFocused,
    ToggleExplorer,
    Clicked(pane_grid::Pane),
    Save,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    Explorer,
    TextEditor,
}

#[derive(Clone, Copy)]
pub struct Pane {
    pub id: usize,
    pub pane_type: PaneType,
    pub is_pinned: bool,
}

pub struct Editor {
    pub opened_file: Option<String>,
    pub explorer_files: Vec<VaultIndexEntry>,
    pub content: Content,
    pub panes: pane_grid::State<Pane>,
    pub panes_created: usize,
    pub focused_pane: Option<pane_grid::Pane>,
    pub show_explorer: bool,
    pub initialized: bool,
}

impl Pane {
    pub fn new(id: usize, pane_type: PaneType) -> Self {
        Self { id, is_pinned: false, pane_type }
    }
}

impl Editor {
    // pub fn new() -> (Self, Task<EditorMessage>) {
    pub fn new() -> Self {
        // (Editor::default(), Task::none())
        let ( mut pane_state, explorer_pane ) = pane_grid::State::new(
            Pane::new(0, PaneType::Explorer)
        );

        match pane_state.split(
            pane_grid::Axis::Vertical,
            explorer_pane,
            Pane::new(0, PaneType::TextEditor)
        ) {
            Some((_pane, split)) => {
                pane_state.resize(split, 0.25);
            }

            None => {}
        }

        Self {
            opened_file: None,
            explorer_files: vec![],
            content: Content::default(),
            panes: pane_state,
            panes_created: 0,
            focused_pane: None,
            show_explorer: true,
            initialized: false,
        }
    }

    // pub fn update(&mut self, editor_state: EditorMessage) -> Task<EditorMessage> {
    pub fn update(&mut self, editor_state: EditorMessage) {
        match editor_state {
            EditorMessage::ActionPerformed(action) => {
                self.content.perform(action);
            }

            // EditorMessage::Split(axis, pane) => {
            //     let result = self.panes.split(axis, pane, Pane::new(self.panes_created));

            //     if let Some((pane, _)) = result {
            //         self.focus = Some(pane);
            //         self.panes_created += 1;
            //     }
            // }

            EditorMessage::Resized(pane_grid::ResizeEvent{ split, ratio }) => {
                self.panes.resize(split, ratio);
            }

            EditorMessage::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(pane) {
                    self.focused_pane = Some(sibling);
                }
            }

            EditorMessage::CloseFocused => {
                if let Some(pane) = self.focused_pane {
                    if let Some(Pane { is_pinned, ..}) = self.panes.get(pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(pane) {
                                self.focused_pane = Some(sibling);
                            }
                        }
                    }
                }
            }

            EditorMessage::Clicked(pane) => {
                self.focused_pane = Some(pane);
            }

            EditorMessage::ToggleExplorer => {
                if self.show_explorer {
                    println!("hiding explorer");
                    let panes = self.panes.clone();

                    for (pane, state) in panes.iter() {
                        if state.pane_type == PaneType::Explorer {
                            if let Some((_, sibling)) = self.panes.close(*pane) {
                                self.focused_pane = Some(sibling);
                            }
                        }
                    }
                } else {
                    println!("showing explorer");
                }

                self.show_explorer = !self.show_explorer;

                // if let Some(pane) = self.focused_pane {
                //     println!("1");
                //     let result = self.panes.split(
                //         pane_grid::Axis::Vertical,
                //         pane,
                //         Pane::new(self.panes_created, PaneType::Explorer)
                //     );

                //     if let Some((new_pane, _)) = result {
                //         println!("2");
                //         self.focused_pane = Some(new_pane);
                //         self.panes_created += 1;
                //      }
                // }
            }

            EditorMessage::Uninitialized => {}

            EditorMessage::Save => {
                // let text = self.content.text();
            }
        }
    }

    pub fn view(&self) -> Element<EditorMessage> {
        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_id, pane, _is_maximized|{
            // let is_focused = self.focused_pane == Some(id);

            let mut pane_grid_content = pane_grid::Content::new(responsive(move |_size|{
                if pane.pane_type == PaneType::TextEditor {
                    let _style = container::Style {
                        background: Some(Background::Color(Color {
                            r: 0.054,
                            g: 0.1,
                            b: 0.14,
                            a: 1.0,
                        })),
                        ..container::Style::default()
                    };

                    if self.opened_file == None {
                        container(column![
                            text("Select a file from the explorer on the left to view/edit!")
                                .align_x(Center)
                                .align_y(Center)
                                .width(Fill)
                                .height(Fill)
                                .center()
                        ])
                            .style(move |_| _style)
                            .into()
                    } else {
                        container(column![
                            text_input("test", "text"),
                            text_editor(&self.content)
                                .on_action(EditorMessage::ActionPerformed)
                                .height(Fill)
                        ])
                            .style(move |_| _style)
                            .into()
                    }
                } else {
                    let style = container::Style {
                        background: Some(Background::Color(Color {
                            r: 0.05,
                            g: 0.09,
                            b: 0.11,
                            a: 1.0
                        })),
                        ..container::Style::default()
                    };

                    if self.explorer_files.is_empty() {
                        container(text!("This shows the notes here..."))
                            .style(move |_| style)
                            .height(Fill)
                            .width(Fill)
                            .align_x(Center)
                            .align_y(Center)
                            .into()
                    } else {
                        container(text!("WIP"))
                            .style(move |_| style)
                            .into()
                    }
                }
            }));

            if pane.pane_type == PaneType::Explorer {
                pane_grid_content = pane_grid_content.title_bar(
                    pane_grid::TitleBar::new(text!("{}", "Explorer"))
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color{
                                r: 0.04,
                                g: 0.05,
                                b: 0.07,
                                a: 1.0,
                            })),
                            ..container::Style::default()
                        })
                );
            }

            return pane_grid_content;
        })
        .width(Fill)
        .height(Fill)
        .on_click(EditorMessage::Clicked)
        .on_resize(10, EditorMessage::Resized);

        container(pane_grid)
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<EditorMessage> {
        on_key_press(|key_code, modifiers|{
            match (modifiers, key_code.as_ref()) {
                (Modifiers::CTRL, Key::Character("s")) => {
                    // TODO: Save the file
                    return None;
                }

                (Modifiers::CTRL, Key::Character("e")) =>
                    Some(EditorMessage::ToggleExplorer),

                _ => None
            }
        })
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

