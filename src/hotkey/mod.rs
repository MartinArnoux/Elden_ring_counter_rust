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

use crate::hotkey::linux_wayland::WaylandHotkey;

mod linux_wayland;

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

#[derive(Clone)]
pub enum PlateformHotKey {
    Windows(WindowsHotkey),
    Wayland(WaylandHotkey),
}

impl Default for PlateformHotKey {
    fn default() -> Self {
        PlateformHotKey::Windows(WindowsHotkey::default())
    }
}

#[derive(Debug)]
pub enum HotkeyError {
    UnsupportedPlatform,
    RegistrationFailed,
}

pub trait GlobalHotkey: Clone {
    type Messages;
    fn register(&self, mods: &[Modifier], key: Key) -> Result<(), HotkeyError>
    where
        Self: Sized;

    fn event_loop(&self);
    fn get_mut_messages(&mut self) -> &mut Self::Messages;
}
