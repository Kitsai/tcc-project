use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::CompileService,
    constants::{EXPORT_MANIFEST_FILENAME, EXPORT_TESTS_PATH},
    error::{AppError, AppResult, FsOperation},
    fs::FsResultExt,
    problem::{PreviewOutcome, ProgrammingLanguage, TestDefinition},
    runner::{ExecutionOptions, Runner},
    util::{Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportManifest {
    pub tests: Vec<ExportManifestEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportManifestEntry {
    pub final_id: u16,
    pub source_test_id: u16,
    pub example: bool,
}

impl SerdePersistant for ExportManifest {}

pub async fn generate_export_tests(
    problem_path: &Path,
    runner: Arc<dyn Runner>,
    compile_service: &CompileService,
) -> AppResult<ExportManifest> {
    prepare_export_dir(problem_path)?;

    let export_path = problem_path.join(EXPORT_TESTS_PATH);

    let mut tests = TestDefinition::get_all(problem_path)?;
    tests.sort_unstable_by_key(|t| t.id);

    log::debug!(
        "[generate_export_tests] problem_path={:?} definition_count={}",
        problem_path,
        tests.len()
    );

    let mut next_id = 1_u16;

    let mut manifest = ExportManifest { tests: Vec::new() };

    for test in tests {
        let (mut entries, id) = resolve_into_export_dir(
            &test,
            &export_path,
            next_id,
            problem_path,
            runner.clone(),
            compile_service,
        )
        .await?;

        log::debug!(
            "[generate_export_tests] definition id={} produced {} file(s), final_ids={}..{}",
            test.id,
            entries.len(),
            next_id,
            id.saturating_sub(1)
        );

        manifest.tests.append(&mut entries);
        next_id = id;
    }

    log::debug!(
        "[generate_export_tests] done, {} final test(s) written to {:?}",
        manifest.tests.len(),
        export_path
    );

    manifest.save(&export_path.join(EXPORT_MANIFEST_FILENAME))?;

    Ok(manifest)
}

/// Clears and recreates the export staging directory (`tests/export/`),
/// mirroring `commands::dev::clean_binaries`'s treatment of `bin/`: a
/// regenerated-every-run artifacts directory, safe to blow away.
pub fn prepare_export_dir(problem_path: &Path) -> AppResult<PathBuf> {
    let dir = problem_path.join(EXPORT_TESTS_PATH);

    if dir.exists() {
        std::fs::remove_dir_all(&dir).fs_context(FsOperation::RemoveDir, &dir)?;
    }
    std::fs::create_dir_all(&dir).fs_context(FsOperation::CreateDir, &dir)?;

    Ok(dir)
}

/// A `Script` definition's generator may batch out many files in one run
/// (testlib's `startTest` convention). If that definition is marked as an
/// example, only the first `EXAMPLES_PER_BATCH` expanded files are treated
/// as examples in the final package — not the whole batch. Matches what a
/// problem-setter would actually want shown in the statement (a couple of
/// samples), not every generated file. `cabo-carente-4`'s own generator
/// hardcodes exactly 2 sample tests, which is where this default comes from,
/// though it's a per-problem choice, not a platform rule — hence a named
/// constant rather than a hardcoded number inline.
const EXAMPLES_PER_BATCH: usize = 2;

async fn resolve_into_export_dir(
    test: &TestDefinition,
    export_dir: &Path,
    mut next_id: u16,
    problem_path: &Path,
    runner: Arc<dyn Runner>,
    compile_service: &CompileService,
) -> AppResult<(Vec<ExportManifestEntry>, u16)> {
    let outcome = test.preview(problem_path, runner, compile_service).await?;

    let contents: Vec<String> = match outcome {
        PreviewOutcome::Single { content } => vec![content],
        PreviewOutcome::Multiple { files } => files.into_iter().map(|f| f.content).collect(),
    };

    let mut collect = Vec::new();

    for (index, content) in contents.into_iter().enumerate() {
        let example = test.example && index < EXAMPLES_PER_BATCH;
        collect.push(save_one_file(export_dir, content, next_id, test, example)?);
        next_id += 1;
    }

    Ok((collect, next_id))
}

fn export_test_path(export_dir: &Path, final_id: u16) -> PathBuf {
    export_dir.join(format!("{:02}", final_id))
}

fn export_answer_path(export_dir: &Path, final_id: u16) -> PathBuf {
    export_dir.join(format!("{:02}.a", final_id))
}

fn save_one_file(
    export_dir: &Path,
    content: String,
    id: u16,
    test: &TestDefinition,
    example: bool,
) -> AppResult<ExportManifestEntry> {
    let path = export_test_path(export_dir, id);
    std::fs::write(&path, content).fs_context(FsOperation::Write, &path)?;

    Ok(ExportManifestEntry {
        final_id: id,
        source_test_id: test.id,
        example,
    })
}

pub async fn validate_export_tests(
    problem_path: &Path,
    validator_path: &Path,
    runner: Arc<dyn Runner>,
) -> AppResult<u16> {
    let manifest = ExportManifest::load(&get_export_manifest_path(problem_path))?;
    let export_dir = get_export_dir(problem_path);

    let language = ProgrammingLanguage::get_from_path(validator_path).ok_or_else(|| {
        AppError::InvalidLanguage {
            path: validator_path.to_owned(),
        }
    })?;
    let request_template = language
        .resolve(validator_path, problem_path)
        .into_request();

    log::debug!(
        "[validate_export_tests] problem_path={:?} validator_path={:?} test_count={}",
        problem_path,
        validator_path,
        manifest.tests.len()
    );

    for test in &manifest.tests {
        let runner = runner.clone();
        let mut request = request_template.clone();

        let path = export_test_path(&export_dir, test.final_id);
        let input = std::fs::read_to_string(&path).fs_context(FsOperation::Read, &path)?;

        request.with_normalized_input(&input);

        let result = runner.execute(request).await.err_to_string()?;

        if result.exit_code != 0 {
            let comment = if !result.stderr.trim().is_empty() {
                result.stderr.trim()
            } else {
                result.stdout.trim()
            };

            log::debug!(
                "[validate_export_tests] test final_id={} source_test_id={} failed: {}",
                test.final_id,
                test.source_test_id,
                comment
            );

            return Err(AppError::from(format!(
                "Test {} (from definition #{}) failed validation: {}",
                test.final_id, test.source_test_id, comment
            )));
        }
    }

    log::debug!(
        "[validate_export_tests] done, {} test(s) validated",
        manifest.tests.len()
    );

    Ok(manifest.tests.len() as u16)
}

/// Safety-net timeout for running the main solution during `solve` — not a
/// real per-problem time limit (the app doesn't have that concept yet), just
/// a bound so a buggy solution (e.g. an infinite loop) can't hang the whole
/// pipeline with no way to cancel from the UI.
const SOLVE_TIMEOUT_MS: u64 = 10_000;

pub async fn solve_export_tests(
    problem_path: &Path,
    solution_path: &Path,
    runner: Arc<dyn Runner>,
) -> AppResult<u16> {
    let manifest = ExportManifest::load(&get_export_manifest_path(problem_path))?;
    let export_dir = get_export_dir(problem_path);

    let language = ProgrammingLanguage::get_from_path(solution_path).ok_or_else(|| {
        AppError::InvalidLanguage {
            path: solution_path.to_owned(),
        }
    })?;
    let mut request_template = language.resolve(solution_path, problem_path).into_request();
    request_template.with_options(ExecutionOptions {
        timeout: Some(SOLVE_TIMEOUT_MS),
        memory_limit: None,
    });

    log::debug!(
        "[solve_export_tests] problem_path={:?} solution_path={:?} test_count={}",
        problem_path,
        solution_path,
        manifest.tests.len()
    );

    for test in &manifest.tests {
        let runner = runner.clone();
        let mut request = request_template.clone();

        let path = export_test_path(&export_dir, test.final_id);
        let input = std::fs::read_to_string(&path).fs_context(FsOperation::Read, &path)?;

        request.with_normalized_input(&input);

        let result = runner.execute(request).await.err_to_string()?;

        if result.exit_code != 0 {
            let comment = if !result.stderr.trim().is_empty() {
                result.stderr.trim()
            } else {
                result.stdout.trim()
            };

            log::debug!(
                "[solve_export_tests] test final_id={} source_test_id={} failed: {}",
                test.final_id,
                test.source_test_id,
                comment
            );

            return Err(AppError::from(format!(
                "Test {} (from definition #{}) failed while running the main solution: {}",
                test.final_id, test.source_test_id, comment
            )));
        }

        let answer_path = export_answer_path(&export_dir, test.final_id);
        std::fs::write(&answer_path, &result.stdout).fs_context(FsOperation::Write, &answer_path)?;
    }

    log::debug!(
        "[solve_export_tests] done, {} test(s) solved",
        manifest.tests.len()
    );

    Ok(manifest.tests.len() as u16)
}

/// Self-checks every test: runs the checker with the main solution's own
/// output used as *both* the "output" and "answer" arguments. This isn't
/// judging correctness (that's near-tautological when output and answer are
/// the same file) — it's a smoke test that the checker survives running
/// against the real generated data, not just the hand-written meta-tests in
/// `CheckerTest`. Requires `solve_export_tests` to have already run (the
/// `.a` files must exist).
pub async fn check_export_tests(
    problem_path: &Path,
    checker_path: &Path,
    runner: Arc<dyn Runner>,
) -> AppResult<u16> {
    let manifest = ExportManifest::load(&get_export_manifest_path(problem_path))?;
    let export_dir = get_export_dir(problem_path);

    let language = ProgrammingLanguage::get_from_path(checker_path).ok_or_else(|| {
        AppError::InvalidLanguage {
            path: checker_path.to_owned(),
        }
    })?;
    let request_template = language.resolve(checker_path, problem_path).into_request();

    log::debug!(
        "[check_export_tests] problem_path={:?} checker_path={:?} test_count={}",
        problem_path,
        checker_path,
        manifest.tests.len()
    );

    for test in &manifest.tests {
        let runner = runner.clone();
        let mut request = request_template.clone();

        let input_path = export_test_path(&export_dir, test.final_id);
        let answer_path = export_answer_path(&export_dir, test.final_id);

        request.with_args(&[
            input_path.to_string_lossy().to_string(),
            answer_path.to_string_lossy().to_string(),
            answer_path.to_string_lossy().to_string(),
        ]);

        let result = runner.execute(request).await.err_to_string()?;

        if result.exit_code != 0 {
            // testlib checkers print their verdict message via quitf(),
            // which writes to stdout — matching CheckerTest::run_all's
            // stdout-first convention (opposite of validate/solve, where
            // the message is on stderr).
            let comment = if !result.stdout.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            };

            log::debug!(
                "[check_export_tests] test final_id={} source_test_id={} failed: exit_code={} {}",
                test.final_id,
                test.source_test_id,
                result.exit_code,
                comment
            );

            return Err(AppError::from(format!(
                "Test {} (from definition #{}) failed checking: {}",
                test.final_id, test.source_test_id, comment
            )));
        }
    }

    log::debug!(
        "[check_export_tests] done, {} test(s) checked",
        manifest.tests.len()
    );

    Ok(manifest.tests.len() as u16)
}

pub fn get_export_dir(problem_path: &Path) -> PathBuf {
    problem_path.join(EXPORT_TESTS_PATH)
}

pub fn get_export_manifest_path(problem_path: &Path) -> PathBuf {
    problem_path
        .join(EXPORT_TESTS_PATH)
        .join(EXPORT_MANIFEST_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        problem::{TestDefinitionCreateDto, TestType},
        runner::ExecutionInfo,
        test_support::{temp_problem, MockRunner},
    };

    fn create_manual(problem_path: &Path, id: u16, content: &str) -> TestDefinition {
        TestDefinition::create(
            TestDefinitionCreateDto {
                id,
                test_type: TestType::Manual,
                content: content.to_string(),
                example: false,
                description: String::new(),
            },
            problem_path,
        )
        .unwrap()
    }

    /// `MockRunner::unreachable` panics if `execute()` is ever called, so
    /// using it as the runner in every test below also asserts, for free,
    /// that `Manual` tests never touch the runner at all.
    fn unreachable_runner() -> Arc<dyn Runner> {
        Arc::new(MockRunner::unreachable())
    }

    #[test]
    fn prepare_export_dir_creates_missing_dir() {
        let (_dir, problem) = temp_problem("p");

        let export_dir = prepare_export_dir(&problem.path).unwrap();

        assert!(export_dir.exists());
        assert_eq!(export_dir, problem.path.join(EXPORT_TESTS_PATH));
    }

    #[test]
    fn prepare_export_dir_wipes_stale_contents() {
        let (_dir, problem) = temp_problem("p");

        let export_dir = prepare_export_dir(&problem.path).unwrap();
        std::fs::write(export_dir.join("01"), "stale").unwrap();

        let export_dir = prepare_export_dir(&problem.path).unwrap();

        assert!(!export_dir.join("01").exists());
    }

    #[tokio::test]
    async fn resolve_into_export_dir_writes_single_outcome_and_advances_id() {
        let (_dir, problem) = temp_problem("p");
        let export_dir = prepare_export_dir(&problem.path).unwrap();
        let test = create_manual(&problem.path, 5, "hello");
        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        let (entries, next_id) = resolve_into_export_dir(
            &test,
            &export_dir,
            1,
            &problem.path,
            runner,
            &compile_service,
        )
        .await
        .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].final_id, 1);
        assert_eq!(entries[0].source_test_id, 5);
        assert!(!entries[0].example);
        assert_eq!(next_id, 2);
        assert_eq!(
            std::fs::read_to_string(export_dir.join("01")).unwrap(),
            "hello"
        );
    }

    /// Regression test for the bug where the `Single`-outcome arm never
    /// advanced `next_id`, so a second definition resolved to a single file
    /// silently overwrote the first one's.
    #[tokio::test]
    async fn resolve_into_export_dir_continues_numbering_across_calls() {
        let (_dir, problem) = temp_problem("p");
        let export_dir = prepare_export_dir(&problem.path).unwrap();
        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        let first = create_manual(&problem.path, 1, "first");
        let (_, next_id) = resolve_into_export_dir(
            &first,
            &export_dir,
            1,
            &problem.path,
            runner.clone(),
            &compile_service,
        )
        .await
        .unwrap();

        let second = create_manual(&problem.path, 2, "second");
        let (entries, next_id) = resolve_into_export_dir(
            &second,
            &export_dir,
            next_id,
            &problem.path,
            runner,
            &compile_service,
        )
        .await
        .unwrap();

        assert_eq!(entries[0].final_id, 2);
        assert_eq!(next_id, 3);
        assert_eq!(
            std::fs::read_to_string(export_dir.join("01")).unwrap(),
            "first"
        );
        assert_eq!(
            std::fs::read_to_string(export_dir.join("02")).unwrap(),
            "second"
        );
    }

    #[tokio::test]
    async fn generate_export_tests_orders_by_definition_id_not_creation_order() {
        let (_dir, problem) = temp_problem("p");
        // Created out of order on purpose: get_all() doesn't sort (it's a
        // raw read_dir), so if generate_export_tests relied on that order
        // instead of sorting itself, this would come back scrambled.
        create_manual(&problem.path, 3, "third");
        create_manual(&problem.path, 1, "first");
        create_manual(&problem.path, 2, "second");

        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        let manifest = generate_export_tests(&problem.path, runner, &compile_service)
            .await
            .unwrap();

        let source_ids: Vec<u16> = manifest.tests.iter().map(|e| e.source_test_id).collect();
        let final_ids: Vec<u16> = manifest.tests.iter().map(|e| e.final_id).collect();
        assert_eq!(source_ids, vec![1, 2, 3]);
        assert_eq!(final_ids, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn generate_export_tests_writes_files_with_correct_content() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "alpha");
        create_manual(&problem.path, 2, "beta");

        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        generate_export_tests(&problem.path, runner, &compile_service)
            .await
            .unwrap();

        let export_dir = get_export_dir(&problem.path);
        assert_eq!(
            std::fs::read_to_string(export_dir.join("01")).unwrap(),
            "alpha"
        );
        assert_eq!(
            std::fs::read_to_string(export_dir.join("02")).unwrap(),
            "beta"
        );
    }

    #[tokio::test]
    async fn generate_export_tests_persists_manifest_to_disk() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "content");

        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        let manifest = generate_export_tests(&problem.path, runner, &compile_service)
            .await
            .unwrap();

        let loaded = ExportManifest::load(&get_export_manifest_path(&problem.path)).unwrap();
        assert_eq!(loaded.tests.len(), manifest.tests.len());
        assert_eq!(
            loaded.tests[0].source_test_id,
            manifest.tests[0].source_test_id
        );
        assert_eq!(loaded.tests[0].final_id, manifest.tests[0].final_id);
    }

    #[tokio::test]
    async fn validate_export_tests_returns_count_when_validator_accepts_all() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "one");
        create_manual(&problem.path, 2, "two");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::exit_code(0));
        let validator_path = PathBuf::from("validator.py");

        let count = validate_export_tests(&problem.path, &validator_path, runner)
            .await
            .unwrap();

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn validate_export_tests_fails_fast_on_first_invalid_test() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "good");
        create_manual(&problem.path, 2, "bad");
        create_manual(&problem.path, 3, "good");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let calls = Arc::new(std::sync::Mutex::new(0));
        let calls_clone = calls.clone();
        let runner: Arc<dyn Runner> = Arc::new(MockRunner::new(move |request| {
            *calls_clone.lock().unwrap() += 1;
            let exit_code = if request.input.contains("bad") { 1 } else { 0 };
            Ok(ExecutionInfo {
                stdout: String::new(),
                stderr: "bad input rejected".to_string(),
                execution_time: std::time::Duration::default(),
                exit_code,
            })
        }));
        let validator_path = PathBuf::from("validator.py");

        let err = validate_export_tests(&problem.path, &validator_path, runner)
            .await
            .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Test 2"));
        assert!(message.contains("definition #2"));
        assert!(message.contains("bad input rejected"));
        assert_eq!(
            *calls.lock().unwrap(),
            2,
            "should stop after the failing test, never reaching test 3"
        );
    }

    #[tokio::test]
    async fn solve_export_tests_writes_answer_files_and_returns_count() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "one");
        create_manual(&problem.path, 2, "two");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::new(|request| {
            Ok(ExecutionInfo {
                stdout: format!("solved: {}", request.input.trim()),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code: 0,
            })
        }));
        let solution_path = PathBuf::from("solution.py");

        let count = solve_export_tests(&problem.path, &solution_path, runner)
            .await
            .unwrap();

        assert_eq!(count, 2);
        let export_dir = get_export_dir(&problem.path);
        assert_eq!(
            std::fs::read_to_string(export_dir.join("01.a")).unwrap(),
            "solved: one"
        );
        assert_eq!(
            std::fs::read_to_string(export_dir.join("02.a")).unwrap(),
            "solved: two"
        );
    }

    #[tokio::test]
    async fn solve_export_tests_fails_fast_on_crash() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "good");
        create_manual(&problem.path, 2, "crash");
        create_manual(&problem.path, 3, "good");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let calls = Arc::new(std::sync::Mutex::new(0));
        let calls_clone = calls.clone();
        let runner: Arc<dyn Runner> = Arc::new(MockRunner::new(move |request| {
            *calls_clone.lock().unwrap() += 1;
            let exit_code = if request.input.contains("crash") { 1 } else { 0 };
            Ok(ExecutionInfo {
                stdout: String::new(),
                stderr: "segfault".to_string(),
                execution_time: std::time::Duration::default(),
                exit_code,
            })
        }));
        let solution_path = PathBuf::from("solution.py");

        let err = solve_export_tests(&problem.path, &solution_path, runner)
            .await
            .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Test 2"));
        assert!(message.contains("definition #2"));
        assert!(message.contains("segfault"));
        assert_eq!(
            *calls.lock().unwrap(),
            2,
            "should stop after the crashing test, never reaching test 3"
        );

        let export_dir = get_export_dir(&problem.path);
        assert!(
            !export_dir.join("03.a").exists(),
            "the untouched third test should never get an answer file"
        );
    }

    /// A solve runner that just echoes back the (trimmed) input, so tests
    /// can set up a real `.a` file per test without caring about its exact
    /// content.
    fn echo_solve_runner() -> Arc<dyn Runner> {
        Arc::new(MockRunner::new(|request| {
            Ok(ExecutionInfo {
                stdout: format!("solved: {}", request.input.trim()),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code: 0,
            })
        }))
    }

    #[tokio::test]
    async fn check_export_tests_returns_count_when_checker_accepts_all() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "one");
        create_manual(&problem.path, 2, "two");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let solution_path = PathBuf::from("solution.py");
        solve_export_tests(&problem.path, &solution_path, echo_solve_runner())
            .await
            .unwrap();

        let checker_runner: Arc<dyn Runner> = Arc::new(MockRunner::exit_code(0));
        let checker_path = PathBuf::from("checker.py");

        let count = check_export_tests(&problem.path, &checker_path, checker_runner)
            .await
            .unwrap();

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn check_export_tests_fails_fast_and_self_checks_with_matching_output_and_answer() {
        let (_dir, problem) = temp_problem("p");
        create_manual(&problem.path, 1, "one");
        create_manual(&problem.path, 2, "two");
        create_manual(&problem.path, 3, "three");

        let gen_runner = unreachable_runner();
        let compile_service = CompileService::new(gen_runner.clone());
        generate_export_tests(&problem.path, gen_runner, &compile_service)
            .await
            .unwrap();

        let solution_path = PathBuf::from("solution.py");
        solve_export_tests(&problem.path, &solution_path, echo_solve_runner())
            .await
            .unwrap();

        let calls = Arc::new(std::sync::Mutex::new(0));
        let calls_clone = calls.clone();
        let checker_runner: Arc<dyn Runner> = Arc::new(MockRunner::new(move |request| {
            *calls_clone.lock().unwrap() += 1;

            // The checker's own 3 arguments are always the *last* 3 in
            // `request.args`: an interpreted language (Python here)
            // prepends the script path as args[0] before with_args() ever
            // runs, so asserting on the whole vector's length/indices would
            // be testing language-resolution behavior, not check_export_tests.
            let checker_args = &request.args[request.args.len() - 3..];
            assert_eq!(
                checker_args[1], checker_args[2],
                "self-check should pass the same file as both output and answer"
            );

            let exit_code = if checker_args[0].ends_with("02") { 1 } else { 0 };
            Ok(ExecutionInfo {
                stdout: "answer rejected".to_string(),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code,
            })
        }));
        let checker_path = PathBuf::from("checker.py");

        let err = check_export_tests(&problem.path, &checker_path, checker_runner)
            .await
            .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Test 2"));
        assert!(message.contains("definition #2"));
        assert!(message.contains("answer rejected"));
        assert_eq!(
            *calls.lock().unwrap(),
            2,
            "should stop after the failing test, never reaching test 3"
        );
    }

    #[tokio::test]
    async fn generate_export_tests_caps_examples_within_a_multi_file_batch() {
        let (_dir, problem) = temp_problem("p");

        // A Script test whose generator produces 4 files in one run
        // (testlib's startTest multi-file convention), marked as an example.
        TestDefinition::create(
            TestDefinitionCreateDto {
                id: 1,
                test_type: TestType::Script,
                content: "gen.py".to_string(),
                example: true,
                description: String::new(),
            },
            &problem.path,
        )
        .unwrap();

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::new(|request| {
            let cwd = request.cwd.as_ref().expect("generator should run with a cwd");
            for (i, content) in ["a", "b", "c", "d"].iter().enumerate() {
                std::fs::write(cwd.join((i + 1).to_string()), content).unwrap();
            }
            Ok(ExecutionInfo {
                stdout: String::new(),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code: 0,
            })
        }));
        let compile_service = CompileService::new(runner.clone());

        let manifest = generate_export_tests(&problem.path, runner, &compile_service)
            .await
            .unwrap();

        assert_eq!(manifest.tests.len(), 4);
        let example_flags: Vec<bool> = manifest.tests.iter().map(|e| e.example).collect();
        assert_eq!(
            example_flags,
            vec![true, true, false, false],
            "only the first EXAMPLES_PER_BATCH files of the batch should be examples"
        );
    }

    #[tokio::test]
    async fn generate_export_tests_marks_a_single_outcome_example_test_as_example() {
        let (_dir, problem) = temp_problem("p");

        TestDefinition::create(
            TestDefinitionCreateDto {
                id: 1,
                test_type: TestType::Manual,
                content: "content".to_string(),
                example: true,
                description: String::new(),
            },
            &problem.path,
        )
        .unwrap();

        let runner = unreachable_runner();
        let compile_service = CompileService::new(runner.clone());

        let manifest = generate_export_tests(&problem.path, runner, &compile_service)
            .await
            .unwrap();

        assert_eq!(manifest.tests.len(), 1);
        assert!(manifest.tests[0].example);
    }
}
