use tauri::State;

use crate::{
    constants::TESTS_PATH,
    error::AppResult,
    problem::{ProblemManager, TestDefinition, TestDefinitionCreateDto, TestDefinitionEditDto},
    util::Persistant,
};

#[tauri::command]
pub fn get_tests(state: State<ProblemManager>) -> AppResult<Vec<TestDefinition>> {
    let problem_path = state.get_current_path()?;

    TestDefinition::get_all(&problem_path)
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
