use tauri::State;

use crate::settings::{AppSettings, AppSettingsDto};

#[tauri::command]
pub fn save_settings(
    new_settings: AppSettingsDto,
    state: State<AppSettings>,
) -> Result<AppSettingsDto, String> {
    state.update(&new_settings)?;
    state.save()?;
    state.to_dto()
}

#[tauri::command]
pub fn get_settings(state: State<AppSettings>) -> Result<AppSettingsDto, String> {
    state.to_dto()
}
