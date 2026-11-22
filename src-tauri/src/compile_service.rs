use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::sync::{Mutex, MutexGuard};

use crate::{
    error::{AppError, AppResult},
    problem::ProgrammingLanguage,
    runner::Runner,
};

pub struct CompileService {
    runner: Arc<dyn Runner>,
    lock: Mutex<()>,
}

impl CompileService {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        Self {
            runner,
            lock: Mutex::new(()),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().await
    }

    pub async fn compile(
        &self,
        language: &ProgrammingLanguage,
        relative: &Path,
        project_path: &Path,
    ) -> AppResult<()> {
        language
            .compile(relative, project_path, self.runner.as_ref())
            .await
    }

    pub async fn compile_selected(
        &self,
        problem_path: &Path,
        select: impl FnOnce() -> AppResult<PathBuf>,
    ) -> AppResult<PathBuf> {
        let _guard = self.lock().await;
        let selected = select()?;

        let language =
            ProgrammingLanguage::get_from_path(&selected).ok_or(AppError::InvalidLanguage {
                path: selected.clone(),
            })?;

        self.compile(&language, &selected, problem_path).await?;

        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::MockRunner;

    #[tokio::test]
    async fn compile_delegates_to_the_language_and_never_calls_an_interpreted_runner() {
        let dir = tempfile::tempdir().unwrap();
        let service = CompileService::new(Arc::new(MockRunner::unreachable()));

        service
            .compile(
                &ProgrammingLanguage::Python3,
                Path::new("solution.py"),
                dir.path(),
            )
            .await
            .expect("python compile should be a no-op and never touch the runner");
    }

    #[tokio::test]
    async fn compile_selected_returns_the_selected_path() {
        let dir = tempfile::tempdir().unwrap();
        let service = CompileService::new(Arc::new(MockRunner::unreachable()));

        let path = service
            .compile_selected(
                dir.path(),
                || Ok(PathBuf::from("files/checker.py")),
            )
            .await
            .unwrap();

        assert_eq!(path, PathBuf::from("files/checker.py"));
    }

    #[tokio::test]
    async fn compile_selected_rejects_an_unknown_language() {
        let dir = tempfile::tempdir().unwrap();
        let service = CompileService::new(Arc::new(MockRunner::unreachable()));

        let err = service
            .compile_selected(
                dir.path(),
                || Ok(PathBuf::from("files/notes.txt")),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::InvalidLanguage { .. }));
    }

    #[tokio::test]
    async fn compile_selected_propagates_a_selection_error() {
        let dir = tempfile::tempdir().unwrap();
        let service = CompileService::new(Arc::new(MockRunner::unreachable()));

        let err = service
            .compile_selected(dir.path(), || Err(AppError::NoProblemOpen))
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::NoProblemOpen));
    }

    #[tokio::test]
    async fn compile_selected_reads_the_selection_while_holding_the_lock() {
        let dir = tempfile::tempdir().unwrap();
        let service = CompileService::new(Arc::new(MockRunner::unreachable()));

        service
            .compile_selected(
                dir.path(),
                || {
                    assert!(
                        service.lock.try_lock().is_err(),
                        "the lock must already be held while the selection is read"
                    );
                    Ok(PathBuf::from("files/checker.py"))
                },
            )
            .await
            .unwrap();
    }
}
