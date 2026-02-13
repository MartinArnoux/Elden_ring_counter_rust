use crate::screens::add_recorder_screen::{AddRecorderMessage, AddRecorderScreen};
use crate::screens::main_screen::{MainScreen, MainScreenMessage};
use crate::screens::settings_screen::{SettingsScreen, SettingsScreenMessage};
use iced::task::Task;
use iced::{Element, Subscription};

#[derive(Clone, Debug)]
pub enum MessageApp {
    MainScreen(MainScreenMessage),
    AddRecorderScreen(AddRecorderMessage),
    SettingsScreen(SettingsScreenMessage),
}

#[derive(Clone, Debug)]
pub enum Screen {
    MainScreen(MainScreen),
    AddRecorderScreen(AddRecorderScreen),
    SettingsScreen(SettingsScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::MainScreen(MainScreen::new())
    }
}

#[derive(Clone)]
pub struct App {
    screen: Screen,
}

impl App {
    pub fn new() -> App {
        App {
            screen: Screen::MainScreen(MainScreen::new()),
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

            MessageApp::SettingsScreen(settings_screen_message) => match settings_screen_message {
                SettingsScreenMessage::ChangeView(screen) => {
                    self.go_to(screen);
                    Task::none()
                }
                _ => match &mut self.screen {
                    Screen::SettingsScreen(settings_screen) => settings_screen
                        .update(settings_screen_message)
                        .map(MessageApp::SettingsScreen),
                    _ => Task::none(),
                },
            },
        }
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match &self.screen {
            Screen::MainScreen(main_screen) => main_screen.view().map(MessageApp::MainScreen),
            Screen::AddRecorderScreen(add_recorder_screen) => add_recorder_screen
                .view()
                .map(MessageApp::AddRecorderScreen),
            Screen::SettingsScreen(settings_screen) => {
                settings_screen.view().map(MessageApp::SettingsScreen)
            }
        };
        main
    }

    pub fn subscription(&self) -> Subscription<MessageApp> {
        match &self.screen {
            Screen::MainScreen(main_screen) => {
                main_screen.subscription().map(MessageApp::MainScreen)
            }

            Screen::AddRecorderScreen(add_recorder_screen) => add_recorder_screen
                .subscription()
                .map(MessageApp::AddRecorderScreen),

            Screen::SettingsScreen(settings_screen) => settings_screen
                .subscription()
                .map(MessageApp::SettingsScreen),
        }
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
