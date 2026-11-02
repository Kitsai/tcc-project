use std::{fs::create_dir_all, path::PathBuf};

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::{constants::DEFAULT_AUTHOR, error::AppResult, APP_NAME};

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

/// Settings files written before `author_name` existed lack the field, so it
/// needs a default or `AppSettings::load` fails (and the app panics on start).
fn default_author_name() -> RwLock<String> {
    RwLock::new(DEFAULT_AUTHOR.to_string())
}

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub max_concurrency: RwLock<ConcurrencySettings>,
    #[serde(default = "default_author_name")]
    pub author_name: RwLock<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            max_concurrency: RwLock::new(ConcurrencySettings::Auto),
            author_name: default_author_name(),
        }
    }
}

impl AppSettings {
    pub fn save(&self) -> AppResult<()> {
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

    pub fn load() -> AppResult<Self> {
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

    pub fn update(&self, dto: &AppSettingsDto) -> AppResult<()> {
        *self.max_concurrency.write().map_err(|e| e.to_string())? =
            ConcurrencySettings::from(dto.max_concurrency);

        // A blank name would produce an empty MOJ `author` file.
        let author = match dto.author_name.trim() {
            "" => DEFAULT_AUTHOR,
            name => name,
        };
        *self.author_name.write().map_err(|e| e.to_string())? = author.to_string();
        Ok(())
    }

    /// Current author name (never blank: `update` replaces a blank one).
    pub fn author(&self) -> AppResult<String> {
        Ok(self.author_name.read().map_err(|e| e.to_string())?.clone())
    }

    pub fn to_dto(&self) -> AppResult<AppSettingsDto> {
        let max_concurrency = self.max_concurrency.read().map_err(|e| e.to_string())?;
        let author_name = self.author_name.read().map_err(|e| e.to_string())?;
        Ok(AppSettingsDto {
            max_concurrency: (*max_concurrency).into(),
            author_name: author_name.clone(),
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsDto {
    max_concurrency: u8,
    author_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrency_settings_round_trip_through_u8() {
        assert!(matches!(
            ConcurrencySettings::from(0),
            ConcurrencySettings::Auto
        ));
        assert!(matches!(
            ConcurrencySettings::from(4),
            ConcurrencySettings::Selected(4)
        ));

        let auto: u8 = ConcurrencySettings::Auto.into();
        assert_eq!(auto, 0);
        let selected: u8 = ConcurrencySettings::Selected(8).into();
        assert_eq!(selected, 8);
    }

    #[test]
    fn update_and_to_dto_round_trip_without_touching_disk() {
        let settings = AppSettings::default();
        settings
            .update(&AppSettingsDto {
                max_concurrency: 6,
                author_name: "Ana".into(),
            })
            .unwrap();

        let dto = settings.to_dto().unwrap();
        assert_eq!(dto.max_concurrency, 6);
    }

    #[test]
    fn update_with_zero_maps_to_auto() {
        let settings = AppSettings::default();
        settings
            .update(&AppSettingsDto {
                max_concurrency: 0,
                author_name: "Ana".into(),
            })
            .unwrap();

        let dto = settings.to_dto().unwrap();
        assert_eq!(dto.max_concurrency, 0);
    }

    #[test]
    fn author_name_round_trips_and_is_trimmed() {
        let settings = AppSettings::default();
        settings
            .update(&AppSettingsDto {
                max_concurrency: 0,
                author_name: "  Ana Souza  ".into(),
            })
            .unwrap();

        assert_eq!(settings.to_dto().unwrap().author_name, "Ana Souza");
    }

    #[test]
    fn author_getter_returns_the_current_name() {
        let settings = AppSettings::default();
        assert_eq!(settings.author().unwrap(), DEFAULT_AUTHOR);

        settings
            .update(&AppSettingsDto {
                max_concurrency: 0,
                author_name: "Ana".into(),
            })
            .unwrap();
        assert_eq!(settings.author().unwrap(), "Ana");
    }

    #[test]
    fn blank_author_name_falls_back_to_the_default() {
        let settings = AppSettings::default();
        settings
            .update(&AppSettingsDto {
                max_concurrency: 0,
                author_name: "   ".into(),
            })
            .unwrap();

        assert_eq!(settings.to_dto().unwrap().author_name, DEFAULT_AUTHOR);
    }

    #[test]
    fn settings_file_without_author_name_still_loads() {
        let settings: AppSettings =
            serde_json::from_str(r#"{"max_concurrency":"Auto"}"#).unwrap();

        assert_eq!(*settings.author_name.read().unwrap(), DEFAULT_AUTHOR);
    }

    #[test]
    fn dto_serializes_as_camel_case() {
        let json = serde_json::to_value(AppSettings::default().to_dto().unwrap()).unwrap();

        assert!(json.get("maxConcurrency").is_some());
        assert!(json.get("authorName").is_some());
        assert!(json.get("max_concurrency").is_none());
    }

    #[test]
    fn settings_file_keeps_snake_case_keys_on_disk() {
        let json = serde_json::to_value(AppSettings::default()).unwrap();

        assert_eq!(json["author_name"], DEFAULT_AUTHOR);
        assert!(json.get("max_concurrency").is_some());
    }
}
