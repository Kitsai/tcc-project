use std::{fs::create_dir_all, path::PathBuf};

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::APP_NAME;

#[derive(Serialize, Deserialize)]
pub enum ConcurrencySettings {
    Auto,
    Selected(u8),
}

impl From<u8> for ConcurrencySettings {
    fn from(value: u8) -> Self {
        if value == 0 {
            Self::Auto
        } else {
            Self::Selected(value)
        }
    }
}

pub fn config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = dirs::config_dir()
        .ok_or("Failed to get config directory.")?
        .join(APP_NAME)
        .join("settings.json");
    Ok(path)
}

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub max_concurrency: RwLock<ConcurrencySettings>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            max_concurrency: RwLock::new(ConcurrencySettings::Auto),
        }
    }
}

impl AppSettings {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = config_file()?;

        let parent = config_path
            .parent()
            .ok_or("File does not have parent directory.")?;

        if !parent.exists() {
            create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;

        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_file()?;
        if !path.exists() {
            let def = Self::default();
            def.save()?;
            return Ok(def);
        }

        let content = std::fs::read(path)?;
        let settings = serde_json::from_slice(&content)?;
        Ok(settings)
    }
}
