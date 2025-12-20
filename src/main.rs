use iced;

mod hotkey;
mod structs;
use crate::structs::app::App;

fn main() -> iced::Result {
    iced::run(App::update, App::view)
}
