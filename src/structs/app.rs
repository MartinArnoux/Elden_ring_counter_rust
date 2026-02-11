use super::recorder::Recorder;
use super::settings::game::{ALL_GAMES, Game};
use super::settings::screen::{ScreenInfo, get_screens_vec};
use super::settings::settings::Settings;
use super::storage::Storage;
use crate::i18n::{
    language::{ALL_LANGUAGES, Language},
    translations::I18n,
};
use crate::screens::add_recorder_screen::{AddRecorderMessage, AddRecorderScreen};
use crate::screens::main_screen::{MainScreen, MainScreenMessage};
use crate::utils::app_worker::{hotkey_subscription, ocr_subscription};
use iced::Color;
use iced::task::Task;
use iced::widget::{Column, PickList};
use iced::widget::{Container, Row, pick_list};
use iced::{
    Element, Length, Subscription,
    time::Duration,
    widget::{button, column, container, row, scrollable, text, text_input, toggler},
};
use iced_aw::Spinner;
use iced_core::widget::Text;
use strsim::normalized_levenshtein;
use uuid::Uuid;
#[derive(Clone, Debug)]
pub enum ActionOCR {
    SearchingDeath,
    SearchingBossName,
    EndingAction,
}

#[derive(Clone, Debug)]
pub enum StatusOCR {
    Starting,
    Started(ActionOCR),
    Stopped,
}

#[derive(Clone, Debug)]
pub enum MessageApp {
    MainScreen(MainScreenMessage),
    AddRecorderScreen(AddRecorderMessage),
    ChangeView(Screen),
    AutosaveTick,
    ActivateOCR(bool),
    StartingOCR,
    OCROK,
    ChangeActionOCR(ActionOCR),
    SaveSettings,
    GameSelected(Game),
    LanguageSelected(Language),
    ScreenSelected(ScreenInfo),
    DeathText(String),
}

#[derive(Clone, Debug)]
pub enum Screen {
    MainScreen(MainScreen),
    AddRecorderScreen(AddRecorderScreen),
    Settings,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::MainScreen(MainScreen::new())
    }
}

#[derive(Clone)]
pub struct App {
    settings: Settings,
    i18n: I18n,
    screen: Screen,
    dirty: bool,
    ocr_activate: bool,
    ocr_status: StatusOCR,
    screens_list: Vec<ScreenInfo>,
}

impl App {
    pub fn new() -> App {
        let mut recorders = {
            #[cfg(feature = "no_save")]
            {
                println!("🐛 Mode DEBUG activé - pas de chargement des données");
                Vec::new()
            }

            #[cfg(not(feature = "no_save"))]
            {
                Storage::load_recorders().unwrap_or_default()
            }
        };
        let settings = Storage::load_settings().unwrap_or_default();
        let i18n = I18n::new(settings.get_language().clone());
        let screens_list = get_screens_vec().unwrap_or_default();
        App {
            screen: Screen::MainScreen(MainScreen::new()),
            dirty: false,
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
            settings,
            screens_list,
            i18n,
        }
    }

    fn go_to(&mut self, screen: Screen) -> () {
        match &self.screen {
            Screen::MainScreen(main_screen) => {
                main_screen.save();
            }
            _ => {}
        }
        match screen {
            Screen::MainScreen(main_screen) => self.screen = Screen::MainScreen(main_screen),
            _ => self.screen = screen,
        }
    }

    pub fn update(&mut self, message: MessageApp) -> Task<MessageApp> {
        match message {
            MessageApp::MainScreen(main_screen_message) => match main_screen_message {
                MainScreenMessage::ChangeView(view) => {
                    self.go_to(view);
                    Task::none()
                }
                _ => match &mut self.screen {
                    Screen::MainScreen(main_screen) => main_screen
                        .update(main_screen_message)
                        .map(MessageApp::MainScreen),
                    _ => Task::none(),
                },
            },

            MessageApp::AddRecorderScreen(add_recorder_screen_message) => {
                match add_recorder_screen_message {
                    AddRecorderMessage::ChangeView(view) => {
                        self.go_to(view);
                        Task::none()
                    }
                    _ => match &mut self.screen {
                        Screen::AddRecorderScreen(add_recorder_screen) => add_recorder_screen
                            .update(add_recorder_screen_message)
                            .map(MessageApp::AddRecorderScreen),
                        _ => Task::none(),
                    },
                }
            }

            MessageApp::ChangeView(screen) => {
                self.screen = screen;
                Task::none()
            }

            MessageApp::AutosaveTick => {
                self.save();
                Task::none()
            }
            MessageApp::DeathText(text) => {
                self.settings.set_death_text(text);
                Task::none()
            }

            MessageApp::ActivateOCR(b) => {
                self.ocr_activate = b;
                if !self.ocr_activate {
                    self.ocr_status = StatusOCR::Stopped;
                    println!("🤖 OCR stopped !");
                }
                Task::none()
            }

            MessageApp::StartingOCR => {
                self.ocr_status = StatusOCR::Starting;
                println!("🤖 OCR starting !");
                Task::none()
            }
            MessageApp::OCROK => {
                self.ocr_status = StatusOCR::Started(ActionOCR::SearchingDeath);
                println!("🤖 OCR started !");
                Task::none()
            }

            MessageApp::ChangeActionOCR(status) => {
                println!("🤖 OCR status changed to {:?}", status);
                self.ocr_status = StatusOCR::Started(status);
                Task::none()
            }

            MessageApp::SaveSettings => {
                let _ = Storage::save_settings(&self.settings);
                self.screen = Screen::MainScreen(MainScreen::new());
                Task::none()
            }
            MessageApp::GameSelected(game) => {
                self.settings.set_game(game);
                Task::none()
            }
            MessageApp::LanguageSelected(language) => {
                self.set_language(language);
                Task::none()
            }
            MessageApp::ScreenSelected(screen) => {
                self.settings.set_screen(screen.index);
                Task::none()
            }
        }
    }

    pub fn set_language(&mut self, language: Language) {
        self.settings.set_language(language.clone());
        self.i18n.set_language(language.clone());
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match &self.screen {
            Screen::MainScreen(main_screen) => main_screen.view().map(MessageApp::MainScreen),
            Screen::AddRecorderScreen(add_recorder_screen) => add_recorder_screen
                .view()
                .map(MessageApp::AddRecorderScreen),
            Screen::Settings => self.view_settings(),
        };
        main
    }

    pub fn subscription(&self) -> Subscription<MessageApp> {
        let autosave = iced::time::every(Duration::from_secs(10)).map(|_| MessageApp::AutosaveTick);

        let hotkey_sub = hotkey_subscription();

        // ✅ Conditionnellement créer la subscription OCR
        let ocr_sub = if self.ocr_activate {
            ocr_subscription(
                self.settings.get_screen(),
                self.settings.get_game_config(),
                self.settings.get_death_text().clone(),
            )
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![autosave, hotkey_sub, ocr_sub])
    }

    fn dirty(&mut self) -> () {
        self.dirty = true;
    }

    pub fn add_recorder(&mut self, title: String) -> () {
        // if self.recorders_exist(title.clone()) {
        //     return;
        // }
        //self.recorders.push(Recorder::new(title));
    }

    pub fn view_settings(&self) -> Element<'_, MessageApp> {
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .push(Text::new("Paramètres").size(30));

        // Valeur sélectionnée (Option<Game>)
        let selected_game = Some(self.settings.get_game());

        let game_pick_list = pick_list(ALL_GAMES, selected_game, MessageApp::GameSelected);

        let game_row = Row::new()
            .spacing(10)
            .push(Text::new("Game:"))
            .push(game_pick_list);

        // On garde tes autres éléments

        let select_language = pick_list(
            ALL_LANGUAGES,                      // &[Language]
            Some(self.settings.get_language()), // Option<Language>
            MessageApp::LanguageSelected,       // fn(Language) -> Message
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
            MessageApp::ScreenSelected,
        );

        let screen_row = Row::new()
            .spacing(10)
            .push(Text::new("Écran:"))
            .push(screen_pick_list);

        let death_text_input = text_input("Death texte", &self.settings.get_death_text())
            .on_input(MessageApp::DeathText);
        let button_save = button("Enregistrer").on_press(MessageApp::SaveSettings);

        content = content
            .push(game_row)
            .push(language_row)
            .push(screen_row)
            .push(death_text_input)
            .push(button_save);

        content.into()
    }

    fn save(&self) {
        match &self.screen {
            Screen::MainScreen(main_screen) => main_screen.save(),
            _ => {}
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn app_starts_empty() {
//         let app = App::new();
//         assert_eq!(app.recorders.len(), 0);
//     }

//     #[test]
//     fn add_recorder_works() {
//         let mut app = App::new();
//         app.add_recorder("A".to_string());

//         assert_eq!(app.recorders.len(), 1);
//         assert_eq!(app.recorders[0].get_title(), "A");
//     }

//     #[test]
//     fn increment_all_recorders() {
//         let mut app = App::new();
//         app.add_recorder("A".to_string());
//         app.add_recorder("B".to_string());

//         app.update(MessageApp::Increment);

//         assert_eq!(app.recorders[0].get_counter(), 1);
//         assert_eq!(app.recorders[1].get_counter(), 1);
//     }

//     #[test]
//     fn delete_recorder_works() {
//         let mut app = App::new();
//         app.add_recorder("A".to_string());
//         assert_eq!(app.recorders.len(), 1);
//         let uuid = app.recorders.get(0).unwrap().get_uuid();

//         app.update(MessageApp::DeleteCounter(*uuid));

//         assert_eq!(app.recorders.len(), 0);
//     }

//     #[test]
//     fn access_recorder_after_delete() {
//         let mut app = App::new();
//         app.add_recorder("A".to_string());
//         app.add_recorder("B".to_string());

//         assert_eq!(app.recorders.len(), 2);
//         let uuid = app.recorders.get(0).unwrap().get_uuid();
//         app.update(MessageApp::DeleteCounter(*uuid));

//         assert_eq!(app.recorders.len(), 1);

//         assert_eq!(app.recorders[0].get_title(), "B")
//     }
// }

impl Drop for App {
    fn drop(&mut self) {
        self.save();
    }
}
