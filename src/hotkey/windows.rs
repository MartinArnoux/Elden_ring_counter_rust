use crate::hotkey::{GlobalHotkey, HotkeyError, Key, Modifier};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct WindowsHotkey {
    id: i32,
    callback: fn(),
}

impl GlobalHotkey for WindowsHotkey {
    fn register(mods: &[Modifier], key: Key, callback: fn()) -> Result<Self, HotkeyError> {
        let mut win_mods = HOT_KEY_MODIFIERS(0);

        for m in mods {
            match m {
                Modifier::Ctrl => win_mods |= MOD_CONTROL,
                Modifier::Alt => win_mods |= MOD_ALT,
                Modifier::Shift => win_mods |= MOD_SHIFT,
            }
        }

        let vk = match key {
            Key::Plus => VK_OEM_PLUS.0 as u32,
            Key::Char(c) => c as u32,
        };

        unsafe {
            RegisterHotKey(None, 1, win_mods, vk).map_err(|_| HotkeyError::RegistrationFailed)?;
            println!("Register")
        }

        Ok(Self { id: 1, callback })
    }

    fn event_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_HOTKEY {
                    (self.callback)();
                }
            }
        }
    }
}
