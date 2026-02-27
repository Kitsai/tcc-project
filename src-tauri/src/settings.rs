use std::path::PathBuf;

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

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    max_concurrency: RwLock<ConcurrencySettings>,
    config_path: PathBuf,
}

impl Default for AppSettings {
    fn default() -> Self {
        let config_dir = dirs::config_dir().unwrap().join(APP_NAME);

        AppSettings {
            max_concurrency: RwLock::new(ConcurrencySettings::Auto),
            config_path: config_dir.join("app_settings.json"),
        }
    }
}

impl AppSettings {}
