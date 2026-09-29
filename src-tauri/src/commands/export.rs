use std::sync::Arc;

use tauri::State;

use crate::{
    compile_service::CompileService,
    error::{AppError, AppResult},
    problem::{self, ExportManifest, ProblemManager, ProgrammingLanguage},
    runner::Runner,
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
