use crate::i18n::translations::AddRecorderKey;
use crate::structs::app::Screen;
use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;
use crate::{i18n::translations::I18n, screens::main_screen::MainScreen};
use iced::{
    Color, Element, Length, Subscription, Task,
    alignment::Alignment,
    widget::{button, column, container, row, text, text_input},
};
#[derive(Debug, Clone)]
pub enum AddRecorderMessage {
    TitleChanged(String),
    AddCounter,
    CancelAddCounter,
    ChangeView(Screen),
}

#[derive(Debug, Clone)]
pub struct AddRecorderScreen {
    title: String,
    error: Option<String>,
}

impl AddRecorderScreen {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            error: None,
        }
    }
    pub fn update(&mut self, message: AddRecorderMessage) -> Task<AddRecorderMessage> {
        match message {
            AddRecorderMessage::TitleChanged(new_title) => {
                self.title = new_title;
                Task::none()
            }
            AddRecorderMessage::AddCounter => {
                if self.title.is_empty() {
                    self.error = Some("Title cannot be empty".to_string());
                    return Task::none();
                }
                let recorder = Recorder::new(self.title.clone());
                match Storage::insert_recorder_at_first_position(&recorder) {
                    Ok(_) => Task::done(AddRecorderMessage::ChangeView(Screen::MainScreen(
                        MainScreen::new(),
                    ))),
                    Err(err) => {
                        self.error = Some(err.to_string());
                        Task::none()
                    }
                }
            }
            AddRecorderMessage::CancelAddCounter => Task::done(AddRecorderMessage::ChangeView(
                Screen::MainScreen(MainScreen::new()),
            )),
            AddRecorderMessage::ChangeView(_) => Task::none(),
        }
    }

    pub fn view(&self, i18n: &I18n) -> Element<'_, AddRecorderMessage> {
        container(
            column![
                text(i18n.add_recorder(AddRecorderKey::Title)).size(28),
                text_input(
                    i18n.add_recorder(AddRecorderKey::InputPlaceholder),
                    &self.title
                )
                .on_input(AddRecorderMessage::TitleChanged)
                .on_submit(AddRecorderMessage::AddCounter)
                .padding(10)
                .size(16),
                // ✅ message d'erreur inline
                if let Some(error) = &self.error {
                    text(error).size(14).color(Color::from_rgb(0.8, 0.0, 0.0))
                } else {
                    text("").into() // ← Utilisez text("") au lieu de column![]
                },
                row![
                    button(i18n.add_recorder(AddRecorderKey::Cancel))
                        .on_press(AddRecorderMessage::CancelAddCounter),
                    if self.title.trim().is_empty() {
                        button(i18n.add_recorder(AddRecorderKey::AddCounter))
                    } else {
                        button(i18n.add_recorder(AddRecorderKey::AddCounter))
                            .on_press(AddRecorderMessage::AddCounter)
                    }
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(15)
            .padding(25)
            .max_width(400)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<AddRecorderMessage> {
        Subscription::none()
    }
}
