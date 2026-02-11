use super::recorder::Recorder;
use super::settings::game::{ALL_GAMES, Game};
use super::settings::screen::{ScreenInfo, get_screens_vec};
use super::settings::settings::Settings;
use super::storage::Storage;
use crate::i18n::{
    language::{ALL_LANGUAGES, Language},
    translations::I18n,
};
use crate::screens::main_screen::{MainScreen, MainScreenMessage};
use crate::utils::app_worker::{hotkey_subscription, ocr_subscription};
use iced::Color;
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
    ChangeView(Screen),
    AddCounter,
    CancelAddCounter,
    AutosaveTick,
    ActivateOCR(bool),
    StartingOCR,
    OCROK,
    ChangeActionOCR(ActionOCR),
    SaveSettings,
    GameSelected(Game),
    LanguageSelected(Language),
    ScreenSelected(ScreenInfo),
    TitleChanged(String),
    DeathText(String),
}

#[derive(Clone, Debug)]
pub enum Screen {
    MainScreen(MainScreen),
    AddRecorder,
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
    new_recorder_title: String,
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
        Self::ensure_global_counters(&mut recorders);
        let settings = Storage::load_settings().unwrap_or_default();
        let i18n = I18n::new(settings.get_language().clone());
        let screens_list = get_screens_vec().unwrap_or_default();
        App {
            screen: Screen::MainScreen(MainScreen::new()),
            new_recorder_title: "".to_string(),
            dirty: false,
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
            settings,
            screens_list,
            i18n,
        }
    }

    fn go_to(&mut self, screen: Screen) -> () {
        match screen {
            Screen::MainScreen(main_screen) => self.screen = Screen::MainScreen(main_screen),
            _ => self.screen = screen,
        }
    }
    fn ensure_global_counters(recorders: &mut Vec<Recorder>) {
        let has_global_deaths = recorders.iter().any(|r| r.is_global_deaths());
        let has_global_bosses = recorders.iter().any(|r| r.is_global_bosses());

        // Retirer les globaux de leur position actuelle
        //recorders.retain(|r| !r.is_global());

        // Les réinsérer dans l'ordre en haut
        if !has_global_bosses {
            recorders.insert(0, Recorder::new_global_bosses());
        }

        if !has_global_deaths {
            recorders.insert(0, Recorder::new_global_deaths());
        }
    }

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::MainScreen(list_message) => match list_message {
                _ => match &mut self.screen {
                    Screen::MainScreen(main_screen) => {
                        main_screen.update(list_message);
                    }
                    _ => {}
                },
            },
            MessageApp::TitleChanged(value) => {
                self.new_recorder_title = value;
            }
            MessageApp::AddCounter => {
                let title = self.new_recorder_title.trim();

                if !title.is_empty() {
                    self.add_recorder(title.to_string());
                    self.dirty();
                    self.go_to(Screen::MainScreen(MainScreen::new()));
                }
            }
            MessageApp::CancelAddCounter => {
                self.go_to(Screen::MainScreen(MainScreen::new()));
            }

            MessageApp::ChangeView(screen) => self.screen = screen,

            MessageApp::AutosaveTick => {
                if self.dirty {
                    #[cfg(feature = "no_save")]
                    {
                        println!("🐛 Mode DEBUG - sauvegarde ignorée");
                    }

                    #[cfg(not(feature = "no_save"))]
                    {
                        println!("💾 Autosave!");
                        //let _ = Storage::save_recorders(&self.recorders);
                    }

                    self.dirty = false;
                }
            }
            MessageApp::DeathText(text) => {
                self.settings.set_death_text(text);
            }

            MessageApp::ActivateOCR(b) => {
                self.ocr_activate = b;
                if !self.ocr_activate {
                    self.ocr_status = StatusOCR::Stopped;
                    println!("🤖 OCR stopped !");
                }
            }

            MessageApp::StartingOCR => {
                self.ocr_status = StatusOCR::Starting;
                println!("🤖 OCR starting !");
            }
            MessageApp::OCROK => {
                self.ocr_status = StatusOCR::Started(ActionOCR::SearchingDeath);
                println!("🤖 OCR started !");
            }

            MessageApp::ChangeActionOCR(status) => {
                println!("🤖 OCR status changed to {:?}", status);
                self.ocr_status = StatusOCR::Started(status);
            }

            MessageApp::SaveSettings => {
                let _ = Storage::save_settings(&self.settings);
                self.screen = Screen::MainScreen(MainScreen::new());
            }
            MessageApp::GameSelected(game) => {
                self.settings.set_game(game);
            }
            MessageApp::LanguageSelected(language) => {
                self.set_language(language);
            }
            MessageApp::ScreenSelected(screen) => {
                self.settings.set_screen(screen.index);
            }
        };
    }

    pub fn set_language(&mut self, language: Language) {
        self.settings.set_language(language.clone());
        self.i18n.set_language(language.clone());
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match &self.screen {
            Screen::MainScreen(main_screen) => main_screen.view().map(MessageApp::MainScreen),
            Screen::AddRecorder => self.view_add_recorder(),
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

    pub fn reset_new_recorder_title(&mut self) -> () {
        self.new_recorder_title = "".to_string()
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

    pub fn view_add_recorder(&self) -> Element<'_, MessageApp> {
        let content = column![
            text("Ajouter un enregistreur".to_string())
                .size(30)
                .width(Length::Fill)
        ]
        .spacing(10)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill);

        let input = text_input("title", &self.new_recorder_title)
            .on_input(MessageApp::TitleChanged)
            .on_submit_maybe(Some(MessageApp::AddCounter));
        let button_row = row![
            button("Ajouter").on_press(MessageApp::AddCounter),
            button("Annuler").on_press(MessageApp::CancelAddCounter)
        ];

        content.push(input).push(button_row).into()
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
        if self.dirty {
            //let _ = Storage::save_recorders(&self.recorders);
        }
    }
}
