#[macro_export]
macro_rules! lap {
    ($start:expr, $label:expr) => {{
        #[cfg(feature = "timing")]
        {
            let elapsed = $start.elapsed();
            println!("{:<30} {:>6} ms", $label, elapsed.as_millis());
        }
    }};
}
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
