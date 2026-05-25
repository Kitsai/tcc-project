use crate::lsp::{ClangdServer, LspBridge, LspRegistryBuilder, PyLspServer};
use crate::problem::ProblemManager;
use crate::runner::SimpleRunner;

use std::sync::Arc;

pub mod commands;
mod constants;
pub mod lsp;
pub mod polygon;
pub mod problem;
pub mod runner;
pub mod settings;
mod util;

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
    if let Ok(current_dir) = std::env::current_dir() {
        let mut resource_path = current_dir.clone();
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

fn setup_global_compile_flags(includes: &[String]) -> Option<String> {
    let mut home = dirs::home_dir()?;
    home.push(".tcc-project");
    let _ = std::fs::create_dir_all(&home);

    let flags_path = home.join("compile_flags.txt");
    let mut content = String::from("-std=c++17\n");
    for include in includes {
        let normalized = include.replace("\\", "/");
        content.push_str(&format!("-I{}\n", normalized));
    }

    if let Err(e) = std::fs::write(&flags_path, content) {
        eprintln!("Warning: Failed to create global compile_flags.txt: {}", e);
        None
    } else {
        println!("Created global compile_flags.txt at {:?}", flags_path);
        Some(home.to_string_lossy().to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = settings::AppSettings::load().expect("Failed to load settings");

    let include_paths = get_include_paths();
    let compile_commands_dir = setup_global_compile_flags(&include_paths);

    let lsp_registry = LspRegistryBuilder::instance()
        .with(Arc::new(ClangdServer::new(
            include_paths,
            compile_commands_dir,
        )))
        .with(Arc::new(PyLspServer::new()))
        .build();

    let lsp_bridge = LspBridge::new(lsp_registry.clone());
    let problem_manager = ProblemManager::new();
    let runner = SimpleRunner::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(settings)
        .manage(lsp_registry)
        .manage(lsp_bridge)
        .manage(problem_manager)
        .manage(runner)
        .invoke_handler(tauri::generate_handler![
            commands::problems::create_problem,
            commands::problems::load_problem,
            commands::problems::save_statement,
            commands::problems::select_problem_file,
            commands::validator::get_validator_tests,
            commands::validator::get_next_validator_test_id,
            commands::validator::create_validator_test,
            commands::validator::edit_validator_test,
            commands::validator::delete_validator_test,
            commands::lsp::lsp_start,
            commands::lsp::lsp_stop_all,
            commands::files::read_file_content,
            commands::files::write_file_content,
            commands::files::create_file_on_dir,
            commands::files::delete_file_on_dir,
            commands::files::get_files,
            commands::settings::get_app_paths,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::compile::check_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
