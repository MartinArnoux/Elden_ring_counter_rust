use crate::hotkey::{GlobalHotkey, HotkeyError, HotkeyMessage, Key, Modifier};

use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone, Debug)]
pub struct WindowsHotkey {
    pub sender: UnboundedSender<HotkeyMessage>,
}

impl WindowsHotkey {
    pub fn new(sender: UnboundedSender<HotkeyMessage>) -> Self {
        Self { sender }
    }
}

impl Default for WindowsHotkey {
    fn default() -> Self {
        let (sender, _) = tokio::sync::mpsc::unbounded_channel();
        Self { sender }
    }
}

impl GlobalHotkey for WindowsHotkey {
    type Messages = UnboundedSender<HotkeyMessage>;

    fn register(&self, mods: &[Modifier], key: Key) -> Result<(), HotkeyError> {
        let mut win_mods = HOT_KEY_MODIFIERS(0);
        for m in mods {
            match m {
                Modifier::Alt => win_mods |= MOD_ALT,
            }
        }

        let vk = match key {
            Key::Plus => VK_ADD.0 as u32,
        };

        unsafe {
            match RegisterHotKey(None, 1, win_mods, vk) {
                Ok(_) => {
                    println!("Hotkey registered successfully");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Register failed: {:?}", e);
                    Err(HotkeyError::RegistrationFailed)
                }
            }
        }
    }

    fn event_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            println!("Starting hotkey event loop...");

            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_HOTKEY {
                    // Envoie le message via le channel tokio
                    // let _ = output

                    if let Err(e) = self.sender.send(HotkeyMessage::Increment) {
                        eprintln!("Failed to send hotkey message: {:?}", e);
                        break;
                    }
                }
            }

            println!("Hotkey event loop ended");
        }
    }
}
