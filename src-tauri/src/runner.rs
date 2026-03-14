pub mod simple_runner;

use std::fmt;
use std::time::Duration;

pub trait Runner {
    fn execute(
        &self,
        request: ExecutionRequest,
    ) -> impl std::future::Future<Output = ExecutionResult> + Send;
}

pub type ExecutionResult = Result<ExecutionInfo, ExecutionError>;

#[derive(Clone, Copy)]
pub struct ExecutionOptions {
    pub timeout: Option<u64>,
    pub memory_limit: Option<usize>,
}

pub struct ExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub input: String,
    pub options: ExecutionOptions,
}

pub struct ExecutionInfo {
    pub output: String,
    pub error: String,
    pub execution_time: Duration,
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

impl std::error::Error for ExecutionError {}
