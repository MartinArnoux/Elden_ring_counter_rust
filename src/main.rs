use iced;
use iced::{Size, window};
mod hotkey;
mod ocr;
mod structs;
mod style;
mod utils;
use crate::structs::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .window(window::Settings {
            min_size: Some(Size::new(800.0, 600.0)),
            ..Default::default()
        })
        .run()
}
