use iced;

mod hotkey;
mod structs;
mod style;
use crate::structs::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}
