use crate::structs::recorder::Recorder;
use std::fs;
use std::path::Path;

pub struct Storage;
const FILE_PATH: &str = "storage.json";

impl Storage {
    pub fn save_recorders(recorders: &Vec<Recorder>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(recorders).map_err(|e| e.to_string())?;

        fs::write(FILE_PATH, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn load_recorders() -> Result<Vec<Recorder>, String> {
        if !Path::new(FILE_PATH).exists() {
            println!("No file ! ");
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(FILE_PATH).map_err(|e| e.to_string())?;

        let recorders = serde_json::from_str(&data).map_err(|e| e.to_string())?;

        Ok(recorders)
    }
}
