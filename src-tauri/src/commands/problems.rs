use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
    str::FromStr,
};

use tauri::webview::cookie::time::UtcDateTime;

use crate::problem::Problem;

#[tauri::command]
pub fn create_problem(name: String, path: String) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).map_err(|e| e.to_string())?;

    let _date = UtcDateTime::now().to_string();

    if !path.is_dir() {
        return Err("Caminho não é um diretório válido!".to_string());
    }

    let path = path.join(&name);

    fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    let problem = Problem::create(name);

    let problem_path = path.join(format!("{}.prblm", problem.name));

    let file = File::create(problem_path).map_err(|e| e.to_string())?;
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &problem).map_err(|e| e.to_string())?;

    create_file_dirs(&path)?;

    Ok(problem)
}

fn create_file_dirs(base_path: &Path) -> Result<(), String> {
    fs::create_dir(base_path.join("generators")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("checkers")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("validators")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("solutions")).map_err(|e| e.to_string())?;
    fs::create_dir(base_path.join("tests")).map_err(|e| e.to_string())?;

    Ok(())
}
