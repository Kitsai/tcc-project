use std::sync::Arc;

use tauri::State;

use crate::{
    compile_service::CompileService,
    error::AppResult,
    problem::{self, ExportManifest, ProblemManager},
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
