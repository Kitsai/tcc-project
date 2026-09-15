use std::path::PathBuf;

use tauri::State;

use crate::{
    compile_service::CompileService,
    error::AppResult,
    polygon::{self, ImportResult},
    problem::ProblemManager,
    util::ResultExt,
};

#[tauri::command]
pub async fn import_polygon_problem(
    source: String,
    dest_parent: String,
    name: String,
    state: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
) -> AppResult<ImportResult> {
    let source = PathBuf::from(source);
    let dest_path = PathBuf::from(dest_parent).join(&name);

    let result = polygon::import::import_polygon_package(&source, &dest_path, &compile_service).await?;

    {
        let mut current = state.current.write().err_to_string()?;
        *current = Some(result.problem.clone());
    }

    Ok(result)
}