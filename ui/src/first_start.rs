///
/// Shows 3 pages in a typical wizard kind of format
///
/// 1. An introduction to secure notes.
/// 2. Information about vaults and where to find them.
///     - Enter name of a vault parent folder.
///     - Enter password for vault.
///     - Option to make this default vault (default selected for the first
///     vault).
/// 3. A "Done" page.
///
/// ### Notes
/// - At the bottom of each page, there are "Back" and "Next" buttons (except
/// the third a.k.a "Done" page).
/// - Vault only created when user clicks on "Next" button on the second page

use std::{
    env::args, thread::sleep, time::Duration, process::{ Command, exit },
};

use iced::{
    Element, Center, Fill, Padding, Color,
    widget::{ column, text, container, Space, button, text_input, TextInput },
};

use crate::utils::vault::create_vault;

#[derive(Debug, Clone)]
pub enum Message {
    Page(Page),
    VaultNameChanged(String),
    VaultPasswordChanged(String),
    CreateVault,
    Restart,
    Exit,
}

#[derive(Default, Debug, Clone)]
pub enum Page {
    #[default]
    P1,
    P2,
    P3
}

#[derive(Debug, Default)]
pub struct FirstStart {
    current_page: Page,
    password_error: bool,
    name_error: bool,
    vault_name: String,
    vault_password: String,
}

impl FirstStart {
    pub fn view(&self) -> Element<Message> {
        match self.current_page {
            Page::P1 => {
                let title = text("Welcome to Secure Notes!")
                    .width(Fill)
                    .size(48)
                    .align_x(Center);

                let version = text("v0.0.1")
                    .width(Fill)
                    .size(14)
                    .align_x(Center);

                let intro_text1 = text("Secure notes is a free and open source notes editor and management tool.")
                    .width(Fill)
                    .align_x(Center);

                let intro_text2 = text("Everything is stored inside a \"Vault\".")
                    .width(Fill)
                    .align_x(Center);

                let intro_text3 = text("Each vault has it's own unique password and contains multiple notes and folders.")
                    .width(Fill)
                    .align_x(Center);

                let intro_text4 = text("Each folder can contain multiple folders or notes.")
                    .width(Fill)
                    .align_x(Center);

                let get_started_button = container(
                    button(text("Get Started"))
                        .style(button::primary)
                        .on_press(Message::Page(Page::P2))
                )
                    .width(Fill)
                    .align_x(Center);

                column![
                    Space::new(Fill, 100),
                    title,
                    version,
                    Space::new(Fill, 20),
                    intro_text1,
                    Space::new(Fill, 10),
                    intro_text2,
                    intro_text3,
                    intro_text4,
                    Space::new(Fill, 40),
                    get_started_button,
                ].into()
            }

            Page::P2 => {
                let title = text("Create a Vault")
                    .width(Fill)
                    .size(48)
                    .align_x(Center);

                let name_row: Element<'_, Message>;

                let password_row: Element<'_, Message>;

                let name_input = container(
                    text_input("Vault Name", &self.vault_name)
                        .width(300)
                        .on_input(Message::VaultNameChanged)
                )
                    .align_x(Center)
                    .align_y(Center)
                    .width(Fill)
                    .into();

                if self.name_error {
                    name_row = column![
                        name_input,
                        text("Vault name must be between 4 and 32 characters.")
                            .width(Fill)
                            .align_x(Center)
                            .color(Color::new(0.9, 0.0, 0.0, 1.0)),
                    ].into();
                } else {
                    name_row = name_input;
                }

                let password_input = container(
                    TextInput::new("Vault Password", &self.vault_password)
                        .secure(true)
                        .width(300)
                        .on_input(Message::VaultPasswordChanged)
                )
                    .align_x(Center)
                    .align_y(Center)
                    .width(Fill)
                    .into();

                if self.password_error {
                    password_row = column![
                        password_input,
                        text("Vault password must be between 8 and 32 characters.")
                            .width(Fill)
                            .align_x(Center)
                            .color(Color::new(0.9, 0.0, 0.0, 1.0)),
                    ].into();
                } else {
                    password_row = password_input;
                }

                let control_row_padding = Padding::from([ 50, 200 ]);

                column![
                    Space::new(Fill, 100),
                    title,
                    Space::new(Fill, 20),
                    name_row,
                    Space::new(Fill, 20),
                    password_row,
                    column![
                        container(
                            button(text("Create Vault"))
                                .style(button::primary)
                                .on_press(Message::CreateVault)
                        )
                            .align_x(Center)
                            .width(Fill),
                        Space::new(Fill, 10),
                        container(
                            button(text("Back"))
                                .style(button::secondary)
                                .on_press(Message::Page(Page::P1))
                        )
                            .align_x(Center)
                            .width(Fill),
                    ].padding(control_row_padding),
                ].into()
            }

            Page::P3 => {
                let title = text("Done!")
                    .width(Fill)
                    .size(48)
                    .align_x(Center);

                let success_text = text("Your new vault has been successfully created.")
                    .width(Fill)
                    .align_x(Center);

                let next_step_text = text("Click on restart to restart the app.")
                    .width(Fill)
                    .align_x(Center);

                let get_started_button = container(
                    button(text("Restart"))
                        .style(button::primary)
                        .on_press(Message::Restart)
                )
                    .width(Fill)
                    .align_x(Center);

                let exit_button = container(
                    button(text("Exit"))
                        .style(button::secondary)
                        .on_press(Message::Exit)
                )
                    .width(Fill)
                    .align_x(Center);

                column![
                    title,
                    Space::new(Fill, 10),
                    success_text,
                    Space::new(Fill, 10),
                    next_step_text,
                    Space::new(Fill, 10),
                    get_started_button,
                    Space::new(Fill, 10),
                    exit_button,
                ].into()
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Page(p) => {
                self.current_page = p;
            }

            Message::VaultNameChanged(updated_vault_name) => {
                self.vault_name = updated_vault_name;
                self.name_error = false;
            }

            Message::VaultPasswordChanged(updated_vault_password) => {
                self.vault_password = updated_vault_password;
                self.password_error = false;
            }

            Message::CreateVault => {
                if self.vault_name.len() > 32 || self.vault_name.len() < 4 {
                    self.name_error = true;
                } else {
                    self.name_error = false;
                }

                if self.vault_password.len() > 32 || self.vault_name.len() < 8 {
                    self.password_error = true;
                } else {
                    self.password_error = false;
                }

                if !(self.name_error && self.password_error) {
                    match create_vault(self.vault_name.clone(), self.vault_password.clone(), true) {
                        Ok(_) => {
                            self.current_page = Page::P3;
                        }

                        Err(_) => {}
                    }
                }
            }

            Message::Restart => {
                let program = args().collect::<String>();

                println!("Opening: {}", program);

                match Command::new(program).spawn() {
                    Ok(_) => {}
                    Err(e) => { eprintln!("{}", e); }
                }

                sleep(Duration::from_millis(1000));

                exit(0);
            }

            Message::Exit => {
                exit(0);
            }
        }
    }
}

