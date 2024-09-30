use std::{ thread, time };
use futures::executor;

use iced::{
    futures::{
        channel::{ mpsc, mpsc::{ Sender, Receiver, TryRecvError } },
        Stream, StreamExt, SinkExt
    },
    keyboard::{ key::{ Key, Named }, on_key_press, Modifiers },
    widget::{
        button, column, container, pane_grid, responsive, text, text_editor,
        text_editor::{Action, Content}, text_input, Space
    }, Background, Center, Color, Element, Fill, Subscription, stream::channel,
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

#[derive(Debug, Clone)]
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
    // Action performed on the text editor
    ActionPerformed(Action),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    CloseFocused,
    ToggleExplorer,
    Clicked(pane_grid::Pane),

    // Messages related to vault creation
    VaultPasswordChanged(String),
    VaultPasswordSubmitted,

    // Messages related to notes
    NoteNameChanged(String),
    Save,
    New,

    // Messages related to password validation
    PVVaultEmpty,
    PVPasswordEmpty,
    PVVaultAndPasswordEmpty,
    PVDoesNotMatch,
    PVLoading,
    PVAuthenticated,
    PVInitSender(Sender<(String, String)>),
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

            EditorMessage::None => {}

            EditorMessage::VaultPasswordChanged(updated_password) => {
                self.vault_password = updated_password;
                self.vault_password_status = EditorVaultPasswordStatus::NONE;
            }

            EditorMessage::VaultPasswordSubmitted => {
                self.vault_password_status = EditorVaultPasswordStatus::Loading;
            }

            EditorMessage::NoteNameChanged(note_name) => {
                match self.opened_file.clone() {
                    Some(mut opened_file) => {
                        opened_file.name = note_name;

                        self.opened_file = Some(opened_file);
                    }

                    None => {}
                }
            }

            EditorMessage::Save => {
                // let text = self.content.text();
            }

            EditorMessage::New => {
                self.opened_file = Some(VaultIndexEntry {
                    id: 0,
                    name: String::from("Untitled Note"),
                    parent_folder: None,
                });
            }

            EditorMessage::PVLoading => {
                println!("Password Validation loading");
            }

            EditorMessage::PVVaultEmpty => {
                println!("Password Validation: vault empty");
            }

            EditorMessage::PVDoesNotMatch => {
                println!("Password Validation: password does not match");
                self.vault_password_status
                    = EditorVaultPasswordStatus::DoesNotMatch;
            }

            EditorMessage::PVPasswordEmpty => {
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

            EditorMessage::PVInitSender(mut sender) => {
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
                            println!("** Sending the values here...");
                            let _ = sender.send((vault_name, password)).await;
                        }

                        let thread_handle = thread::spawn(move || {
                            executor::block_on(send_values(&mut sender, vault_name, password));
                            println!("** Sending the vault name and password");
                        });

                        let _ = thread_handle.join();
                    }

                    None => {}
                }
            }
        }
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

                                container(column![
                                    text_input("", opened_file)
                                        .on_input(
                                            EditorMessage::NoteNameChanged
                                        ),
                                    text_editor(&self.content)
                                        .on_action(EditorMessage::ActionPerformed)
                                        .height(Fill)
                                ])
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
                            .secure(true)
                            .width(300)
                            .on_input(EditorMessage::VaultPasswordChanged)
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
                            text("Passwrod cannot be empty!")
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
        let key_press_sub = on_key_press(|key_code, modifiers|{
            match (modifiers, key_code.as_ref()) {
                (Modifiers::CTRL, Key::Character("s")) => {
                    // TODO: Save the file
                    return None;
                }

                (Modifiers::CTRL, Key::Character("e")) =>
                    Some(EditorMessage::ToggleExplorer),

                (Modifiers::CTRL, Key::Character("n")) =>
                    Some(EditorMessage::New),

                _ => None
            }
        });

        let auth_sub;

        if self.vault_password_status == EditorVaultPasswordStatus::Loading {
            auth_sub = Subscription::run(auth_worker);
        } else {
            auth_sub = Subscription::none();
        }

        Subscription::batch([
            key_press_sub,
            auth_sub,
        ])
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

fn auth_worker() -> impl Stream<Item = EditorMessage> {
    channel(1, move | mut sender | async move {
        let ( pv_sender, mut pv_receiver ) = mpsc::channel::<(String, String)>(1);

        println!("Sending the sender");
        let _ = sender.send(EditorMessage::PVInitSender(pv_sender)).await;

        println!("Getting the username and password");

        async fn authenticate(
            pv_receiver: &mut Receiver<(String, String)>,
            sender: &mut Sender<EditorMessage>
        ) {
            let pause_time = time::Duration::from_millis(1000);
            let mut loop_counter = 0;
            let mut vault_name: String = String::default();
            let mut password: String = String::default();

            println!("Inside thread!");

            while loop_counter < 100 {
                match pv_receiver.try_next() {
                    Ok(receiver_option) => {
                        match receiver_option {
                            Some((_vault_name, _password)) => {
                                vault_name = _vault_name;
                                password = _password;
                                println!("** vault: {}, password: {}", vault_name, password);

                                break;
                            }

                            None => {}
                        }
                    }

                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }

                loop_counter += 1;
                thread::sleep(pause_time);
            }

            let vault_empty = vault_name.is_empty();
            let password_empty = password.is_empty();

            if !vault_empty && !password_empty {
                println!("authenticating");
                if authenticate_vault(vault_name.as_str(), password.as_str()) {
                    let _ = sender.send(EditorMessage::PVAuthenticated).await;
                } else {
                    let _ = sender.send(EditorMessage::PVDoesNotMatch).await;
                }
            } else {
                if vault_empty && password_empty {
                    let _ = sender.send(EditorMessage::PVVaultAndPasswordEmpty).await;
                } else if vault_empty {
                    let _ = sender.send(EditorMessage::PVVaultEmpty).await;
                } else {
                    let _ = sender.send(EditorMessage::PVPasswordEmpty).await;
                }
            }
        }

        let _ = thread::spawn(move || {
            executor::block_on(authenticate(&mut pv_receiver, &mut sender));
        });
    })
}

