use iced;

mod hotkey;
mod structs;
mod style;
use crate::structs::app::App;

fn main() -> iced::Result {
    iced::run(App::update, App::view)
}
