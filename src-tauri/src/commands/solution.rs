use std::{path::PathBuf, sync::Arc};

use tauri::State;

use crate::{
    compile_service::CompileService,
    constants::LANGUAGE_INVALID_ERR,
    problem::{ProblemManager, ProgrammingLanguage, SolutionDescription, SolutionTag},
    runner::Runner,
    util::{ResultExt, StringResult},
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
pub async fn create_new_solution(
    file_name: String,
    problem: State<'_, ProblemManager>,
) -> StringResult<()> {
    let problem_path = problem.get_current_path()?;

    SolutionDescription::create_new(file_name, &problem_path)
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

#[tauri::command]
pub async fn change_tag(
    file_name: String,
    tag: SolutionTag,
    state: State<'_, ProblemManager>,
) -> StringResult<Vec<SolutionDescription>> {
    let path = state.get_current_path()?;

    let solutions = SolutionDescription::change_tag(&path, &file_name, tag)?;
    state.sync_main_solution(&solutions)?;
    Ok(solutions)
}

#[tauri::command]
pub async fn verify_solutions(
    state: State<'_, ProblemManager>,
) -> StringResult<Vec<SolutionDescription>> {
    let path = state.get_current_path()?;
    let solutions = SolutionDescription::verify_and_load(&path)?;
    state.sync_main_solution(&solutions)?;
    Ok(solutions)
}

#[tauri::command]
pub async fn output_from_main(
    input: String,
    problem: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
    runner: State<'_, Arc<dyn Runner>>,
) -> StringResult<String> {
    let problem_path = problem.get_current_path()?;

    let solution_path = problem
        .get_main_solution_path()?
        .ok_or_else(|| "No main solution set for this problem".to_string())?;

    let language = ProgrammingLanguage::get_from_path(&solution_path)
        .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;

    {
        let _guard = compile_service.lock().await;
        compile_service
            .compile(&language, &solution_path, &problem_path)
            .await?;
    }

    let mut request = language
        .resolve(&solution_path, &problem_path)
        .into_request();
    request.with_input(&input);

    runner.execute(request).await.err_to_string()?.to_result()
}
