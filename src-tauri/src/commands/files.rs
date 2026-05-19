use std::{
    fs::{self, File},
    path::PathBuf,
};

use tauri::State;

use crate::problem::{ProblemDir, ProblemManager};

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file_from_dir(
    dir: ProblemDir,
    file_name: String,
    state: State<ProblemManager>,
) -> Result<String, String> {
    let path = {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file_content(path: String, content: String) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    content: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    File::create_new(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    File::create_new(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().map_err(|e| e.to_string())?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };
    fs::remove_file(path).map_err(|e| e.to_string())
}
