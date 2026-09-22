use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::{self, CompileService},
    constants::EXPORT_TESTS_PATH,
    error::{AppResult, FsOperation},
    fs::FsResultExt,
    problem::{export, GeneratedFile, PreviewOutcome, TestDefinition},
    runner::Runner,
    util::SerdePersistant,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub tests: Vec<ExportManifestEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportManifestEntry {
    pub final_id: u16,
    pub source_test_id: u16,
    pub example: bool,
}

impl SerdePersistant for ExportManifest {}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_problem;

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
}

