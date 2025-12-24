use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;

use iced::{
    Alignment, Background, Color, Element, Length, Program, Subscription, Task, Theme, border,
    task, time,
    time::Duration,
    widget::{Space, button, column, container, row, stack, text, text_input, toggler},
};

#[derive(Clone)]
pub enum MessageApp {
    Increment,
    ChangeView(Screen),
    AddCounter,
    DeleteCounter(usize),
    ToggleCounter(usize),
    TitleChanged(String),
    CancelAddCounter,
    SaveRecorders,
    AutosaveTick,
}
#[derive(Default, Clone)]
enum Screen {
    #[default]
    List,
    AddRecorder,
}

#[derive(Default, Clone)]
pub struct App {
    recorders: Vec<Recorder>,
    screen: Screen,
    new_recorder_title: String,
    dirty: bool,
}

impl App {
    pub fn new() -> App {
        let recorders = Storage::load_recorders().unwrap_or_default();
        println!("Load recorders : {:?}", recorders);
        App {
            recorders: recorders,
            screen: Screen::List,
            new_recorder_title: "".to_string(),
            dirty: false,
        }
    }

    fn go_to(&mut self, screen: Screen) -> () {
        match screen {
            Screen::AddRecorder => self.screen = screen,
            Screen::List => {
                self.reset_new_recorder_title();
                self.screen = screen
            }
        }
    }

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::Increment => self.update_all_counter(),
            MessageApp::AddCounter => {
                self.add_recorder(self.new_recorder_title.clone());
                self.dirty();
                self.go_to(Screen::List);
            }
            MessageApp::CancelAddCounter => {
                self.go_to(Screen::List);
            }
            MessageApp::DeleteCounter(x) => {
                self.delete_recorder(x);
                self.dirty();
            }

            MessageApp::ToggleCounter(i) => {
                if let Some(r) = self.recorders.get_mut(i) {
                    r.activate_deactivate();
                    self.dirty();
                }
            }
            MessageApp::ChangeView(screen) => self.screen = screen,
            MessageApp::TitleChanged(value) => {
                self.new_recorder_title = value;
            }
            MessageApp::SaveRecorders => {
                if let Err(e) = Storage::save_recorders(&self.recorders) {
                    eprintln!("Erreur de sauvegarde : {e}");
                }
            }
            MessageApp::AutosaveTick => {
                if self.dirty {
                    let _ = Storage::save_recorders(&self.recorders);
                    self.dirty = false;
                }
            }
        };
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match self.screen {
            Screen::List => self.view_list(),
            Screen::AddRecorder => self.view_add_recorder(),
        };

        main
    }
    pub fn subscription(&self) -> Subscription<MessageApp> {
        iced::time::every(Duration::from_secs(10)).map(|_| MessageApp::AutosaveTick)
    }

    pub fn reset_new_recorder_title(&mut self) -> () {
        self.new_recorder_title = "".to_string()
    }

    fn dirty(&mut self) -> () {
        self.dirty = true;
    }

    fn update_all_counter(&mut self) {
        // itération mutable sur le Vec
        for recorder in self.recorders.iter_mut() {
            recorder.increment();
        }
    }

    pub fn add_recorder(&mut self, title: String) -> () {
        self.recorders.push(Recorder::new(title));
    }

    pub fn delete_recorder(&mut self, index: usize) -> () {
        if index > self.recorders.len() {
            return;
        }
        self.recorders.remove(index);
    }

    pub fn save(&self) -> () {
        match Storage::save_recorders(&self.recorders) {
            Ok(_) => {}
            Err(e) => {
                println!("Error save : {}", e)
            }
        }
    }

    pub fn view_list(&self) -> Element<MessageApp> {
        let mut content = column![].spacing(10).padding(20);

        for (index, recorder) in self.recorders.iter().enumerate() {
            let is_active = recorder.get_status_recorder();

            let recorder_row = row![
                text(recorder.get_title()).size(20).width(Length::Fill),
                text(recorder.get_counter().to_string()).size(20),
                toggler(is_active).on_toggle(move |_| MessageApp::ToggleCounter(index))
            ]
            .spacing(20);

            let recorder_container = container(recorder_row)
                .padding(15)
                .width(Length::Fill)
                .style(if is_active {
                    crate::style::style::container_active
                } else {
                    crate::style::style::container_inactive
                });

            content = content.push(recorder_container);
        }

        content = content.push(
            button("➕ Ajouter un recorder").on_press(MessageApp::ChangeView(Screen::AddRecorder)),
        );

        content.into()
    }

    pub fn view_add_recorder(&self) -> Element<MessageApp> {
        let content = column![
            text("Ajouter un enregistreur".to_string())
                .size(30)
                .width(Length::Fill)
        ]
        .spacing(10)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill);
        let input =
            text_input("title", &self.new_recorder_title).on_input(MessageApp::TitleChanged);
        let button = row![
            button("Ajouter").on_press(MessageApp::AddCounter),
            button("Annuler").on_press(MessageApp::CancelAddCounter)
        ];
        content.push(input).push(button).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_starts_empty() {
        let app = App::new();
        assert_eq!(app.recorders.len(), 0);
    }

    #[test]
    fn add_recorder_works() {
        let mut app = App::new();
        app.add_recorder("A".to_string());

        assert_eq!(app.recorders.len(), 1);
        assert_eq!(app.recorders[0].get_title(), "A");
    }

    #[test]
    fn increment_all_recorders() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        app.add_recorder("B".to_string());

        app.update(MessageApp::Increment);

        assert_eq!(app.recorders[0].get_counter(), 1);
        assert_eq!(app.recorders[1].get_counter(), 1);
    }

    #[test]
    fn delete_recorder_works() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        assert_eq!(app.recorders.len(), 1);

        app.update(MessageApp::DeleteCounter(0));

        assert_eq!(app.recorders.len(), 0);
    }

    #[test]
    fn access_recorder_after_delete() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        app.add_recorder("B".to_string());

        assert_eq!(app.recorders.len(), 2);

        app.update(MessageApp::DeleteCounter(0));

        assert_eq!(app.recorders.len(), 1);

        assert_eq!(app.recorders[0].get_title(), "B")
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.dirty {
            let _ = Storage::save_recorders(&self.recorders);
        }
    }
}
