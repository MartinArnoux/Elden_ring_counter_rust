use crate::structs::recorder::Recorder;
use iced::{
    Alignment, Background, Color, Element, Length, Task, Theme, border,
    widget::{Space, button, column, container, row, stack, text, text_input, toggler},
};

use crate::style::*;
#[derive(Clone)]
pub enum MessageApp {
    Increment,
    AddCounter(String),
    DeleteCounter(usize),
    WindowAddCounter,
    ActivateCounter(usize),
    CloseCounter(usize),
    ToggleCounter(usize),
    TitleInputChanged(String),
    ConfirmAddCounter,
    CancelAddCounter,
}
#[derive(Default)]
pub struct App {
    recorders: Vec<Recorder>,
    window_add_counter: bool,
    new_recorder_title: String,
    title_error: bool,
}

impl App {
    pub fn new() -> App {
        App {
            recorders: Vec::new(),
            window_add_counter: false,
            new_recorder_title: String::new(),
            title_error: false,
        }
    }

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::Increment => self.update_all_counter(),
            MessageApp::AddCounter(title) => self.add_recorder(title),
            MessageApp::DeleteCounter(x) => self.delete_recorder(x),
            MessageApp::WindowAddCounter => {
                self.window_add_counter = true;
                self.new_recorder_title.clear();
                self.title_error = false;
            }
            MessageApp::ActivateCounter(x) => self.activate_recorder(x),
            MessageApp::CloseCounter(x) => self.deactivate_recorder(x),
            MessageApp::ToggleCounter(i) => {
                if let Some(r) = self.recorders.get_mut(i) {
                    r.activate_deactivate();
                }
            }
            MessageApp::CancelAddCounter => {
                self.window_add_counter = false;
                self.new_recorder_title.clear();
                self.title_error = false;
            }
            MessageApp::TitleInputChanged(value) => {
                self.new_recorder_title = value;
                self.title_error = false;
            }
            MessageApp::ConfirmAddCounter => {
                if self.new_recorder_title.trim().is_empty() {
                    self.title_error = true;
                } else {
                    self.recorders
                        .push(Recorder::new(self.new_recorder_title.clone()));
                    self.window_add_counter = false;
                    self.new_recorder_title.clear();
                    self.title_error = false;
                }
            }
        };
    }

    pub fn view(&self) -> Element<MessageApp> {
        let main = self.main_view();

        main
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

    pub fn activate_recorder(&mut self, index: usize) -> () {
        match self.recorders.get_mut(index) {
            None => (),
            Some(recorder) => recorder.activate(),
        }
    }

    pub fn deactivate_recorder(&mut self, index: usize) -> () {
        match self.recorders.get_mut(index) {
            None => (),
            Some(recorder) => recorder.deactivate(),
        }
    }

    pub fn delete_recorder(&mut self, index: usize) -> () {
        if index > self.recorders.len() {
            return;
        }
        self.recorders.remove(index);
    }

    pub fn main_view(&self) -> Element<MessageApp> {
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

        content =
            content.push(button("➕ Ajouter un recorder").on_press(MessageApp::WindowAddCounter));

        content.into()
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
