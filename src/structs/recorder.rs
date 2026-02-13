use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecorderType {
    Classic,      // Compteur normal (boss spécifique)
    GlobalDeaths, // Compteur global de toutes les morts
    GlobalBosses, // Compteur global de morts contre des boss uniquement
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recorder {
    uuid: Uuid,
    title: String,
    counter: u32,
    active: bool,
    recorder_type: RecorderType,
}

impl Recorder {
    pub fn new(title: String) -> Recorder {
        Recorder {
            uuid: Uuid::new_v4(),
            title,
            counter: 0,
            active: true,
            recorder_type: RecorderType::Classic,
        }
    }
    // Créer le compteur global des morts
    pub fn new_global_deaths() -> Self {
        Recorder {
            title: "💀 MORTS TOTALES".to_string(),
            counter: 0,
            uuid: Uuid::from_u128(1), // UUID spécial
            active: true,
            recorder_type: RecorderType::GlobalDeaths,
        }
    }

    // Créer le compteur global des morts contre boss
    pub fn new_global_bosses() -> Self {
        Recorder {
            title: "⚔️ MORTS CONTRE BOSS".to_string(),
            counter: 0,
            uuid: Uuid::from_u128(2), // UUID spécial
            active: true,
            recorder_type: RecorderType::GlobalBosses,
        }
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn increment(&mut self) -> () {
        if self.active {
            self.counter += 1;
        }
    }
    pub fn force_increment(&mut self) -> () {
        self.counter += 1;
    }
    pub fn force_decrement(&mut self) -> () {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }
    pub fn get_counter(&self) -> u32 {
        self.counter
    }
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn activate_deactivate(&mut self) {
        self.active = !self.active
    }

    pub fn get_status_recorder(&self) -> bool {
        self.active
    }

    pub fn get_type(&self) -> &RecorderType {
        &self.recorder_type
    }

    pub fn is_global_deaths(&self) -> bool {
        self.recorder_type == RecorderType::GlobalDeaths
    }

    pub fn is_global_bosses(&self) -> bool {
        self.recorder_type == RecorderType::GlobalBosses
    }

    pub fn is_classic(&self) -> bool {
        self.recorder_type == RecorderType::Classic
    }

    pub fn from_db(
        uuid_string: String,
        title: String,
        counter: u32,
        is_active: bool,
        recorder_type: RecorderType,
    ) -> Self {
        let uuid = Uuid::parse_str(&uuid_string).unwrap();
        Recorder {
            uuid,
            title,
            counter,
            active: is_active,
            recorder_type: recorder_type,
        }
    }
}

impl RecorderType {
    pub fn to_db_str(&self) -> &str {
        match self {
            RecorderType::Classic => "Classic",
            RecorderType::GlobalDeaths => "GlobalDeaths",
            RecorderType::GlobalBosses => "GlobalBosses",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "GlobalDeaths" => RecorderType::GlobalDeaths,
            "GlobalBosses" => RecorderType::GlobalBosses,
            _ => RecorderType::Classic, // Valeur par défaut
        }
    }
}

////TEST////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_recorder_is_active() {
        let r = Recorder::new("Test".to_string());
        assert!(r.get_status_recorder());
        assert_eq!(r.get_counter(), 0);
    }

    #[test]
    fn increment_only_when_active() {
        let mut r = Recorder::new("Test".to_string());

        r.increment();
        r.activate_deactivate();
        r.increment();

        assert_eq!(r.get_counter(), 1);
    }

    #[test]
    fn activate_deactivate_works() {
        let mut r = Recorder::new("Test".to_string());

        r.activate_deactivate();
        assert!(!r.get_status_recorder());

        r.activate_deactivate();
        assert!(r.get_status_recorder());
    }
}
