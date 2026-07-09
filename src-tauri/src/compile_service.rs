use std::{path::Path, sync::Arc};

use crate::{problem::ProgrammingLanguage, runner::Runner};

/// Thin facade bundling the shared `Runner` with the compile step, so command
/// handlers don't each need their own `Arc<dyn Runner>` plumbing.
///
/// Known limitation: compiling the same source from two call sites at once
/// (e.g. selecting a checker and immediately running tests against it) is not
/// synchronized here, so concurrent compiles can race at the compiler's
/// shared output file. Left as a known issue for now.
pub struct CompileService {
    runner: Arc<dyn Runner>,
}

impl CompileService {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        Self { runner }
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
