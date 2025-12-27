use std::sync::Arc;

use iced::futures::lock::Mutex;

use crate::{
    hotkey::{GlobalHotkey, HotkeyError, Key, Modifier},
    structs::app::MessageApp,
};

#[derive(Clone)]
pub struct WaylandHotkey {
    messages: Arc<Mutex<Vec<MessageApp>>>,
}

impl GlobalHotkey for WaylandHotkey {
    type Messages = Arc<Mutex<Vec<MessageApp>>>;
    fn get_mut_messages(&mut self) -> &mut Self::Messages {
        &mut self.messages
    }
    fn register(&self, mods: &[Modifier], key: Key) -> Result<(), HotkeyError>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn event_loop(&self) {}
}
