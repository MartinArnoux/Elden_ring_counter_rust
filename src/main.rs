#![windows_subsystem = "windows"]
use iced;

mod hotkey;
mod ocr;
mod structs;
mod style;
use crate::structs::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
