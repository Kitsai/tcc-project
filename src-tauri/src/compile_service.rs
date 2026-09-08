use std::{path::Path, sync::Arc};

use tokio::sync::{Mutex, MutexGuard};

use crate::{error::AppResult, problem::ProgrammingLanguage, runner::Runner};

/// Thin facade bundling the shared `Runner` with the compile step, so command
/// handlers don't each need their own `Arc<dyn Runner>` plumbing.
///
/// `compile()` does not lock by itself — callers must hold a `lock()` guard
/// across their whole read-current-selection -> compile (-> persist)
/// sequence. This keeps a selection change and a test run from interleaving:
/// whichever acquires the lock first fully finishes (including persisting
/// the new checker/validator, for a selection) before the other proceeds, so
/// a queued run never reads a stale or half-updated selection.
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
        language.compile(relative, project_path, self.runner.as_ref()).await
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
            .compile(&ProgrammingLanguage::Python3, Path::new("solution.py"), dir.path())
            .await
            .expect("python compile should be a no-op and never touch the runner");
    }
}
