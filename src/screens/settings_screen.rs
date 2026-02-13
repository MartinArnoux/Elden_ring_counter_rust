use crate::{
    i18n::{Language, language::ALL_LANGUAGES},
    screens::main_screen::MainScreen,
    structs::{
        app::Screen,
        settings::{
            game::{ALL_GAMES, Game},
            screen::{ScreenInfo, get_screens_vec},
            settings::Settings,
        },
    },
};

use iced::{Alignment, Element, Length, Task};
use iced::{
    Subscription,
    widget::{PickList, button, column, container, pick_list, row, text, text_input},
};

#[derive(Debug, Clone)]
pub enum SettingsScreenMessage {
    SaveSettings,
    GameSelected(Game),
    LanguageSelected(Language),
    ScreenSelected(ScreenInfo),
    DeathText(String),

    ChangeView(Screen),
}

#[derive(Debug, Clone)]
pub struct SettingsScreen {
    settings: Settings,
    screens_list: Vec<ScreenInfo>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        let screens_list = get_screens_vec().unwrap_or_default();
        let settings = Settings::load();
        Self {
            settings,
            screens_list,
        }
    }

    pub fn update(&mut self, message: SettingsScreenMessage) -> Task<SettingsScreenMessage> {
        match message {
            SettingsScreenMessage::SaveSettings => {
                self.settings.save();
                Task::done(SettingsScreenMessage::ChangeView(Screen::MainScreen(
                    MainScreen::new(),
                )))
            }
            SettingsScreenMessage::GameSelected(game) => {
                self.settings.set_game(game);
                Task::none()
            }
            SettingsScreenMessage::LanguageSelected(language) => {
                self.set_language(language);
                Task::none()
            }
            SettingsScreenMessage::ScreenSelected(screen) => {
                self.settings.set_screen(screen.index);
                Task::none()
            }
            SettingsScreenMessage::DeathText(text) => {
                self.settings.set_death_text(text);
                Task::none()
            }
            SettingsScreenMessage::ChangeView(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, SettingsScreenMessage> {
        let label_width = 120;
        let spacing_item = 15;
        let form = column![
            container(text("Paramètres").size(32))
                .width(Length::Fill)
                .center_x(Length::Fill),
            row![
                text("Game:").width(label_width),
                pick_list(
                    ALL_GAMES,
                    Some(self.settings.get_game()),
                    SettingsScreenMessage::GameSelected,
                )
                .width(Length::Fill)
            ]
            .align_y(Alignment::Center)
            .spacing(spacing_item),
            row![
                text("Langue:").width(label_width),
                pick_list(
                    ALL_LANGUAGES,
                    Some(self.settings.get_language()),
                    SettingsScreenMessage::LanguageSelected,
                )
                .width(Length::Fill)
            ]
            .align_y(Alignment::Center)
            .spacing(spacing_item),
            row![
                text("Écran:").width(label_width),
                PickList::new(
                    self.screens_list.as_slice(),
                    self.screens_list
                        .iter()
                        .find(|s| s.index == self.settings.get_screen())
                        .cloned(),
                    SettingsScreenMessage::ScreenSelected,
                )
                .width(Length::Fill)
            ]
            .align_y(Alignment::Center)
            .spacing(spacing_item),
            row![
                text("Death texte:").width(label_width),
                text_input("Death texte", &self.settings.get_death_text())
                    .on_input(SettingsScreenMessage::DeathText)
                    .width(Length::Fill)
            ]
            .align_y(Alignment::Center)
            .spacing(spacing_item),
            button("Enregistrer")
                .on_press(SettingsScreenMessage::SaveSettings)
                .width(Length::Fill)
        ]
        .spacing(20)
        .max_width(500);

        container(form).center_x(Length::Fill).padding(30).into()
    }

    pub fn subscription(&self) -> Subscription<SettingsScreenMessage> {
        Subscription::none()
    }

    pub fn set_language(&mut self, language: Language) {
        self.settings.set_language(language.clone());
    }
}
