use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use tempfile::NamedTempFile;

use crate::{
    error::{AppError, AppResult},
    problem::ProgrammingLanguage,
    runner::{ExecutionInfo, ExecutionRequest, Runner},
    util::{EventEmitter, PathExt, Persistant, SerdePersistant},
};

#[derive(Clone, Serialize)]
pub struct TestRunError {
    pub id: u16,
    pub error: String,
}

pub trait MetaTest: SerdePersistant + Clone + Send + 'static {
    const EVENT_PREFIX: &'static str;
    const TESTS_PATH: &'static str;

    fn id(&self) -> u16;
    fn prepare(&self, request: &mut ExecutionRequest) -> AppResult<Vec<NamedTempFile>>;
    fn record(&mut self, result: &ExecutionInfo);
}

pub(super) async fn run_all<T: MetaTest>(
    problem_path: &Path,
    tests: Vec<T>,
    program_path: PathBuf,
    emitter: impl EventEmitter,
    runner: Arc<dyn Runner>,
) -> AppResult<()> {
    let tests_path = problem_path.join(T::TESTS_PATH);

    let language = ProgrammingLanguage::get_from_path(&program_path).ok_or_else(|| {
        AppError::InvalidLanguage {
            path: program_path.clone(),
        }
    })?;

    let request_template = language.resolve(&program_path, problem_path).into_request();

    log::debug!(
        "[run_all] program={:?} command={:?} args={:?} tests={}",
        program_path,
        request_template.command,
        request_template.args,
        tests.len()
    );

    let mut handles = Vec::new();

    for mut test in tests {
        let runner = runner.clone();
        let emitter = emitter.clone();
        let tests_path = tests_path.clone();
        let mut request = request_template.clone();

        let handle = tokio::spawn(async move {
            let _files = match test.prepare(&mut request) {
                Ok(f) => f,
                Err(e) => {
                    emitter.emit(
                        &format!("{}_test_error", T::EVENT_PREFIX),
                        TestRunError {
                            id: test.id(),
                            error: e.to_string(),
                        },
                    );
                    return;
                }
            };

            match runner.execute(request).await {
                Err(e) => {
                    emitter.emit(
                        &format!("{}_test_error", T::EVENT_PREFIX),
                        TestRunError {
                            id: test.id(),
                            error: e.to_string(),
                        },
                    );
                }
                Ok(info) => {
                    test.record(&info);
                    test.save(&tests_path.join_id(test.id())).ok();

                    emitter.emit(&format!("{}_test_result", T::EVENT_PREFIX), test);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.ok();
    }

    Ok(())
}
