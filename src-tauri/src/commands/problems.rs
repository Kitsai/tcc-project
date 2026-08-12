use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::debug;
use tauri::{webview::cookie::time::UtcDateTime, State};

use crate::{
    compile_service::CompileService,
    constants::{LANGUAGE_INVALID_ERR, NO_PRBLM_ERR},
    problem::{
        get_default_checkers_path, Problem, ProblemFileType, ProblemManager, ProblemStatement,
        ProgrammingLanguage,
    },
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
    fs::create_dir(base_path.join("tests/main")).err_to_string()?;
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
pub async fn select_problem_file(
    file_type: ProblemFileType,
    file: String,
    state: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> Result<(), String> {
    if !matches!(
        file_type,
        ProblemFileType::Validator | ProblemFileType::Checker
    ) {
        return Err(format!("Filetype {} is not valid", file_type));
    }

    let problem_path = state.get_current_path()?;
    let relative = Path::new(file_type.directory()).join(&file);
    let full_path = problem_path.join(&relative);

    if !full_path.exists() {
        return Err(format!("File does not exist: {:?}", full_path));
    }

    // Only a file that compiles successfully may be selected as the problem's
    // validator/checker, so a broken file can never be set as active.
    let language = ProgrammingLanguage::get_from_path(&relative)
        .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;

    // Held through the persist below so a concurrent run_*_tests call can
    // never read this selection mid-update (see CompileService doc comment).
    let _guard = compile_service.lock().await;
    compile_service
        .compile(&language, &relative, &problem_path)
        .await?;

    let mut current = state.current.write().err_to_string()?;

    if let Some(problem) = current.as_mut() {
        match file_type {
            ProblemFileType::Validator => problem.definition.validator = Some(file),
            ProblemFileType::Checker => problem.definition.checker = Some(file),
            _ => unreachable!("file_type was validated above"),
        }

        problem.save_to_disk()
    } else {
        Err(NO_PRBLM_ERR.to_string())
    }
}

#[tauri::command]
pub async fn select_default_checker(
    name: String,
    state: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> Result<(), String> {
    let checkers_path = get_default_checkers_path()
        .ok_or_else(|| "Default checkers directory not found".to_string())?;
    let checker_path = checkers_path.join(&name);

    if !checker_path.exists() {
        return Err(format!("Default checker '{}' not found", name));
    }

    let problem_path = state.get_current_path()?;

    let language = ProgrammingLanguage::get_from_path(&checker_path)
        .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;

    // Held through the persist below so a concurrent run_checker_tests call
    // can never read this selection mid-update (see CompileService doc comment).
    let _guard = compile_service.lock().await;
    compile_service
        .compile(&language, &checker_path, &problem_path)
        .await?;

    let mut current = state.current.write().err_to_string()?;
    if let Some(problem) = current.as_mut() {
        problem.definition.checker = Some(format!("@default:{}", name));
        problem.save_to_disk()
    } else {
        Err(NO_PRBLM_ERR.to_string())
    }
}
