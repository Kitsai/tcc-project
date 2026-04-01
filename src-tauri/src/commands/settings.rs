use tauri::State;
use crate::settings::{AppSettings, AppSettingsDto};

#[tauri::command]
pub fn get_app_paths() -> Result<serde_json::Value, String> {
    let mut home = dirs::home_dir().ok_or("Could not find home directory")?;
    home.push(".tcc-project");

    let mut workspace = home.clone();
    workspace.push("workspace");
    let _ = std::fs::create_dir_all(&workspace);

    let mut includes = home.clone();
    includes.push("includes");
    let _ = std::fs::create_dir_all(&includes);

    Ok(serde_json::json!({
        "workspace": workspace.to_string_lossy().to_string(),
        "user_includes": includes.to_string_lossy().to_string(),
        "project_root": home.to_string_lossy().to_string()
    }))
}

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
