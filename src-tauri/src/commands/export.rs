use std::sync::Arc;

use tauri::State;

use crate::{
    compile_service::CompileService,
    error::{AppError, AppResult},
    problem::{self, ExportManifest, MojExportReport, ProblemManager},
    runner::Runner,
    settings::AppSettings,
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

    let validator_path = compile_service
        .compile_selected(
            &problem_path,
            || problem_manager.get_current_validator_path(),
            "No validator configured for this problem",
        )
        .await?;

    problem::validate_export_tests(&problem_path, &validator_path, runner.inner().clone()).await
}

#[tauri::command]
pub async fn solve_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<u16> {
    let problem_path = problem_manager.get_current_path()?;

    let solution_path = compile_service
        .compile_selected(
            &problem_path,
            || problem_manager.get_main_solution_path(),
            "No main solution set for this problem",
        )
        .await?;

    problem::solve_export_tests(&problem_path, &solution_path, runner.inner().clone()).await
}

#[tauri::command]
pub async fn check_export_tests(
    runner: State<'_, Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<u16> {
    let problem_path = problem_manager.get_current_path()?;

    let checker_path = compile_service
        .compile_selected(
            &problem_path,
            || problem_manager.get_current_checker_path(),
            "No checker configured for this problem",
        )
        .await?;

    problem::check_export_tests(&problem_path, &checker_path, runner.inner().clone()).await
}

/// Packs the current problem as a MOJ package at `dest_path`. A non-empty
/// `author` is persisted to the problem first (MOJ requires one). A problem
/// with no author (e.g. imported without one) is exported under the global
/// author name from the settings, without writing it to the problem.
#[tauri::command]
pub fn export_moj_package(
    problem_manager: State<'_, ProblemManager>,
    settings: State<'_, AppSettings>,
    dest_path: String,
    author: Option<String>,
) -> AppResult<MojExportReport> {
    if let Some(author) = author.filter(|a| !a.trim().is_empty()) {
        problem_manager.with_current_mut(|p| p.definition.author = Some(author))?;
    }

    let mut problem = problem_manager
        .current
        .read()
        .err_to_string()?
        .clone()
        .ok_or(AppError::NoProblemOpen)?;
    problem.definition.fill_missing_author(&settings.author()?);

    problem::export_moj_package(&problem, std::path::Path::new(&dest_path))
}
