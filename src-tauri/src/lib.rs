use crate::lsp::{ClangdServer, LspBridge, LspRegistryBuilder, PyLspServer};

use std::sync::Arc;

pub mod commands;
pub mod lsp;
pub mod polygon;
pub mod project;
pub mod runner;
pub mod settings;

const APP_NAME: &str = "tcc-project";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = settings::AppSettings::load().expect("Failed to load settings");
    let registry = LspRegistryBuilder::instance()
        .with(Arc::new(ClangdServer::new()))
        .with(Arc::new(PyLspServer::new()))
        .build();

    let bridge = LspBridge::new(registry.clone());
    tauri::Builder::default()
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
            commands::lsp::lsp_start,
            commands::lsp::lsp_stop_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
