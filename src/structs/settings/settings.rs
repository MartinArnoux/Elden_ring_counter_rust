// src/structs/settings/settings.rs
use super::game::{Game, GameConfig};
use crate::{i18n::language::Language, structs::storage::Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    game: Game,
    screen: i8,
    language: Language,
    death_text: String,

    /// Configurations personnalisées par jeu
    #[serde(default)]
    pub custom_game_configs: HashMap<Game, GameConfig>,
}

impl Settings {
    /// Obtenir la config du jeu actuel
    pub fn get_game_config(&self) -> GameConfig {
        // D'abord chercher une config personnalisée
        if let Some(custom) = self.custom_game_configs.get(&self.game) {
            return custom.clone();
        }

        // Sinon, utiliser les valeurs par défaut
        Self::default_game_config(&self.game)
    }

    /// Configurations par défaut pour chaque jeu
    fn default_game_config(game: &Game) -> GameConfig {
        match game {
            Game::EldenRing => GameConfig::elden_ring_default(),
            // Game::DarkSouls3 => GameConfig::dark_souls_3_default(),
        }
    }

    /// Définir une configuration personnalisée
    // pub fn set_custom_game_config(&mut self, game: Game, config: GameConfig) {
    //     self.custom_game_configs.insert(game, config);
    // }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
    }
    pub fn get_game(&self) -> Game {
        self.game
    }

    pub fn set_screen(&mut self, screen: i8) {
        self.screen = screen;
    }
    pub fn get_screen(&self) -> i8 {
        self.screen
    }

    pub fn get_language(&self) -> &Language {
        &self.language
    }
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }
    pub fn set_death_text(&mut self, death_text: String) {
        self.death_text = death_text;
    }
    pub fn get_death_text(&self) -> &String {
        &self.death_text
    }
    pub fn load() -> Self {
        Storage::load_settings().unwrap_or_default()
    }
    pub fn save(&self) {
        let _ = Storage::save_settings(self);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            game: Game::EldenRing,
            screen: 0,
            language: Language::French,
            custom_game_configs: HashMap::new(),
            death_text: String::from("VOUS AVEZ PERI"),
        }
    }
}
