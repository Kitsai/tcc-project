use std::fs;

use tauri::{AppHandle, State};

use crate::{
    compile_service::CompileService,
    constants::{LANGUAGE_INVALID_ERR, VALIDATOR_TESTS_PATH},
    problem::{
        ProblemManager, ProgrammingLanguage, ValidatorTest, ValidatorTestCreateDto,
        ValidatorTestEditDto,
    },
    runner::Runner,
    util::{next_available_id, Persistant, ResultExt},
};

#[tauri::command]
pub fn get_validator_tests(state: State<ProblemManager>) -> Result<Vec<ValidatorTest>, String> {
    let problem_path = state.get_current_path()?;

    ValidatorTest::get_all(&problem_path)
}

#[tauri::command]
pub fn get_next_validator_test_id(state: State<ProblemManager>) -> Result<u16, String> {
    let path = state.get_current_path()?.join(VALIDATOR_TESTS_PATH);
    Ok(next_available_id(&path))
}

#[tauri::command]
pub fn validator_test_exists(id: u16, state: State<ProblemManager>) -> Result<bool, String> {
    let path = state
        .get_current_path()?
        .join(VALIDATOR_TESTS_PATH)
        .join(format!("{:02}", id));

    Ok(path.exists())
}

#[tauri::command]
pub fn create_validator_test(
    test: ValidatorTestCreateDto,
    state: State<ProblemManager>,
) -> Result<(), String> {
    ValidatorTest::create(test, &state.get_current_path()?)
}

#[tauri::command]
pub fn edit_validator_test(
    dto: ValidatorTestEditDto,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("{}/{:02}", VALIDATOR_TESTS_PATH, dto.id));

    if path.exists() {
        let mut current = ValidatorTest::load(&path)?;

        current.edit(&dto.input, dto.verdict.parse()?);
        current.save(&path)
    } else {
        Err(format!("Test with id {} does not exist", dto.id))
    }
}

#[tauri::command]
pub fn delete_validator_test(id: u16, state: State<ProblemManager>) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("{}/{:02}", VALIDATOR_TESTS_PATH, id));

    fs::remove_file(path).err_to_string()
}

#[tauri::command]
pub async fn run_validator_tests(
    runner: State<'_, std::sync::Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
    app: AppHandle,
) -> Result<(), String> {
    let problem_path = problem_manager.get_current_path()?;

    let validator_path = problem_manager
        .get_current_validator_path()?
        .ok_or_else(|| "No validator configured for this problem".to_string())?;

    log::debug!(
        "[run_validator_tests] problem_path={:?} validator_path={:?}",
        problem_path,
        validator_path
    );

    let language = ProgrammingLanguage::get_from_path(&validator_path)
        .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;
    compile_service.compile(&language, &validator_path, &problem_path).await?;

    ValidatorTest::run_all(&problem_path, validator_path, app, runner.inner().clone()).await
}
