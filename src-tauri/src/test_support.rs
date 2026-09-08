use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use tempfile::TempDir;

use crate::{
    problem::Problem,
    runner::{ExecutionInfo, ExecutionRequest, ExecutionResult, Runner},
    util::EventEmitter,
};

/// A `Runner` whose behavior is entirely driven by a closure, so tests never
/// spawn real processes. Prefer `MockRunner::exit_code` for the common case
/// and `MockRunner::unreachable` to assert a code path never invokes it.
pub struct MockRunner {
    handler: Box<dyn Fn(&ExecutionRequest) -> ExecutionResult + Send + Sync>,
}

impl MockRunner {
    pub fn new(handler: impl Fn(&ExecutionRequest) -> ExecutionResult + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    pub fn exit_code(code: i32) -> Self {
        Self::new(move |_| {
            Ok(ExecutionInfo {
                stdout: String::new(),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code: code,
            })
        })
    }

    pub fn unreachable() -> Self {
        Self::new(|request| panic!("MockRunner was not expected to run {:?}", request.command))
    }
}

#[async_trait]
impl Runner for MockRunner {
    async fn execute(&self, request: ExecutionRequest) -> ExecutionResult {
        (self.handler)(&request)
    }
}

/// A `Clone`-able `EventEmitter` that just records every `emit()` call, so
/// tests can assert on what was emitted instead of needing a real `AppHandle`.
#[derive(Clone, Default)]
pub struct RecordingEmitter {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl RecordingEmitter {
    pub fn events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().unwrap().clone()
    }
}

impl EventEmitter for RecordingEmitter {
    fn emit<S: Serialize + Clone + Send + 'static>(&self, event: &str, payload: S) {
        let value = serde_json::to_value(payload).expect("payload must serialize");
        self.events.lock().unwrap().push((event.to_string(), value));
    }
}

/// Builds a minimal on-disk problem directory (mirroring the layout
/// `commands/problems.rs::create_file_dirs` creates) under a fresh tempdir,
/// and returns the `Problem` pointing at it, already persisted to disk.
pub fn temp_problem(name: &str) -> (TempDir, Problem) {
    let dir = tempfile::tempdir().expect("failed to create tempdir");
    let path = dir.path().join(name);

    std::fs::create_dir_all(path.join("files")).unwrap();
    std::fs::create_dir_all(path.join("solutions")).unwrap();
    std::fs::create_dir_all(path.join("tests/validator")).unwrap();
    std::fs::create_dir_all(path.join("tests/checker")).unwrap();
    std::fs::create_dir_all(path.join("tests/main")).unwrap();
    std::fs::create_dir_all(path.join("statement")).unwrap();

    let problem = Problem::create(name, path);
    problem.save_to_disk().expect("failed to save fixture problem");

    (dir, problem)
}
