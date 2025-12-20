use crate::structs::recorder::Recorder;
use iced::widget::column;
use iced::{Element, Task};

pub enum MessageApp {
    Increment,
    AddCounter,
    WindowAddCounter,
    ActivateCounter(usize),
}
#[derive(Default)]
pub struct App {
    recorders: Vec<Recorder>,
    window_add_counter: bool,
}

impl App {
    pub fn new() -> App {
        App {
            recorders: Vec::new(),
            window_add_counter: false,
        }
    }

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::Increment => self.update_all_counter(),
            MessageApp::AddCounter => (),
            MessageApp::WindowAddCounter => self.window_add_counter = true,
            MessageApp::ActivateCounter(x) => (),
        };
    }

    fn update_all_counter(&mut self) {
        // itération mutable sur le Vec
        for recorder in self.recorders.iter_mut() {
            recorder.increment();
        }
    }

    pub fn view(&self) -> Element<MessageApp> {
        column![].into()
    }

    //RegisterHotKey(None, hotkey_id, MOD_CONTROL, VK_ADD.0 as u32)
    // unsafe {
    //     // ID arbitraire du raccourci
    //     let hotkey_id = 1;

    //     // Ctrl + '+'

    //     RegisterHotKey(None, hotkey_id, MOD_CONTROL, VK_ADD.0 as u32)
    //         .expect("RegisterHotKey failed");

    //     println!("Raccourci Ctrl + '+' enregistré. Appuie pour incrémenter.");
    //     println!("Ctrl + C pour quitter.");

    //     let mut count = 0;
    //     let mut msg = MSG::default();

    //     while GetMessageW(&mut msg, None, 0, 0).into() {
    //         if msg.message == WM_HOTKEY && msg.wParam.0 == hotkey_id as usize {
    //             count += 1;
    //             println!("Compteur = {}", count);
    //         }
    //     }

    //     UnregisterHotKey(None, hotkey_id);
    // }
}
