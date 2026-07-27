use std::fs;

use tauri::{AppHandle, State};

use crate::{
    compile_service::CompileService,
    constants::{CHECKER_TESTS_PATH, LANGUAGE_INVALID_ERR},
    problem::{CheckerTest, CheckerTestCreateDto, CheckerTestEditDto, ProblemManager, ProgrammingLanguage},
    runner::Runner,
    util::{next_available_id, Persistant, ResultExt},
};

#[tauri::command]
pub fn get_checker_tests(state: State<ProblemManager>) -> Result<Vec<CheckerTest>, String> {
    let problem_path = state.get_current_path()?;

    CheckerTest::get_all(&problem_path)
}

#[tauri::command]
pub fn get_next_checker_test_id(state: State<ProblemManager>) -> Result<u16, String> {
    let path = state.get_current_path()?.join(CHECKER_TESTS_PATH);
    Ok(next_available_id(&path))
}

#[tauri::command]
pub fn checker_test_exists(id: u16, state: State<ProblemManager>) -> Result<bool, String> {
    let path = state
        .get_current_path()?
        .join(CHECKER_TESTS_PATH)
        .join(format!("{:02}", id));

    Ok(path.exists())
}

#[tauri::command]
pub fn create_checker_test(
    test: CheckerTestCreateDto,
    state: State<ProblemManager>,
) -> Result<(), String> {
    CheckerTest::create(test, &state.get_current_path()?)
}

#[tauri::command]
pub fn edit_checker_test(
    dto: CheckerTestEditDto,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("{}/{:02}", CHECKER_TESTS_PATH, dto.id));

    if path.exists() {
        let mut current = CheckerTest::load(&path)?;

        current.edit(&dto.input, &dto.output, &dto.answer, dto.verdict.parse()?);
        current.save(&path)
    } else {
        Err(format!("Test with id {} does not exist", dto.id))
    }
}

#[tauri::command]
pub fn delete_checker_test(id: u16, state: State<ProblemManager>) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("{}/{:02}", CHECKER_TESTS_PATH, id));

    fs::remove_file(path).err_to_string()
}

#[tauri::command]
pub async fn run_checker_tests(
    runner: State<'_, std::sync::Arc<dyn Runner>>,
    problem_manager: State<'_, ProblemManager>,
    compile_service: State<'_, CompileService>,
    app: AppHandle,
) -> Result<(), String> {
    let problem_path = problem_manager.get_current_path()?;

    // Held while reading the current checker so a concurrent select_* call
    // can never leave us reading a stale/half-updated selection (see
    // CompileService doc comment). Released before running tests so a
    // pending select isn't blocked for the whole test run.
    let checker_path = {
        let _guard = compile_service.lock().await;

        let checker_path = problem_manager
            .get_current_checker_path()?
            .ok_or_else(|| "No checker configured for this problem".to_string())?;

        log::debug!(
            "[run_checker_tests] problem_path={:?} checker_path={:?}",
            problem_path,
            checker_path
        );

        let language = ProgrammingLanguage::get_from_path(&checker_path)
            .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;

        // Always attempted; compile() itself skips the actual compiler
        // invocation when the binary is already up to date.
        compile_service.compile(&language, &checker_path, &problem_path).await?;

        checker_path
    };

    CheckerTest::run_all(&problem_path, checker_path, app, runner.inner().clone()).await
}
