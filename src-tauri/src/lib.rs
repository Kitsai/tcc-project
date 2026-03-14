pub mod commands;
pub mod polygon;
pub mod project;
pub mod runner;
pub mod settings;
pub mod util;

const APP_NAME: &str = "tcc-project";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = settings::AppSettings::load().expect("Failed to load settings");
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(settings)
        .invoke_handler(tauri::generate_handler![
            commands::settings::save_settings,
            commands::settings::get_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
