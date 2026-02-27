use tauri::State;

use crate::settings::AppSettings;

#[tauri::command]
pub fn save_settings(state: State<AppSettings>) -> Result<(), String> {
    state.save().map_err(|e| e.to_string())
}
