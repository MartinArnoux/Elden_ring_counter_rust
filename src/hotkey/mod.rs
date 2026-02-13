#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsHotkey;

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Alt,
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Plus,
}

#[derive(Debug)]
pub enum HotkeyError {
    RegistrationFailed,
}

#[derive(Debug, Clone)]
pub enum HotkeyMessage {
    Increment,
}

pub trait GlobalHotkey: Clone {
    type Messages;
    fn register(&self, mods: &[Modifier], key: Key) -> Result<(), HotkeyError>
    where
        Self: Sized;

    fn event_loop(&self);
    //fn get_mut_messages(&mut self) -> &mut Self::Messages;
}
