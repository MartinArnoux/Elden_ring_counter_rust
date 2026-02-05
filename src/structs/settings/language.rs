use serde::{Deserialize, Serialize};
use std::fmt;

pub const ALL_LANGUAGES: &[Language] = &[Language::French];
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    French,
}
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::French => write!(f, "Français"),
        }
    }
}
impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Language::French, Language::French) => true,
        }
    }
}
