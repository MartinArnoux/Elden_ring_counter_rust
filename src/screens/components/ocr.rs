use std::{fmt, time::Duration};

use iced::{
    Color, Subscription, Task,
    widget::{column, row, text, toggler},
    window::settings,
};
use strsim::normalized_levenshtein;

use crate::{
    structs::settings::settings::Settings,
    utils::app_worker::{hotkey_subscription, ocr_subscription},
};

#[derive(Clone, Debug)]
pub enum ActionOCR {
    SearchingDeath,
    SearchingBossName,
    EndingAction,
}
impl fmt::Display for ActionOCR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ActionOCR::SearchingDeath => "Recherche de ta mort... ça arrive",
            ActionOCR::SearchingBossName => "Recherche du nom du boss...",
            ActionOCR::EndingAction => "OCR démarré - sleeping a bit",
        };

        write!(f, "{text}")
    }
}

#[derive(Clone, Debug)]
pub enum StatusOCR {
    Starting,
    Started(ActionOCR),
    Stopped,
}

impl StatusOCR {
    pub fn color(&self) -> Color {
        match self {
            StatusOCR::Starting => Color::from_rgb(1.0, 0.65, 0.0),
            StatusOCR::Started(_) => Color::from_rgb(0.0, 0.8, 0.0),
            StatusOCR::Stopped => Color::from_rgb(0.6, 0.6, 0.6),
        }
    }
    pub fn show_spinner(&self) -> bool {
        matches!(self, StatusOCR::Started(ActionOCR::SearchingBossName))
    }
}

impl fmt::Display for StatusOCR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            StatusOCR::Starting => "Démarrage...",
            StatusOCR::Started(action) => return write!(f, "{action}"),
            StatusOCR::Stopped => "OCR arrêté",
        };

        write!(f, "{text}")
    }
}

#[derive(Clone, Debug)]
pub enum OcrMessage {
    ActivateOCR(bool),
    ChangeActionOCR(StatusOCR),
    BossesFoundOCR(Vec<String>),
    DeathDetected,
}

#[derive(Clone, Debug)]
pub struct OcrComponent {
    settings: Settings,
    ocr_activate: bool,
    ocr_status: StatusOCR,
}
impl Default for OcrComponent {
    fn default() -> Self {
        Self {
            settings: Settings::load(),
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
        }
    }
}

impl OcrComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: OcrMessage) -> Task<OcrMessage> {
        match message {
            OcrMessage::ActivateOCR(active) => {
                if active {
                    self.ocr_status = StatusOCR::Starting;
                } else {
                    self.ocr_status = StatusOCR::Stopped;
                }

                self.set_ocr_active(active);
                Task::none()
            }

            OcrMessage::ChangeActionOCR(action) => {
                match action {
                    StatusOCR::Started(action) => {
                        self.set_ocr_action(action);
                    }
                    _ => {}
                }
                Task::none()
            }
            OcrMessage::BossesFoundOCR(bosses) => {
                let bosses_names: String = bosses
                    .into_iter()
                    .filter(|b| !b.trim().is_empty())
                    .map(|b| b.trim().to_string())
                    .collect::<Vec<_>>()
                    .join(" - ");

                if !bosses_names.is_empty() {
                    //self.handle_boss_death(bosses_names);
                }
                Task::none()
            }
            OcrMessage::DeathDetected => {
                println!("💀 Mort détectée ! Recherche des boss...");
                //self.list.increment_global_deaths();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<OcrMessage> {
        column![
            row![
                text("OCR Auto-détection :"),
                toggler(self.ocr_activate).on_toggle(OcrMessage::ActivateOCR)
            ]
            .spacing(10),
            // Texte du statut OCR avec couleur
            text(self.ocr_status.to_string())
                .color(self.ocr_status.color())
                .size(16)
        ]
        .spacing(10)
        .into()
    }

    pub fn subscription(&self) -> Subscription<OcrMessage> {
        // ✅ Conditionnellement créer la subscription OCR
        let ocr_sub = if self.ocr_activate {
            ocr_subscription(
                self.settings.get_screen(),
                self.settings.get_game_config(),
                self.settings.get_death_text().clone(),
            )
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![ocr_sub])
    }

    pub fn set_ocr_active(&mut self, active: bool) {
        self.ocr_activate = active;
    }

    pub fn set_ocr_action(&mut self, action: ActionOCR) {
        self.ocr_status = StatusOCR::Started(action);
    }
}
