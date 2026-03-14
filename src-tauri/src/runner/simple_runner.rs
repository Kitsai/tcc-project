use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use tokio::sync::Semaphore;

use super::*;

pub struct SimpleRunner {
    semaphore: Semaphore,
}

impl SimpleRunner {
    pub fn new(concurrency: usize) -> Self {
        Self {
            semaphore: Semaphore::new(concurrency),
        }
    }
}

impl Default for SimpleRunner {
    fn default() -> Self {
        Self::new(4)
    }
}

impl Runner for SimpleRunner {
    async fn execute(&self, request: ExecutionRequest) -> ExecutionResult {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|err| ExecutionError::OTHER(err.to_string()))?;

        let mut command = tokio::process::Command::new(request.command);
        command
            .args(request.args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let now = Instant::now();

        let mut child = command
            .spawn()
            .map_err(|err| ExecutionError::OTHER(err.to_string()))?;

        if let Some(mut std_in) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            std_in.write_all(request.input.as_bytes()).await.ok();
            drop(std_in);
        }

        let result = if let Some(timeout_ms) = request.options.timeout {
            match tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
                .await
            {
                Ok(res) => res.map_err(|err| ExecutionError::OTHER(err.to_string())),
                Err(_) => Err(ExecutionError::TLE(now.elapsed())),
            }
        } else {
            child
                .wait_with_output()
                .await
                .map_err(|err| ExecutionError::OTHER(err.to_string()))
        }?;

        let duration = now.elapsed();

        Ok(ExecutionInfo {
            output: String::from_utf8_lossy(&result.stdout).to_string(),
            error: String::from_utf8_lossy(&result.stderr).to_string(),
            execution_time: duration,
        })
    }
}
