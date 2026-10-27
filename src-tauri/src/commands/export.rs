use std::sync::Arc;

use tauri::State;

use crate::{
    compile_service::CompileService,
    error::{AppError, AppResult},
    problem::{self, ExportManifest, MojExportReport, ProblemManager, ProgrammingLanguage},
    runner::Runner,
    util::ResultExt,
};

#[tauri::command]
pub async fn generate_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<ExportManifest> {
    let problem_path = problem_manager.get_current_path()?;
    problem::generate_export_tests(&problem_path, runner.inner().clone(), &compile_service).await
}

#[tauri::command]
pub async fn validate_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<u16> {
    let problem_path = problem_manager.get_current_path()?;

    let validator_path = {
        let _guard = compile_service.lock().await;

        let validator_path = problem_manager
            .get_current_validator_path()?
            .ok_or_else(|| "No validator configured for this problem".to_string())?;

        let language = ProgrammingLanguage::get_from_path(&validator_path)
            .ok_or_else(|| AppError::InvalidLanguage { path: validator_path.clone() })?;
        compile_service.compile(&language, &validator_path, &problem_path).await?;

        validator_path
    };

    problem::validate_export_tests(&problem_path, &validator_path, runner.inner().clone()).await
}

#[tauri::command]
pub async fn solve_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<u16> {
    let problem_path = problem_manager.get_current_path()?;

    let solution_path = {
        let _guard = compile_service.lock().await;

        let solution_path = problem_manager
            .get_main_solution_path()?
            .ok_or_else(|| "No main solution set for this problem".to_string())?;

        let language = ProgrammingLanguage::get_from_path(&solution_path)
            .ok_or_else(|| AppError::InvalidLanguage { path: solution_path.clone() })?;
        compile_service.compile(&language, &solution_path, &problem_path).await?;

        solution_path
    };

    problem::solve_export_tests(&problem_path, &solution_path, runner.inner().clone()).await
}

#[tauri::command]
pub async fn check_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<u16> {
    let problem_path = problem_manager.get_current_path()?;

    let checker_path = {
        let _guard = compile_service.lock().await;

        let checker_path = problem_manager
            .get_current_checker_path()?
            .ok_or_else(|| "No checker configured for this problem".to_string())?;

        let language = ProgrammingLanguage::get_from_path(&checker_path)
            .ok_or_else(|| AppError::InvalidLanguage { path: checker_path.clone() })?;
        compile_service.compile(&language, &checker_path, &problem_path).await?;

        checker_path
    };

    problem::check_export_tests(&problem_path, &checker_path, runner.inner().clone()).await
}

/// Packs the current problem as a MOJ package at `dest_path`. A non-empty
/// `author` is persisted to the problem first (MOJ requires one).
#[tauri::command]
pub fn export_moj_package(
    problem_manager: State<'_, ProblemManager>,
    dest_path: String,
    author: Option<String>,
) -> AppResult<MojExportReport> {
    if let Some(author) = author.filter(|a| !a.trim().is_empty()) {
        problem_manager.with_current_mut(|p| p.definition.author = Some(author))?;
    }

    let problem = problem_manager
        .current
        .read()
        .err_to_string()?
        .clone()
        .ok_or(AppError::NoProblemOpen)?;

    problem::export_moj_package(&problem, std::path::Path::new(&dest_path))
}
