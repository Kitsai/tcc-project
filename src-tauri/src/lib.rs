use crate::lsp::{ClangdServer, LspBridge, LspRegistryBuilder, PyLspServer};

use std::sync::Arc;

pub mod commands;
pub mod lsp;
pub mod polygon;
pub mod problem;
pub mod runner;
pub mod settings;

const APP_NAME: &str = "tcc-project";

fn get_include_paths() -> Vec<String> {
    let mut includes = Vec::new();

    // 1. Resolve user headers in ~/.tcc-project/includes
    if let Some(mut home) = dirs::home_dir() {
        home.push(".tcc-project");
        home.push("includes");
        let _ = std::fs::create_dir_all(&home);
        includes.push(home.to_string_lossy().to_string());
    }

    // 2. Resolve bundled resource headers
    // During development, we use the absolute path to our src-tauri/resources/includes folder.
    // In production, Tauri's resource resolver should be used (handled in the LspBridge instead).
    if let Ok(current_dir) = std::env::current_dir() {
        let mut resource_path = current_dir.clone();
        // Depending on where it's launched, we might be in the root or src-tauri
        if resource_path.ends_with("src-tauri") {
            resource_path.push("resources/includes");
        } else {
            resource_path.push("src-tauri/resources/includes");
        }

        if resource_path.exists() {
            includes.push(resource_path.to_string_lossy().to_string());
        }
    }

    includes
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = settings::AppSettings::load().expect("Failed to load settings");

    let registry = LspRegistryBuilder::instance()
        .with(Arc::new(ClangdServer::with_includes(get_include_paths())))
        .with(Arc::new(PyLspServer::new()))
        .build();

    let bridge = LspBridge::new(registry.clone());
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
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
        .manage(registry)
        .manage(bridge)
        .invoke_handler(tauri::generate_handler![
            commands::problems::create_problem,
            commands::problems::load_problem,
            commands::lsp::lsp_start,
            commands::lsp::lsp_stop_all,
            commands::settings::get_app_paths,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
