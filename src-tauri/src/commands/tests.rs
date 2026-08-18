use std::sync::Arc;

use tauri::State;

use crate::{
    compile_service::CompileService,
    constants::TESTS_PATH,
    error::AppResult,
    problem::{ProblemManager, TestDefinition, TestDefinitionCreateDto, TestDefinitionEditDto},
    runner::Runner,
    util::{next_available_id, Persistant},
};

#[tauri::command]
pub fn get_tests(state: State<ProblemManager>) -> AppResult<Vec<TestDefinition>> {
    let problem_path = state.get_current_path()?;

    TestDefinition::get_all(&problem_path)
}

#[tauri::command]
pub fn get_next_test_id(state: State<ProblemManager>) -> AppResult<u16> {
    let path = state.get_current_path()?.join(TESTS_PATH);
    Ok(next_available_id(&path))
}

#[tauri::command]
pub fn create_test(test: TestDefinitionCreateDto, state: State<ProblemManager>) -> AppResult<()> {
    TestDefinition::create(test, &state.get_current_path()?)
}

#[tauri::command]
pub fn edit_test(dto: TestDefinitionEditDto, state: State<ProblemManager>) -> AppResult<TestDefinition> {
    let path = state
        .get_current_path()?
        .join(format!("{}/{:02}", TESTS_PATH, dto.id));

    let mut current = TestDefinition::load(&path)?;
    current.edit(dto);
    current.save(&path)?;

    Ok(current)
}

#[tauri::command]
pub fn delete_test(id: u16, problem: State<ProblemManager>) -> AppResult<()> {
    TestDefinition::delete(id, &problem.get_current_path()?)
}

#[tauri::command]
pub async fn preview_test(
    id: u16,
    problem: State<'_, ProblemManager>,
    runner: State<'_, Arc<dyn Runner>>,
    compile_service: State<'_, CompileService>,
) -> AppResult<String> {
    let problem_path = problem.get_current_path()?;
    let path = problem_path.join(format!("{}/{:02}", TESTS_PATH, id));

    let test = TestDefinition::load(&path)?;
    test.preview(&problem_path, runner.inner().clone(), &compile_service)
        .await
}
