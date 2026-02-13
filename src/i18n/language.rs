use serde::{Deserialize, Serialize};
use std::fmt;

pub const ALL_LANGUAGES: &[Language] = &[Language::French, Language::English];
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    French,
    English,
}
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::French => write!(f, "Français"),
            Language::English => write!(f, "English"),
        }
    }
}
impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Language::French, Language::French) => true,
            (Language::English, Language::English) => true,
            _ => false,
        }
    }
}
