#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsHotkey;

#[cfg(all(target_os = "linux", not(feature = "wayland")))]
mod linux_x11;
#[cfg(all(target_os = "linux", not(feature = "wayland")))]
pub use linux_x11::X11Hotkey;

#[cfg(all(target_os = "linux", feature = "wayland"))]
mod linux_wayland;
#[cfg(all(target_os = "linux", feature = "wayland"))]
pub use linux_wayland::WaylandHotkey;

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Plus,
    Char(char),
}

#[derive(Debug)]
pub enum HotkeyError {
    UnsupportedPlatform,
    RegistrationFailed,
}

pub trait GlobalHotkey {
    fn register(mods: &[Modifier], key: Key, callback: fn()) -> Result<Self, HotkeyError>
    where
        Self: Sized;

    fn event_loop(&self);
}
