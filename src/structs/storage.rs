use super::recorder::Recorder;
use super::settings::settings::Settings;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct Storage;

impl Storage {
    // Fonction pour obtenir le répertoire de l'application
    fn get_app_dir() -> Result<PathBuf, String> {
        ProjectDirs::from("", "", "DeathCompteur")
            .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
            .ok_or_else(|| "Impossible de déterminer le répertoire de données".to_string())
    }

    // Créer le répertoire s'il n'existe pas
    fn ensure_app_dir() -> Result<PathBuf, String> {
        let app_dir = Self::get_app_dir()?;
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        Ok(app_dir)
    }

    // Chemins des fichiers
    fn recorders_path() -> Result<PathBuf, String> {
        let mut path = Self::ensure_app_dir()?;
        path.push("recorders.json");
        Ok(path)
    }

    fn settings_path() -> Result<PathBuf, String> {
        let mut path = Self::ensure_app_dir()?;
        path.push("settings.json");
        Ok(path)
    }

    pub fn save_recorders(recorders: &Vec<Recorder>) -> Result<(), String> {
        let path = Self::recorders_path()?;
        println!("path recorder : {:?}", path);
        let json = serde_json::to_string_pretty(recorders).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_recorders() -> Result<Vec<Recorder>, String> {
        let path = Self::recorders_path()?;

        if !path.exists() {
            println!("Aucun fichier de sauvegarde trouvé");
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let recorders = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        Ok(recorders)
    }

    pub fn save_settings(settings: &Settings) -> Result<(), String> {
        let path = Self::settings_path()?;
        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_settings() -> Result<Settings, String> {
        let path = Self::settings_path()?;

        if !path.exists() {
            println!("Aucun fichier de paramètres trouvé, utilisation des valeurs par défaut");
            return Ok(Settings::default());
        }

        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let settings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        Ok(settings)
    }
}
