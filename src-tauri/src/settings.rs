use std::{fs::create_dir_all, path::PathBuf};

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::APP_NAME;

#[derive(Serialize, Deserialize, Clone, Copy)]
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

impl Into<u8> for ConcurrencySettings {
    fn into(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Selected(n) => n,
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
    pub fn save(&self) -> Result<(), String> {
        let config_path = config_file().map_err(|err| err.to_string())?;

        let parent = config_path
            .parent()
            .ok_or("File does not have parent directory.")?;

        if !parent.exists() {
            create_dir_all(parent).map_err(|err| err.to_string())?;
        }

        let contents = serde_json::to_string_pretty(self).map_err(|err| err.to_string())?;
        std::fs::write(&config_path, contents).map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        let path = config_file().map_err(|err| err.to_string())?;
        if !path.exists() {
            let def = Self::default();
            def.save()?;
            return Ok(def);
        }

        let content = std::fs::read(path).map_err(|err| err.to_string())?;
        let settings = serde_json::from_slice(&content).map_err(|err| err.to_string())?;
        Ok(settings)
    }

    pub fn update(&self, dto: &AppSettingsDto) -> Result<(), String> {
        *self.max_concurrency.write().map_err(|e| e.to_string())? =
            ConcurrencySettings::from(dto.max_concurrency);
        Ok(())
    }

    pub fn to_dto(&self) -> Result<AppSettingsDto, String> {
        let max_concurrency = self.max_concurrency.read().map_err(|e| e.to_string())?;
        Ok(AppSettingsDto {
            max_concurrency: (*max_concurrency).into(),
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AppSettingsDto {
    max_concurrency: u8,
}
