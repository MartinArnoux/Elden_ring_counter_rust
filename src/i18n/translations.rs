use crate::screens::components::ocr::{ActionOCR, StatusOCR};

use super::language::Language;
#[derive(Debug, Clone)]
pub struct I18n {
    pub language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
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
}

#[derive(Debug, Clone, Copy)]
pub enum TranslationKey {
    General(GeneralKey),
    Ocr(OcrKey),
}

#[derive(Debug, Clone, Copy)]
pub enum GeneralKey {
    Title,
    TotalDeaths,
    TotalBossDeaths,
    Delete,
    Welcome,
    ChangeLanguage,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum OcrKey {
    Starting,
    SearchingBossName,
    EndingAction,
    SearchingDeath,
    Stopped,
}

impl TranslationKey {
    pub fn fr(self) -> &'static str {
        match self {
            TranslationKey::General(key) => match key {
                GeneralKey::Title => "Mon Application",
                GeneralKey::TotalDeaths => "Total des morts",
                GeneralKey::TotalBossDeaths => "Total des morts contre les boss",
                GeneralKey::Delete => "Supprimer",
                GeneralKey::Welcome => "Bienvenue",
                GeneralKey::ChangeLanguage => "Changer la langue",
                GeneralKey::Quit => "Quitter",
            },

            TranslationKey::Ocr(key) => match key {
                OcrKey::Starting => "Démarrage...",
                OcrKey::SearchingBossName => "Recherche du nom du boss...",
                OcrKey::EndingAction => "OCR démarré - pause en cours",
                OcrKey::SearchingDeath => "Recherche de ta mort...",
                OcrKey::Stopped => "OCR arrêté",
            },
        }
    }

    pub fn en(self) -> &'static str {
        match self {
            TranslationKey::General(key) => match key {
                GeneralKey::Title => "My Application",
                GeneralKey::TotalDeaths => "Total deaths",
                GeneralKey::TotalBossDeaths => "Total boss deaths",
                GeneralKey::Delete => "Delete",
                GeneralKey::Welcome => "Welcome",
                GeneralKey::ChangeLanguage => "Change language",
                GeneralKey::Quit => "Quit",
            },

            TranslationKey::Ocr(key) => match key {
                OcrKey::Starting => "Starting...",
                OcrKey::SearchingBossName => "Searching boss name...",
                OcrKey::EndingAction => "OCR started - sleeping",
                OcrKey::SearchingDeath => "Searching your death...",
                OcrKey::Stopped => "OCR stopped",
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
