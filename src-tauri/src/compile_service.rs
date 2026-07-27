use std::{path::Path, sync::Arc};

use tokio::sync::{Mutex, MutexGuard};

use crate::{problem::ProgrammingLanguage, runner::Runner};

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
    ) -> Result<(), String> {
        language.compile(relative, project_path, self.runner.as_ref()).await
    }
}
