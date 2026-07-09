use std::fs::{self, File};

use tauri::State;

use crate::{
    problem::{get_default_checkers_path, ProblemDir, ProblemManager},
    util::ResultExt,
};

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
        let curr = state.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    std::fs::read_to_string(path).err_to_string()
}

#[tauri::command]
pub fn write_file_content(path: String, content: String) -> Result<(), String> {
    std::fs::write(path, content).err_to_string()
}

#[tauri::command]
pub fn write_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    content: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    std::fs::write(path, content).err_to_string()
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    File::create_new(path).err_to_string()?;
    Ok(())
}

#[tauri::command]
pub fn create_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };

    File::create_new(path).err_to_string()?;
    Ok(())
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    fs::remove_file(path).err_to_string()
}

#[tauri::command]
pub fn delete_file_on_dir(
    dir: ProblemDir,
    file_name: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = {
        let curr = state.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            problem.path.join(dir.as_ref()).join(file_name)
        } else {
            return Err("Problem not opened".to_string());
        }
    };
    fs::remove_file(path).err_to_string()
}

#[tauri::command]
pub fn get_default_checker_files() -> Vec<String> {
    let Some(path) = get_default_checkers_path() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            name.contains('.').then_some(name)
        })
        .filter(|n| matches!(n.rsplit_once('.').map(|(_, ext)| ext), Some("cpp" | "py")))
        .collect();
    names.sort();
    names
}

#[tauri::command]
pub fn read_default_checker_content(name: String) -> Result<String, String> {
    let path = get_default_checkers_path()
        .ok_or_else(|| "Default checkers directory not found".to_string())?
        .join(&name);
    std::fs::read_to_string(path).err_to_string()
}

#[tauri::command]
pub fn get_files(state: State<ProblemManager>) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = Vec::new();

    let path = {
        let curr = state.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            problem.path.join("files")
        } else {
            return Ok(files);
        }
    };

    let dir_entries = fs::read_dir(path).err_to_string()?;

    for entry in dir_entries.flatten() {
        files.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(files)
}
