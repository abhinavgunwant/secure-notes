use std::{ thread, sync::Arc };
use futures::executor;

use iced::{
    futures::{
        channel::{ mpsc, mpsc::Sender },
        Stream, SinkExt
    },
    keyboard::{ key::Key, Event::KeyPressed },
    widget::{
        button, column, row, container, pane_grid, responsive, text, text_editor,
        text_editor::{Action, Content}, text_input, Space, svg,
    },
    event::{ self, Event },
    Background, Center, Color, Element, Fill, Subscription, stream::channel,
    Task,
};

use crate::{
    types::{ vault_index_entry::VaultIndexEntry, DefaultVaultFileError },
    utils::{get_default_vault_name, vault::authenticate_vault},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum EditorVaultPasswordStatus {
    /// When password has not been entered
    #[default]
    NONE,

    /// When password field was empty when submit button was pressed
    Empty,

    /// When password does not match
    DoesNotMatch,

    /// When password authentication is in progress
    Loading,

    /// When password is authenticated
    Authenticated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorScreen {
    /// Shows the password prompt to user to open the default vault
    PasswordPrompt,

    /// Shows the prompt to select the vault the user wants to log into
    VaultSelectionPrompt,

    /// New session, shows a blank right hand side
    Editor,
}

#[derive(Debug, Default, Clone)]
pub enum EditorMessage {
    #[default]
    None,
    Event(Event),
    // Action performed on the text editor
    ActionPerformed(Action),
    Resized(pane_grid::ResizeEvent),
    // Close(pane_grid::Pane),
    // CloseFocused,
    ToggleExplorer,
    Clicked(pane_grid::Pane),

    // Messages related to vault creation
    VaultPasswordChanged(String),
    VaultPasswordSubmitted,

    // Messages related to notes
    EditNoteName(bool),
    NoteNameChanged(String),
    SaveNoteName,
    // Save,
    New,

    // Messages related to password validation
    PVVaultEmpty,
    PVPasswordEmpty,
    PVVaultAndPasswordEmpty,
    PVDoesNotMatch,
    // PVLoading,
    PVAuthenticated,
    PVInitSender(Arc<thread::JoinHandle<()>>, Sender<(String, String)>),
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
    pub vault_password: String,
    pub vault_password_status: EditorVaultPasswordStatus,
    pub screen: EditorScreen,

    /// When name is being edited
    pub edit_name: bool,
    pub temp_note_name: String,
    pub opened_vault: Option<String>,
    pub opened_file: Option<VaultIndexEntry>,
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

        let screen;
        let opened_vault;

        match get_default_vault_name() {
            Ok(vault_name) => {
                screen = EditorScreen::PasswordPrompt;
                opened_vault= Some(vault_name.clone());

                println!("default vault: {}", vault_name);
            }

            Err(e) => {
                match e {
                    DefaultVaultFileError::OSError(_s) => {
                        // TODO: show an error message to the user
                    }

                    DefaultVaultFileError::FirstLineEmpty => {
                        eprintln!("First line empty!");
                    }

                    DefaultVaultFileError::VaultDoesNotExist => {
                        eprintln!("Vault does not exist!");
                    }

                    DefaultVaultFileError::FileDoesNotExist => {
                        eprintln!("Default vault file does not exist!");
                    }
                }

                screen = EditorScreen::VaultSelectionPrompt;
                opened_vault = None;
            }
        }

        Self {
            vault_password: String::default(),
            vault_password_status: EditorVaultPasswordStatus::default(),
            screen,
            edit_name: false,
            temp_note_name: String::default(),
            opened_vault,
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
    pub fn update(&mut self, editor_state: EditorMessage) -> Task<EditorMessage> {
        match editor_state {
            EditorMessage::Event(event) => {
                match event {
                    Event::Keyboard(KeyPressed {
                        key, modifiers, ..
                    }) => {
                        if modifiers.control() {
                            match key {
                                Key::Character(k) => {
                                    match k.as_str() {
                                        "s" => {
                                        }

                                        "e" => {
                                            return Task::done(EditorMessage::ToggleExplorer);
                                        }

                                        "n" => {
                                            return Task::done(EditorMessage::New);
                                        }

                                        _ => {}
                                    }
                                }

                                _ => {}
                            }
                        }
                    }

                    _ => {}
                }
            }

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

            // EditorMessage::Close(pane) => {
            //     if let Some((_, sibling)) = self.panes.close(pane) {
            //         self.focused_pane = Some(sibling);
            //     }
            // }

            // EditorMessage::CloseFocused => {
            //     if let Some(pane) = self.focused_pane {
            //         if let Some(Pane { is_pinned, ..}) = self.panes.get(pane) {
            //             if !is_pinned {
            //                 if let Some((_, sibling)) = self.panes.close(pane) {
            //                     self.focused_pane = Some(sibling);
            //                 }
            //             }
            //         }
            //     }
            // }

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

            EditorMessage::None => {}

            EditorMessage::VaultPasswordChanged(updated_password) => {
                self.vault_password = updated_password;
                self.vault_password_status = EditorVaultPasswordStatus::NONE;
            }

            EditorMessage::VaultPasswordSubmitted => {
                self.vault_password_status = EditorVaultPasswordStatus::Loading;
            }

            EditorMessage::EditNoteName(should_edit) => {
                self.edit_name = should_edit;
                if should_edit {
                    if let Some(note_name) = &self.opened_file {
                        self.temp_note_name = note_name.name.clone();
                    }
                }
            }

            EditorMessage::NoteNameChanged(note_name) => {
                self.temp_note_name = note_name;
            }

            EditorMessage::SaveNoteName => {
                match self.opened_file.clone() {
                    Some(file_index_entry) => {
                        let new_index_entry = VaultIndexEntry {
                            id: file_index_entry.id,
                            name: self.temp_note_name.clone(),
                            parent_folder: file_index_entry.parent_folder,
                        };

                        self.opened_file = Some(new_index_entry);
                        self.edit_name = false;
                    }

                    None => {}
                }
            }

            // EditorMessage::Save => {
            //     // let text = self.content.text();
            // }

            EditorMessage::New => {
                self.opened_file = Some(VaultIndexEntry {
                    id: 0,
                    name: String::from("Untitled Note"),
                    parent_folder: None,
                });
            }

            // EditorMessage::PVLoading => {
            //     println!("Password Validation loading");
            // }

            EditorMessage::PVVaultEmpty => {
                println!("Password Validation: vault empty");
            }

            EditorMessage::PVDoesNotMatch => {
                println!("Password Validation: password does not match");
                self.vault_password_status
                    = EditorVaultPasswordStatus::DoesNotMatch;
            }

            EditorMessage::PVPasswordEmpty => {
                self.vault_password_status = EditorVaultPasswordStatus::Empty;
                println!("Password Validation password field is empty");
            }

            EditorMessage::PVAuthenticated => {
                println!("Password Validation authenticated");
                self.screen = EditorScreen::Editor;
                self.vault_password_status
                    = EditorVaultPasswordStatus::Authenticated;
            }

            EditorMessage::PVVaultAndPasswordEmpty => {
                println!("Password Validation vault name and password are empty");
            }

            EditorMessage::PVInitSender(t_handle, mut sender) => {
                match self.opened_vault.clone() {
                    Some(vault_name) => {
                        let password = self.vault_password.clone();
                        println!(
                            "Got the sender from the worker, vault_name: {}, vault_password: {}",
                            vault_name,
                            password,
                        );

                        async fn send_values(
                            sender: &mut Sender<(String, String)>,
                            vault_name: String, password: String
                        ) {
                            let _ = sender.send((vault_name, password)).await;
                        }

                        let thread_handle = thread::spawn(move || {
                            executor::block_on(send_values(&mut sender, vault_name, password));
                            println!("** Thread 2: Sending the vault name and password");
                        });

                        let _ = thread_handle.join();

                        t_handle.thread().unpark();
                    }

                    None => {}
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<EditorMessage> {
        let vault_name;

        match self.opened_vault.clone() {
            Some(v_name) => { vault_name = v_name; }
            None => { vault_name = String::default(); }
        }

        let style = container::Style {
            background: Some(Background::Color(Color {
                r: 0.05,
                g: 0.09,
                b: 0.11,
                a: 1.0
            })),
            ..container::Style::default()
        };

        match self.screen {
            EditorScreen::Editor => {
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
                                        // .align_y(Center)
                                        .width(Fill)
                                        // .height(Fill)
                                        .center(),
                                    text("Ctrl + N for new note")
                                        .align_x(Center)
                                        // .align_y(Center)
                                        .width(Fill)
                                        .center(),
                                ])
                                    .style(move |_| _style)
                                    .width(Fill)
                                    .height(Fill)
                                    .align_x(Center)
                                    .align_y(Center)
                                    .into()
                            } else {
                                let opened_file;

                                if let Some(o_file) = &self.opened_file {
                                    opened_file = o_file.name.as_str();
                                } else {
                                    opened_file = ""; 
                                }

                                let mut ui_column = column![];

                                if self.edit_name {
                                    ui_column = ui_column.push(row![
                                        text_input("", self.temp_note_name.as_str())
                                            .size(18)
                                            .on_input(
                                                EditorMessage::NoteNameChanged
                                            )
                                            .on_submit(
                                                EditorMessage::SaveNoteName
                                            ),
                                        Space::new(4, 0),
                                        button(
                                            svg("./assets/check-line.svg")
                                                .width(16)
                                                .height(22)
                                        )
                                            .on_press(EditorMessage::SaveNoteName),
                                        Space::new(4, 0),
                                        button(
                                            svg("./assets/close-line.svg")
                                                .width(16)
                                                .height(22)
                                        )
                                            .on_press(
                                                EditorMessage::EditNoteName(
                                                    false
                                            ))
                                    ]);
                                } else {
                                    ui_column = ui_column.push(column![
                                        Space::new(0, 2),
                                        row![
                                            Space::new(5, 0),
                                            column![
                                                Space::new(0, 3),
                                                text(opened_file)
                                                    .size(18),
                                            ],
                                            Space::new(16, 0),
                                            button(
                                                svg("./assets/pencil-fill.svg")
                                                    .width(16)
                                                    .height(18)
                                            )
                                                .on_press(
                                                    EditorMessage::EditNoteName(
                                                        !self.edit_name
                                                    )
                                                )
                                        ],
                                        Space::new(0, 2),
                                    ]);
                                }

                                ui_column = ui_column.push(
                                    text_editor(&self.content)
                                        .on_action(EditorMessage::ActionPerformed)
                                        .height(Fill)
                                );

                                container(ui_column)
                                    .style(move |_| _style)
                                    .into()
                            }
                        } else {
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
                            pane_grid::TitleBar::new(text!("Vault: {}", vault_name))
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

            EditorScreen::VaultSelectionPrompt => {
                container(column![
                    text("Select a vault:")
                        .align_x(Center)
                        .width(Fill)
                ])
                    .into()
            }

            EditorScreen::PasswordPrompt => {
                let vault_name;

                match self.opened_vault.clone() {
                    Some(v_name) => { vault_name = v_name; }
                    None => { vault_name = String::default(); }
                }

                let mut cols = column![
                    Space::new(Fill, 100),
                    text!("Enter Password for Vault \"{}\"", vault_name)
                        .align_x(Center)
                        .width(Fill),
                    Space::new(Fill, 16),
                    container(
                        text_input("", &self.vault_password)
                            .id(text_input::Id::new("vault-password"))
                            .secure(true)
                            .width(300)
                            .on_input(EditorMessage::VaultPasswordChanged)
                            .on_submit(EditorMessage::VaultPasswordSubmitted)
                    )
                        .align_x(Center)
                        .width(Fill),
                    Space::new(Fill, 16),
                    container(
                        button(text("Open Vault"))
                            .on_press(EditorMessage::VaultPasswordSubmitted)
                    )
                        .align_x(Center)
                        .width(Fill),
                ];

                match self.vault_password_status {
                    EditorVaultPasswordStatus::DoesNotMatch => {
                        cols = cols.push(Space::new(Fill, 16));

                        cols = cols.push(
                            text("Wrong password!")
                                .align_x(Center)
                                .width(Fill)
                                .color(Color::new(0.9, 0.0, 0.0, 1.0))
                        );
                    }

                    EditorVaultPasswordStatus::Empty => {
                        cols = cols.push(Space::new(Fill, 16));

                        cols = cols.push(
                            text("Password cannot be empty!")
                                .align_x(Center)
                                .width(Fill)
                                .color(Color::new(0.9, 0.0, 0.0, 1.0))
                        );
                    }

                    EditorVaultPasswordStatus::NONE => {}

                    EditorVaultPasswordStatus::Loading => {
                        println!("editor vault password status: loading");
                        cols = column![
                            text!("Please wait...")
                                .align_x(Center)
                                .width(Fill),
                        ];
                    }

                    EditorVaultPasswordStatus::Authenticated => {}
                }

                container(cols)
                    .style(move |_| style)
                    .width(Fill)
                    .height(Fill)
                    .into()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<EditorMessage> {
        let event_subscription = event::listen().map(EditorMessage::Event);
        let auth_sub;

        if self.vault_password_status == EditorVaultPasswordStatus::Loading {
            auth_sub = Subscription::run(auth_worker);
        } else {
            auth_sub = Subscription::none();
        }

        Subscription::batch([
            event_subscription,
            auth_sub,
        ])
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

fn send_async_message(sender: &mut Sender<EditorMessage>, msg: EditorMessage) {
    async fn send_values(
        sender: &mut Sender<EditorMessage>,
        msg: EditorMessage
    ) {
        let _ = sender.send(msg).await;
    }

    executor::block_on(send_values(sender, msg));
}

fn auth_worker() -> impl Stream<Item = EditorMessage> {
    channel(1, move | mut sender | async move {
        let ( pv_sender, mut pv_receiver ) = mpsc::channel::<(String, String)>(1);

        let mut sender_clone = sender.clone();

        let t_handle = thread::spawn(move || {
            let mut vault_name: String = String::default();
            let mut password: String = String::default();

            thread::park();

            match pv_receiver.try_next() {
                Ok(receiver_option) => {
                    match receiver_option {
                        Some((_vault_name, _password)) => {
                            vault_name = _vault_name;
                            password = _password;
                        }

                        None => {}
                    }
                }

                Err(e) => {
                    eprintln!("Thread 1: Error: {}", e);
                }
            }

            let vault_empty = vault_name.is_empty();
            let password_empty = password.is_empty();

            if !vault_empty && !password_empty {
                println!("Thread 1: authenticating");
                if authenticate_vault(vault_name.as_str(), password.as_str()) {
                    send_async_message(&mut sender, EditorMessage::PVAuthenticated);
                } else {
                    send_async_message(&mut sender, EditorMessage::PVDoesNotMatch);
                }
            } else {
                if vault_empty && password_empty {
                    send_async_message(&mut sender, EditorMessage::PVVaultAndPasswordEmpty);
                } else if vault_empty {
                    send_async_message(&mut sender, EditorMessage::PVVaultEmpty);
                } else {
                    send_async_message(&mut sender, EditorMessage::PVPasswordEmpty);
                }
            }
        });

        let _ = sender_clone.send(EditorMessage::PVInitSender(Arc::new(t_handle), pv_sender)).await;
    })
}

