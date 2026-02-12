use iced::window::settings;

use crate::structs::settings::settings::Settings;

#[derive(Clone, Debug)]
pub enum ActionOCR {
    SearchingDeath,
    SearchingBossName,
    EndingAction,
}

#[derive(Clone, Debug)]
pub enum StatusOCR {
    Starting,
    Started(ActionOCR),
    Stopped,
}

pub enum OcrMessage {
    ActivateOCR(bool),
    StartingOCR,
    OCROK,
    ChangeActionOCR(ActionOCR),
}

pub struct OcrComponent {
    settings: Settings,
    ocr_activate: bool,
    ocr_status: StatusOCR,
}

impl OcrComponent {
    pub fn new() -> Self {
        Self {
            settings: Settings::load(),
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
        }
    }

    pub fn update(&mut self, message: OcrMessage) {
        match message {
            OcrMessage::ActivateOCR(active) => {
                self.set_ocr_active(active);
            }
            OcrMessage::StartingOCR => {
                // Start OCR process
            }
            OcrMessage::OCROK => {
                // OCR completed successfully
            }
            OcrMessage::ChangeActionOCR(action) => {
                self.set_ocr_action(action);
            }
        }
    }

    pub fn view(&self) -> iced::Element<OcrMessage> {
        // Implement view logic here
    }

    pub fn set_ocr_active(&mut self, active: bool) {
        self.ocr_activate = active;
    }

    pub fn set_ocr_action(&mut self, action: ActionOCR) {
        self.ocr_status = StatusOCR::Started(action);
    }
}
