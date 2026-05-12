use std::{
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};

use log::debug;
use tauri::{webview::cookie::time::UtcDateTime, State};

use crate::problem::{Problem, ProblemManager, ProblemStatement};

#[tauri::command]
pub fn create_problem(
    name: String,
    path: String,
    state: State<ProblemManager>,
) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).map_err(|e| e.to_string())?;

    let _date = UtcDateTime::now().to_string();

    if !path.is_dir() {
        return Err("Caminho não é um diretório válido!".to_string());
    }

    let path = path.join(&name);

    fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    debug!("Created folder to problem");

    let problem = Problem::create(&name, path.clone());

    create_file_dirs(&path)?;

    problem.save()?;

    {
        let mut current = state.current.write().map_err(|e| e.to_string())?;

        *current = Some(problem.clone());
    }

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
pub fn load_problem(path: String, state: State<ProblemManager>) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).map_err(|e| e.to_string())?;

    verify_path(&path)?;

    let problem = Problem::load(&path)?;

    {
        let mut current = state.current.write().map_err(|e| e.to_string())?;

        *current = Some(problem.clone());
    }

    Ok(problem)
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

#[tauri::command]
pub fn save_statement(stmt: ProblemStatement, state: State<ProblemManager>) -> Result<(), String> {
    let mut current = state.current.write().map_err(|e| e.to_string())?;

    if let Some(problem) = current.as_mut() {
        problem.stmt = stmt;
        problem.save()?;
        Ok(())
    } else {
        Err("No problem open to save statement!".to_string())
    }
}

#[tauri::command]
pub fn get_files_from(dir: String, state: State<ProblemManager>) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = Vec::new();

    let mut path = PathBuf::new();

    {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            path = problem.path.join(dir);
        }
    }

    let dir_entries = fs::read_dir(path).map_err(|e| e.to_string())?;

    for entry in dir_entries.flatten() {
        files.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(files)
}

#[tauri::command]
pub fn create_file_in(
    dir: String,
    filename: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let mut path = PathBuf::new();

    {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            path = problem.path.join(dir);
        } else {
            return Err("No problem open".to_string());
        }
    }

    let file_path = path.join(filename);
    if file_path.exists() {
        return Err("File already exists".to_string());
    }

    fs::write(file_path, "").map_err(|e| e.to_string())?;

    Ok(())
}
