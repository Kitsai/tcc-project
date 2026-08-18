use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;

#[async_trait]
pub trait Runner: Send + Sync + 'static {
    async fn execute(&self, request: ExecutionRequest) -> ExecutionResult;
}

#[async_trait]
impl Runner for std::sync::Arc<dyn Runner> {
    async fn execute(&self, request: ExecutionRequest) -> ExecutionResult {
        self.as_ref().execute(request).await
    }
}

pub type ExecutionResult = Result<ExecutionInfo, ExecutionError>;

#[derive(Clone)]
pub struct ExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub input: String,
    pub options: ExecutionOptions,
    pub cwd: Option<PathBuf>,
}

impl ExecutionRequest {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_owned(),
            args: Vec::new(),
            input: String::new(),
            options: ExecutionOptions::default(),
            cwd: None,
        }
    }

    pub fn with_arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_owned());

        self
    }

    pub fn with_args(&mut self, args: &[String]) -> &mut Self {
        self.args.extend_from_slice(args);
        self
    }

    pub fn with_options(&mut self, options: ExecutionOptions) -> &mut Self {
        self.options = options;

        self
    }

    pub fn with_input(&mut self, input: &str) -> &mut Self {
        self.input = input.to_owned();

        self
    }

    pub fn with_cwd(&mut self, path: &Path) -> &mut Self {
        self.cwd = Some(path.to_owned());

        self
    }
}

#[derive(Clone, Copy, Default)]
pub struct ExecutionOptions {
    pub timeout: Option<u64>,
    pub memory_limit: Option<usize>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub exit_code: i32,
}

impl ExecutionInfo {
    pub fn to_result(self) -> Result<String, String> {
        if self.exit_code == 0 {
            Ok(self.stdout)
        } else {
            Err(self.stderr)
        }
    }
}

impl fmt::Display for ExecutionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stdout: {}\nStderr: {}\nDuration: {:?}\n",
            self.stdout, self.stderr, self.execution_time,
        )
    }
}

#[derive(Clone, Debug)]
pub enum ExecutionError {
    TLE(Duration),
    ME(usize),
    OTHER(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TLE(time) => write!(f, "Time limit exceeded: {}ms", time.as_millis()),
            Self::ME(size) => write!(f, "Memory limit exceeded: {}mb", size),
            Self::OTHER(message) => write!(f, "{}", message),
        }
    }
}

impl Into<String> for ExecutionError {
    fn into(self) -> String {
        self.to_string()
    }
}

impl std::error::Error for ExecutionError {}

mod simple_runner;

pub use simple_runner::SimpleRunner;
