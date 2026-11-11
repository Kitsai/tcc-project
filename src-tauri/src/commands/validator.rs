use tauri::{AppHandle, State};

use crate::{
    compile_service::CompileService,
    error::AppResult,
    problem::{
        ProblemManager, ValidatorTest, ValidatorTestCreateDto,
        ValidatorTestEditDto,
    },
    runner::Runner,
};

#[tauri::command]
pub fn get_validator_tests(state: State<ProblemManager>) -> AppResult<Vec<ValidatorTest>> {
    let problem_path = state.get_current_path()?;

    ValidatorTest::get_all(&problem_path)
}

#[tauri::command]
pub fn get_next_validator_test_id(state: State<ProblemManager>) -> AppResult<u16> {
    Ok(ValidatorTest::next_id(&state.get_current_path()?))
}

#[tauri::command]
pub fn validator_test_exists(id: u16, state: State<ProblemManager>) -> AppResult<bool> {
    Ok(ValidatorTest::exists(&state.get_current_path()?, id))
}

#[tauri::command]
pub fn create_validator_test(
    test: ValidatorTestCreateDto,
    state: State<ProblemManager>,
) -> AppResult<()> {
    ValidatorTest::create(test, &state.get_current_path()?)
}

#[tauri::command]
pub fn edit_validator_test(dto: ValidatorTestEditDto, state: State<ProblemManager>) -> AppResult<()> {
    ValidatorTest::update(&state.get_current_path()?, dto)
}

#[tauri::command]
pub fn delete_validator_test(id: u16, state: State<ProblemManager>) -> AppResult<()> {
    ValidatorTest::delete(&state.get_current_path()?, id)
}

#[tauri::command]
pub async fn run_validator_tests(
    runner: State<'_, std::sync::Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
    app: AppHandle,
) -> AppResult<()> {
    let problem_path = problem_manager.get_current_path()?;

    let validator_path = compile_service
        .compile_selected(
            &problem_path,
            || problem_manager.get_current_validator_path(),
            "No validator configured for this problem",
        )
        .await?;

    ValidatorTest::run_all(&problem_path, validator_path, app, runner.inner().clone()).await
}
