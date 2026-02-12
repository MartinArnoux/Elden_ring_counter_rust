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

use iced::widget::{Column, PickList, Row, button, pick_list, text_input};
use iced::{Element, Length, Task};
use iced_core::widget::Text;

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
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .push(Text::new("Paramètres").size(30));

        // Valeur sélectionnée (Option<Game>)
        let selected_game = Some(self.settings.get_game());

        let game_pick_list = pick_list(
            ALL_GAMES,
            selected_game,
            SettingsScreenMessage::GameSelected,
        );

        let game_row = Row::new()
            .spacing(10)
            .push(Text::new("Game:"))
            .push(game_pick_list);

        // On garde tes autres éléments

        let select_language = pick_list(
            ALL_LANGUAGES,                           // &[Language]
            Some(self.settings.get_language()),      // Option<Language>
            SettingsScreenMessage::LanguageSelected, // fn(Language) -> Message
        );

        let language_row = Row::new()
            .spacing(10)
            .push(Text::new("Langue:"))
            .push(select_language);

        let selected_screen = self
            .screens_list
            .iter()
            .find(|s| s.index == self.settings.get_screen())
            .cloned();

        let screen_pick_list = PickList::new(
            self.screens_list.as_slice(), // ✅ IMPORTANT
            selected_screen,
            SettingsScreenMessage::ScreenSelected,
        );

        let screen_row = Row::new()
            .spacing(10)
            .push(Text::new("Écran:"))
            .push(screen_pick_list);

        let death_text_input = text_input("Death texte", &self.settings.get_death_text())
            .on_input(SettingsScreenMessage::DeathText);
        let button_save = button("Enregistrer").on_press(SettingsScreenMessage::SaveSettings);

        content = content
            .push(game_row)
            .push(language_row)
            .push(screen_row)
            .push(death_text_input)
            .push(button_save);

        content.into()
    }

    pub fn set_language(&mut self, language: Language) {
        self.settings.set_language(language.clone());
    }
}
