use super::crop_position::CropPosition;
use serde::{Deserialize, Serialize};
use std::fmt;
pub const ALL_GAMES: &[Game] = &[Game::EldenRing];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Hash)]
pub enum Game {
    EldenRing,
}
impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Game::EldenRing, Game::EldenRing) => true,
        }
    }
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Game::EldenRing => write!(f, "Elden Ring"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct GameConfig {
    /// Zone pour détecter "You Died"
    death_zone: CropPosition,

    /// Zones pour détecter les noms de boss (peut y en avoir plusieurs)
    boss_zones: Vec<CropPosition>,
}

impl GameConfig {
    pub fn elden_ring_default() -> Self {
        Self {
            death_zone: CropPosition::new(31, 46, 39, 10),
            boss_zones: vec![
                CropPosition::new(25, 30, 50, 15), // Zone principale du boss
                                                   // Ajoutez d'autres zones si nécessaire
            ],
        }
    }

    pub fn dark_souls_3_default() -> Self {
        Self {
            death_zone: CropPosition::new(30, 45, 40, 12),
            boss_zones: vec![CropPosition::new(20, 25, 60, 20)],
        }
    }

    pub fn get_death_zone(&self) -> &CropPosition {
        &self.death_zone
    }
    pub fn get_boss_zones(&self) -> &Vec<CropPosition> {
        &self.boss_zones
    }
}
