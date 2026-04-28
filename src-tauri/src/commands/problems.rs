use std::{
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};

use log::debug;
use serde_json::to_string;
use tauri::webview::cookie::time::UtcDateTime;

use crate::problem::{Problem, ProblemModule, ProblemStatement};

#[tauri::command]
pub fn create_problem(name: String, path: String) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).map_err(|e| e.to_string())?;

    let _date = UtcDateTime::now().to_string();

    if !path.is_dir() {
        return Err("Caminho não é um diretório válido!".to_string());
    }

    let path = path.join(&name);

    fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    debug!("Created folder to problem");

    let problem = Problem::create(&name);

    create_file_dirs(&path)?;

    problem.save(&path)?;

    Ok(problem)
}

fn create_file_dirs(base_path: &Path) -> Result<(), String> {
    fs::create_dir(base_path.join("generators")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("checkers")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("validators")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("solutions")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("tests")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("statement")).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn load_problem(path: String) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).map_err(|e| e.to_string())?;

    verify_path(&path)?;

    Problem::load(&path)
}

fn find_problem_file(path: &Path) -> Result<PathBuf, String> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(".prblm") {
                return Ok(entry.path());
            }
        }
    }

    Err("Could not find path to problem".to_string())
}

fn verify_path(path: &Path) -> Result<(), String> {
    if let Some(extension) = path.extension() {
        if extension == "prblm" {
            return Ok(());
        }
    }

    Err("File is not a problem".to_string())
}
