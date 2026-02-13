use crate::structs::recorder::RecorderType;

use super::recorder::Recorder;
use super::settings::settings::Settings;
use directories::ProjectDirs;
use rusqlite::{Connection, Result as SqlResult};
use serde::Deserialize;
use std::{fs, path::PathBuf};

pub struct Storage;

#[derive(Deserialize)]
struct LegacyData {
    recorders: Vec<Recorder>,
    settings: Settings,
}
impl Storage {
    // Obtenir le chemin de la base de données
    fn get_db_path() -> Result<PathBuf, String> {
        let proj_dirs = ProjectDirs::from("", "", "DeathCompteur")
            .ok_or_else(|| "Impossible de déterminer le répertoire de données".to_string())?;

        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;

        Ok(data_dir.join("deathcompteur.db"))
    }

    // Ouvrir la connexion et initialiser les tables
    fn open() -> Result<Connection, String> {
        let path = Self::get_db_path()?;
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Self::init_tables(&conn)?;
        Ok(conn)
    }

    // Créer les tables si elles n'existent pas
    fn init_tables(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS recorders (
                uuid            TEXT PRIMARY KEY,
                title           TEXT NOT NULL,
                counter         INTEGER NOT NULL DEFAULT 0,
                is_active       INTEGER NOT NULL DEFAULT 0,
                position        INTEGER NOT NULL DEFAULT 0,
                recorder_type   TEXT NOT NULL DEFAULT 'Classic'
            );

            CREATE TABLE IF NOT EXISTS settings (
                key     TEXT PRIMARY KEY,
                value   TEXT NOT NULL
            );
        ",
        )
        .map_err(|e| e.to_string())?;

        // Migrations : ajoutées au fur et à mesure des évolutions du schéma
        // let _ = conn.execute_batch(
        //     "ALTER TABLE recorders ADD COLUMN recorder_type TEXT NOT NULL DEFAULT 'Classic';"
        // );

        Ok(())
    }

    // -------------------------
    // Recorders
    // -------------------------

    pub fn save_recorders(recorders: &Vec<Recorder>) -> Result<(), String> {
        let conn = Self::open()?;

        conn.execute("DELETE FROM recorders", [])
            .map_err(|e| e.to_string())?;

        for (i, recorder) in recorders.iter().enumerate() {
            conn.execute(
                "INSERT INTO recorders (uuid, title, counter, is_active, position, recorder_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    recorder.get_uuid().to_string(),
                    recorder.get_title(),
                    recorder.get_counter(),
                    recorder.get_status_recorder() as i32,
                    i as i32,
                    recorder.get_type().to_db_str()
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn load_recorders() -> Result<Vec<Recorder>, String> {
        let conn = Self::open()?;

        let mut stmt = conn
            .prepare("SELECT uuid, title, counter, is_active, recorder_type FROM recorders ORDER BY position ASC")
            .map_err(|e| e.to_string())?;

        let recorders = stmt
            .query_map([], |row| {
                let uuid_str: String = row.get(0)?;
                let title: String = row.get(1)?;
                let counter: u32 = row.get(2)?;
                let is_active: i32 = row.get(3)?;
                let recorder_type: String = row.get(4)?;
                Ok((uuid_str, title, counter, is_active, recorder_type))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .map(|(uuid_str, title, counter, is_active, recorder_type)| {
                Recorder::from_db(
                    uuid_str,
                    title,
                    counter,
                    is_active != 0,
                    RecorderType::from_db_str(&recorder_type),
                )
            })
            .collect();

        Ok(recorders)
    }

    pub fn save_recorder(recorder: &Recorder, position: usize) -> Result<(), String> {
        let conn = Self::open()?;

        conn.execute(
            "INSERT INTO recorders (uuid, title, counter, is_active, position, recorder_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(uuid) DO UPDATE SET
                title = excluded.title,
                counter = excluded.counter,
                is_active = excluded.is_active,
                position = excluded.position,
                recorder_type = excluded.recorder_type",
            rusqlite::params![
                recorder.get_uuid().to_string(),
                recorder.get_title(),
                recorder.get_counter(),
                recorder.get_status_recorder() as i32,
                position as i32,
                recorder.get_type().clone().to_db_str()
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
    pub fn insert_recorder_at_first_position(recorder: &Recorder) -> Result<(), String> {
        let mut conn = Self::open().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 1️⃣ Décaler toutes les positions existantes
        tx.execute("UPDATE recorders SET position = position + 1", [])
            .map_err(|e| e.to_string())?;

        // 2️⃣ Insérer le nouveau recorder en position 0
        tx.execute(
            "INSERT INTO recorders (uuid, title, counter, is_active, position, recorder_type)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)
             ON CONFLICT(uuid) DO UPDATE SET
                title = excluded.title,
                counter = excluded.counter,
                is_active = excluded.is_active,
                position = 0,
                recorder_type = excluded.recorder_type",
            rusqlite::params![
                recorder.get_uuid().to_string(),
                recorder.get_title(),
                recorder.get_counter(),
                recorder.get_status_recorder() as i32,
                recorder.get_type().clone().to_db_str()
            ],
        )
        .map_err(|e| e.to_string())?;

        // 3️⃣ Commit transaction
        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn delete_recorder(uuid: &str) -> Result<(), String> {
        let conn = Self::open()?;
        conn.execute("DELETE FROM recorders WHERE uuid = ?1", [uuid])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // -------------------------
    // Settings
    // -------------------------

    pub fn save_settings(settings: &Settings) -> Result<(), String> {
        let conn = Self::open()?;
        let json = serde_json::to_string(settings).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('settings', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [json],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn load_settings() -> Result<Settings, String> {
        let conn = Self::open()?;

        let result: SqlResult<String> = conn.query_row(
            "SELECT value FROM settings WHERE key = 'settings'",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
            Err(_) => Ok(Settings::default()),
        }
    }

    // Obtenir les chemins des anciens fichiers JSON
    fn get_legacy_recorders_path() -> Result<PathBuf, String> {
        let proj_dirs = ProjectDirs::from("", "", "DeathCompteur")
            .ok_or_else(|| "Impossible de déterminer le répertoire de données".to_string())?;

        Ok(proj_dirs.data_dir().join("recorders.json"))
    }

    fn get_legacy_settings_path() -> Result<PathBuf, String> {
        let proj_dirs = ProjectDirs::from("", "", "DeathCompteur")
            .ok_or_else(|| "Impossible de déterminer le répertoire de données".to_string())?;

        Ok(proj_dirs.data_dir().join("settings.json"))
    }

    // Migrer les recorders
    fn migrate_recorders() -> Result<bool, String> {
        let json_path = Self::get_legacy_recorders_path()?;

        if !json_path.exists() {
            return Ok(false); // Pas de fichier à migrer
        }

        println!("Migration des recorders depuis {:?}...", json_path);

        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Erreur lecture recorders.json: {}", e))?;

        let recorders: Vec<Recorder> = serde_json::from_str(&json_content)
            .map_err(|e| format!("Erreur parsing recorders.json: {}", e))?;

        Self::save_recorders(&recorders)?;

        // Backup
        let backup_path = json_path.with_extension("json.backup");
        fs::rename(&json_path, &backup_path)
            .map_err(|e| format!("Erreur backup recorders: {}", e))?;

        println!("✓ Recorders migrés ({} enregistrements)", recorders.len());
        Ok(true)
    }

    // Migrer les settings
    fn migrate_settings() -> Result<bool, String> {
        let json_path = Self::get_legacy_settings_path()?;

        if !json_path.exists() {
            return Ok(false); // Pas de fichier à migrer
        }

        println!("Migration des settings depuis {:?}...", json_path);

        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Erreur lecture settings.json: {}", e))?;

        let settings: Settings = serde_json::from_str(&json_content)
            .map_err(|e| format!("Erreur parsing settings.json: {}", e))?;

        Self::save_settings(&settings)?;

        // Backup
        let backup_path = json_path.with_extension("json.backup");
        fs::rename(&json_path, &backup_path)
            .map_err(|e| format!("Erreur backup settings: {}", e))?;

        println!("✓ Settings migrés");
        Ok(true)
    }

    // Migration complète
    pub fn migrate_from_json() -> Result<(), String> {
        let recorders_migrated = Self::migrate_recorders()?;
        let settings_migrated = Self::migrate_settings()?;

        if recorders_migrated || settings_migrated {
            println!("✓ Migration terminée avec succès !");
        }

        Ok(())
    }

    // Vérifier et exécuter la migration si nécessaire
    pub fn ensure_migrated() -> Result<(), String> {
        let db_path = Self::get_db_path()?;

        // Si la DB n'existe pas, tenter la migration
        if !db_path.exists() {
            Self::migrate_from_json()?;
        }

        Ok(())
    }
}
