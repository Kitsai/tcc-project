use std::path::PathBuf;

use tauri::State;

use crate::{
    problem::{ProblemManager, SolutionDescription},
    util::StringResult,
};

#[tauri::command]
pub async fn get_solutions(
    state: State<'_, ProblemManager>,
) -> StringResult<Vec<SolutionDescription>> {
    let project_path = state.get_current_path()?;

    SolutionDescription::load_all(&project_path)
}

#[tauri::command]
pub async fn delete_solution(
    file_name: String,
    project: State<'_, ProblemManager>,
) -> StringResult<()> {
    let project_path = project.get_current_path()?;

    SolutionDescription::delete_solution(&project_path, file_name)
}

#[tauri::command]
pub async fn add_solution_files(
    paths: Vec<PathBuf>,
    problem: State<'_, ProblemManager>,
) -> StringResult<()> {
    let problem_path = problem.get_current_path()?;

    for path in paths {
        SolutionDescription::create_from_existing(path, &problem_path)?;
    }
    Ok(())
}
