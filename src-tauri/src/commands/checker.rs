use tauri::{AppHandle, State};

use crate::{
    compile_service::CompileService,
    error::AppResult,
    problem::{CheckerTest, CheckerTestCreateDto, CheckerTestEditDto, ProblemManager},
    runner::Runner,
};

#[tauri::command]
pub fn get_checker_tests(state: State<ProblemManager>) -> AppResult<Vec<CheckerTest>> {
    let problem_path = state.get_current_path()?;

    CheckerTest::get_all(&problem_path)
}

#[tauri::command]
pub fn get_next_checker_test_id(state: State<ProblemManager>) -> AppResult<u16> {
    Ok(CheckerTest::next_id(&state.get_current_path()?))
}

#[tauri::command]
pub fn checker_test_exists(id: u16, state: State<ProblemManager>) -> AppResult<bool> {
    Ok(CheckerTest::exists(&state.get_current_path()?, id))
}

#[tauri::command]
pub fn create_checker_test(
    test: CheckerTestCreateDto,
    state: State<ProblemManager>,
) -> AppResult<()> {
    CheckerTest::create(test, &state.get_current_path()?)
}

#[tauri::command]
pub fn edit_checker_test(dto: CheckerTestEditDto, state: State<ProblemManager>) -> AppResult<()> {
    CheckerTest::update(&state.get_current_path()?, dto)
}

#[tauri::command]
pub fn delete_checker_test(id: u16, state: State<ProblemManager>) -> AppResult<()> {
    CheckerTest::delete(&state.get_current_path()?, id)?;
    Ok(())
}

#[tauri::command]
pub async fn run_checker_tests(
    runner: State<'_, std::sync::Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
    app: AppHandle,
) -> AppResult<()> {
    let problem_path = problem_manager.get_current_path()?;

    let checker_path = compile_service
        .compile_selected(
            &problem_path,
            || problem_manager.get_current_checker_path(),
            "No checker configured for this problem",
        )
        .await?;

    CheckerTest::run_all(&problem_path, checker_path, app, runner.inner().clone()).await
}
