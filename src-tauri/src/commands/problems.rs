use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::debug;
use tauri::{webview::cookie::time::UtcDateTime, State};

use crate::{
    constants::NO_PRBLM_ERR,
    problem::{Problem, ProblemFileType, ProblemManager, ProblemStatement, ValidatorTest},
    util::{Persistant, ResultExt},
};

#[tauri::command]
pub fn create_problem(
    name: String,
    path: String,
    state: State<ProblemManager>,
) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).err_to_string()?;

    let _date = UtcDateTime::now().to_string();

    if !path.is_dir() {
        return Err("Caminho não é um diretório válido!".to_string());
    }

    let path = path.join(&name);

    fs::create_dir_all(&path).err_to_string()?;

    debug!("Created folder to problem");

    let problem = Problem::create(&name, path.clone());

    create_file_dirs(&path)?;

    problem.save_to_disk()?;

    {
        let mut current = state.current.write().err_to_string()?;

        *current = Some(problem.clone());
    }

    Ok(problem)
}

fn create_file_dirs(base_path: &Path) -> Result<(), String> {
    fs::create_dir(base_path.join("files")).err_to_string()?;
    fs::create_dir(base_path.join("solutions")).err_to_string()?;
    fs::create_dir(base_path.join("tests")).err_to_string()?;
    fs::create_dir(base_path.join("tests/validator")).err_to_string()?;
    fs::create_dir(base_path.join("tests/checker")).err_to_string()?;
    fs::create_dir(base_path.join("statement")).err_to_string()?;

    Ok(())
}

#[tauri::command]
pub fn load_problem(path: String, state: State<ProblemManager>) -> Result<Problem, String> {
    let path = PathBuf::from_str(&path).err_to_string()?;

    verify_path(&path)?;

    let problem = Problem::load(&path)?;

    {
        let mut current = state.current.write().err_to_string()?;

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
    let mut current = state.current.write().err_to_string()?;

    if let Some(problem) = current.as_mut() {
        problem.stmt = stmt;
        problem.save_to_disk()?;
        Ok(())
    } else {
        Err("No problem open to save statement!".to_string())
    }
}

#[tauri::command]
pub fn select_problem_file(
    file_type: ProblemFileType,
    file: String,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let mut current = state.current.write().err_to_string()?;

    if let Some(problem) = current.as_mut() {
        let full_path = problem.path.join("files").join(&file);

        if !full_path.exists() {
            return Err(format!("File does not exist: {:?}", full_path));
        }

        match file_type {
            ProblemFileType::Validator => problem.definition.validator = Some(file),
            ProblemFileType::Checker => problem.definition.checker = Some(file),
            _ => return Err(format!("Filetype {} is not valid", file_type)),
        }

        problem.save_to_disk()?;

        Ok(())
    } else {
        Err(NO_PRBLM_ERR.to_string())
    }
}

#[tauri::command]
pub fn get_validator_tests(state: State<ProblemManager>) -> Result<Vec<ValidatorTest>, String> {
    let mut tests: Vec<ValidatorTest> = Vec::new();

    let path = state.get_current_path()?.join("tests/validator");

    let dir_entries = fs::read_dir(path).err_to_string()?;

    for entry in dir_entries.flatten() {
        tests.push(ValidatorTest::load(&entry.path())?);
    }

    Ok(tests)
}

#[tauri::command]
pub fn create_validator_test(
    test: ValidatorTest,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("tests/validator{:02}", test.id));

    if path.exists() {
        Err(format!("Test with id {} already exists", test.id))
    } else {
        test.save(&path)
    }
}

#[tauri::command]
pub fn edit_validator_test(
    test: ValidatorTest,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("tests/validator{:02}", test.id));

    if path.exists() {
        test.save(&path)
    } else {
        Err(format!("Test with id {} does not exist", test.id))
    }
}

#[tauri::command]
pub fn delete_validator_test(id: u16, state: State<ProblemManager>) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("test/validator{:02}", id));

    fs::remove_file(path).err_to_string()
}
