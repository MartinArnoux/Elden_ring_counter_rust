use crate::{
    screens::components::ocr::{ActionOCR, StatusOCR},
    structs::settings::settings::Settings,
};

use super::language::Language;
#[derive(Debug, Clone)]
pub struct I18n {
    pub language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
    pub fn load() -> Self {
        let settings = Settings::load();
        let language = settings.get_language();
        Self::new(language.clone())
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn get(&self, key: TranslationKey) -> &'static str {
        match self.language {
            Language::French => key.fr(),
            Language::English => key.en(),
        }
    }
    // Raccourcis pour chaque catégorie
    pub fn general(&self, key: GeneralKey) -> &'static str {
        self.get(TranslationKey::General(key))
    }

    pub fn ocr(&self, key: OcrKey) -> &'static str {
        self.get(TranslationKey::Ocr(key))
    }

    pub fn settings(&self, key: SettingsKey) -> &'static str {
        self.get(TranslationKey::Settings(key))
    }

    pub fn list(&self, key: ListKey) -> &'static str {
        self.get(TranslationKey::List(key))
    }
    pub fn add_recorder(&self, key: AddRecorderKey) -> &'static str {
        self.get(TranslationKey::AddRecorder(key))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TranslationKey {
    General(GeneralKey),
    Settings(SettingsKey),
    Ocr(OcrKey),
    List(ListKey),
    AddRecorder(AddRecorderKey),
}

#[derive(Debug, Clone, Copy)]
pub enum AddRecorderKey {
    Title,
    InputPlaceholder,
    AddCounter,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
pub enum GeneralKey {
    Delete,
    AddRecorder,
    Save,
    Settings,
}

#[derive(Debug, Clone, Copy)]
pub enum OcrKey {
    Starting,
    SearchingBossName,
    EndingAction,
    SearchingDeath,
    Stopped,
    AutoDetection,
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsKey {
    Language,
    Game,
    Screen,
    DeathText,
    DeathTextInput,
}

#[derive(Debug, Clone, Copy)]
pub enum ListKey {
    InputTextPlaceholder,
    TitleGlobalDeaths,
    TitleGlobalCounter,
}

impl TranslationKey {
    pub fn fr(self) -> &'static str {
        match self {
            TranslationKey::General(key) => match key {
                GeneralKey::Delete => "Supprimer",
                GeneralKey::Save => "Enregistrer",
                GeneralKey::Settings => "Paramètres",
                GeneralKey::AddRecorder => "Ajouter un enregistreur",
            },
            TranslationKey::Settings(key) => match key {
                SettingsKey::Language => "Langue",
                SettingsKey::Game => "Jeu",
                SettingsKey::Screen => "Écran",
                SettingsKey::DeathText => "Texte de mort",
                SettingsKey::DeathTextInput => "Texte de Mort",
            },

            TranslationKey::Ocr(key) => match key {
                OcrKey::Starting => "Démarrage...",
                OcrKey::SearchingBossName => "Recherche du nom du boss...",
                OcrKey::EndingAction => "OCR démarré - pause en cours",
                OcrKey::SearchingDeath => "Recherche de ta mort... ça arrive",
                OcrKey::Stopped => "OCR arrêté",
                OcrKey::AutoDetection => "OCR Auto-détection :",
            },
            TranslationKey::List(key) => match key {
                ListKey::InputTextPlaceholder => "Entrer le titre",
                ListKey::TitleGlobalDeaths => "Morts Totales",
                ListKey::TitleGlobalCounter => "VS Boss",
            },
            TranslationKey::AddRecorder(key) => match key {
                AddRecorderKey::Title => "Ajouter un enregistreur",
                AddRecorderKey::InputPlaceholder => "Titre",
                AddRecorderKey::AddCounter => "Ajouter",
                AddRecorderKey::Cancel => "Annuler",
            },
        }
    }

    pub fn en(self) -> &'static str {
        match self {
            TranslationKey::General(key) => match key {
                GeneralKey::Delete => "Delete",
                GeneralKey::Save => "Save",
                GeneralKey::Settings => "Settings",
                GeneralKey::AddRecorder => "Add recorder",
            },
            TranslationKey::Settings(key) => match key {
                SettingsKey::Language => "Language",
                SettingsKey::DeathTextInput => "Death text",
                SettingsKey::DeathText => "Death text",
                SettingsKey::Game => "Game",
                SettingsKey::Screen => "Screen",
            },

            TranslationKey::Ocr(key) => match key {
                OcrKey::Starting => "Starting...",
                OcrKey::SearchingBossName => "Searching boss name...",
                OcrKey::EndingAction => "OCR started - sleeping",
                OcrKey::SearchingDeath => "Searching your death...",
                OcrKey::Stopped => "OCR stopped",
                OcrKey::AutoDetection => "OCR Auto-detection :",
            },
            TranslationKey::List(key) => match key {
                ListKey::InputTextPlaceholder => "Enter the title",
                ListKey::TitleGlobalDeaths => "Deaths VS Boss",
                ListKey::TitleGlobalCounter => "Global Deaths",
            },
            TranslationKey::AddRecorder(key) => match key {
                AddRecorderKey::Title => "Add Recorder",
                AddRecorderKey::InputPlaceholder => "Title",
                AddRecorderKey::AddCounter => "Add",
                AddRecorderKey::Cancel => "Cancel",
            },
        }
    }
}

impl From<StatusOCR> for TranslationKey {
    fn from(status: StatusOCR) -> Self {
        match status {
            StatusOCR::Starting => TranslationKey::Ocr(OcrKey::Starting),
            StatusOCR::Stopped => TranslationKey::Ocr(OcrKey::Stopped),
            StatusOCR::Started(action) => match action {
                ActionOCR::SearchingBossName => TranslationKey::Ocr(OcrKey::SearchingBossName),
                ActionOCR::EndingAction => TranslationKey::Ocr(OcrKey::EndingAction),
                ActionOCR::SearchingDeath => TranslationKey::Ocr(OcrKey::SearchingDeath),
            },
        }
    }
}
