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
mod i18n;
mod ocr;
mod screens;
mod structs;
mod style;
mod utils;
use crate::structs::app::App;
use crate::structs::storage::Storage;

fn main() -> iced::Result {
    if let Err(e) = Storage::ensure_migrated() {
        eprintln!("Erreur migration: {}", e);
    }
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .window(window::Settings {
            min_size: Some(Size::new(800.0, 600.0)),
            ..Default::default()
        })
        .run()
}
