use crate::screens::add_recorder_screen::AddRecorderScreen;
use crate::screens::components::list::{ListComponent, ListMessage};
use crate::screens::settings_screen::SettingsScreen;
use crate::structs::app::{MessageApp, Screen};
use crate::structs::recorder::Recorder;
use crate::structs::settings::settings::Settings;
use crate::structs::storage::Storage;
use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{Color, Element, Length, Task};
use strsim::normalized_levenshtein;
use uuid::Uuid;
// -------------------------------------------------------
// Messages propres à la vue List
// -------------------------------------------------------
#[derive(Debug, Clone)]
pub enum MainScreenComponents {
    List(ListComponent),
}

#[derive(Debug, Clone)]
pub enum MainScreenMessage {
    List(ListMessage),
    BossesFoundOCR(Vec<String>),
    DeathDetected,
    ChangeView(Screen),
}

// -------------------------------------------------------
// État propre à la vue List
// -------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct MainScreen {
    list: ListComponent,
    ocr: OcrComponent,
    settings: Settings,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            list: ListComponent::new(),
            ocr: OcrComponent::new(),
            settings: Settings::load(),
        }
    }

    pub fn update(&mut self, message: MainScreenMessage) -> Task<MainScreenMessage> {
        match message {
            MainScreenMessage::ChangeView(_) => Task::none(),
            MainScreenMessage::List(message) => {
                self.list.update(message).map(MainScreenMessage::List)
            }
            MainScreenMessage::BossesFoundOCR(bosses) => {
                let bosses_names: String = bosses
                    .into_iter()
                    .filter(|b| !b.trim().is_empty())
                    .map(|b| b.trim().to_string())
                    .collect::<Vec<_>>()
                    .join(" - ");

                if !bosses_names.is_empty() {
                    self.handle_boss_death(bosses_names);
                }
                Task::none()
            }
            MainScreenMessage::DeathDetected => {
                println!("💀 Mort détectée ! Recherche des boss...");
                self.list.increment_global_deaths();
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, MainScreenMessage> {
        column![
            self.list.view().map(MainScreenMessage::List),
            row![
                button("Ajouter un compteur").on_press(MainScreenMessage::ChangeView(
                    crate::structs::app::Screen::AddRecorderScreen(AddRecorderScreen::new())
                )),
                button("Paramètres").on_press(MainScreenMessage::ChangeView(
                    crate::structs::app::Screen::SettingsScreen(SettingsScreen::new())
                ))
            ]
            .padding(10)
            .spacing(10)
        ]
        .into()
    }

    // --- Vue d'un compteur classique ---

    pub fn save(&self) {
        self.list.save();
    }

    fn handle_boss_death(&mut self, boss_name: String) {
        println!("⚔️  Mort contre : {}", boss_name);

        let global_count = self.list.recorders.iter().filter(|r| r.is_global()).count();
        let normalized_boss = boss_name.trim().to_uppercase();

        // 1. Chercher correspondance exacte d'abord
        if let Some(pos) = self
            .list
            .recorders
            .iter()
            .position(|r| r.is_classic() && r.get_title().to_uppercase() == normalized_boss)
        {
            let mut recorder = self.list.recorders.remove(pos);
            recorder.increment();
            self.list.recorders.insert(global_count, recorder);
            println!("✅ Compteur '{}' incrémenté (match exact)", boss_name);
        } else {
            // 2. Pas de match exact, chercher une similarité
            let similar = self.find_similar_boss(&normalized_boss, 0.80);

            match similar {
                Some((pos, similarity, existing_name)) => {
                    println!(
                        "🔍 Boss similaire trouvé: '{}' ~= '{}' ({}% similaire)",
                        normalized_boss,
                        existing_name,
                        (similarity * 100.0) as u32
                    );

                    let mut recorder = self.list.recorders.remove(pos);
                    recorder.increment();
                    self.list.recorders.insert(global_count, recorder);
                    println!(
                        "✅ Compteur '{}' incrémenté (match similaire)",
                        existing_name
                    );
                }
                None => {
                    // 3. Pas de match similaire : créer nouveau compteur
                    let mut new_recorder = Recorder::new(boss_name.clone());
                    new_recorder.force_increment();
                    self.list.recorders.insert(global_count, new_recorder);
                    println!("✅ Nouveau compteur '{}' créé", boss_name);
                }
            }
        }

        self.list.increment_global_bosses();
    }

    /// Trouve le boss le plus similaire dans la liste
    /// Retourne: Option<(position, score_similarité, nom_existant)>
    fn find_similar_boss(&self, boss_name: &str, threshold: f64) -> Option<(usize, f64, String)> {
        let mut best_match: Option<(usize, f64, String)> = None;

        for (i, recorder) in self.list.recorders.iter().enumerate() {
            if !recorder.is_classic() {
                continue; // Ignorer les globaux
            }

            let existing_name = recorder.get_title().to_uppercase();
            let similarity = normalized_levenshtein(boss_name, &existing_name);

            // Garder le meilleur match si au-dessus du seuil
            if similarity >= threshold {
                if let Some((_, best_score, _)) = &best_match {
                    if similarity > *best_score {
                        best_match = Some((i, similarity, recorder.get_title().to_string()));
                    }
                } else {
                    best_match = Some((i, similarity, recorder.get_title().to_string()));
                }
            }
        }

        best_match
    }
}
