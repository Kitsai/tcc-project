use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::CompileService,
    constants::{EXPORT_MANIFEST_FILENAME, EXPORT_TESTS_PATH},
    error::{AppResult, FsOperation},
    fs::FsResultExt,
    problem::{PreviewOutcome, TestDefinition},
    runner::Runner,
    util::{Persistant, SerdePersistant},
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

    for content in contents {
        collect.push(save_one_file(export_dir, content, next_id, test)?);
        next_id += 1;
    }

    Ok((collect, next_id))
}

fn save_one_file(
    export_dir: &Path,
    content: String,
    id: u16,
    test: &TestDefinition,
) -> AppResult<ExportManifestEntry> {
    let path = export_dir.join(format!("{:02}", id));
    std::fs::write(&path, content).fs_context(FsOperation::Write, &path)?;

    Ok(ExportManifestEntry {
        final_id: id,
        source_test_id: test.id,
        example: test.example,
    })
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
}
