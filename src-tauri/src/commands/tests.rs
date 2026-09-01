use std::{collections::HashSet, sync::Arc};

use tauri::State;

use crate::{
    compile_service::CompileService,
    constants::TESTS_PATH,
    error::AppResult,
    problem::{
        GeneratedFile, PreviewOutcome, ProblemManager, TestDefinition, TestDefinitionCreateDto,
        TestDefinitionEditDto, TestType,
    },
    runner::Runner,
    util::{next_available_id, Persistant, ResultExt},
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
    TestDefinition::create(test, &state.get_current_path()?)?;
    Ok(())
}

#[tauri::command]
pub fn edit_test(dto: TestDefinitionEditDto, state: State<ProblemManager>) -> AppResult<TestDefinition> {
    let path = TestDefinition::path(&state.get_current_path()?, dto.id);

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
) -> AppResult<PreviewOutcome> {
    let problem_path = problem.get_current_path()?;
    let path = TestDefinition::path(&problem_path, id);

    let test = TestDefinition::load(&path)?;
    test.preview(&problem_path, runner.inner().clone(), &compile_service)
        .await
}

/// Materializes generated files (surfaced by `preview_test` when a
/// generator writes more than one file in a single run) as brand-new
/// Manual tests, each getting the next free test id. The originating
/// Script test is left untouched.
#[tauri::command]
pub fn import_generated_tests(
    files: Vec<GeneratedFile>,
    state: State<ProblemManager>,
) -> AppResult<Vec<TestDefinition>> {
    let problem_path = state.get_current_path()?;
    let tests_path = problem_path.join(TESTS_PATH);

    // next_available_id rescans the whole directory on every call, so
    // rather than calling it once per file (O(files²) I/O as the directory
    // grows), the existing ids are scanned once up front and free ids are
    // handed out locally as each file is assigned one.
    let mut used_ids: HashSet<u16> = std::fs::read_dir(&tests_path)
        .err_to_string()?
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
        .collect();

    let mut created = Vec::with_capacity(files.len());
    let mut candidate: u16 = 1;
    for file in files {
        while used_ids.contains(&candidate) {
            candidate += 1;
        }
        used_ids.insert(candidate);

        let dto = TestDefinitionCreateDto {
            id: candidate,
            test_type: TestType::Manual,
            content: file.content,
            example: false,
            description: format!("Generated: {}", file.name),
        };
        created.push(TestDefinition::create(dto, &problem_path)?);
    }

    Ok(created)
}
