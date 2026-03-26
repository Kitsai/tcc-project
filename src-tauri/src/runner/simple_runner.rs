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

#[async_trait]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_base_cmd() -> String {
        if cfg!(target_os = "windows") {
            "cmd".to_string()
        } else {
            "sh".to_string()
        }
    }

    fn get_args(script: &str) -> Vec<String> {
        if cfg!(target_os = "windows") {
            vec!["/C".to_string(), script.to_string()]
        } else {
            vec!["-c".to_string(), script.to_string()]
        }
    }

    #[tokio::test]
    async fn test_stdout() {
        let runner = SimpleRunner::default();
        let request = ExecutionRequest {
            command: get_base_cmd(),
            args: get_args("echo hello"),
            input: String::new(),
            options: ExecutionOptions {
                timeout: None,
                memory_limit: None,
            },
        };

        let result = runner.execute(request).await.expect("Execution failed");
        assert!(result.output.contains("hello"));
        assert!(result.error.trim().is_empty());
    }

    #[tokio::test]
    async fn test_stderr() {
        let runner = SimpleRunner::default();
        let request = ExecutionRequest {
            command: get_base_cmd(),
            args: get_args("echo error 1>&2"),
            input: String::new(),
            options: ExecutionOptions {
                timeout: None,
                memory_limit: None,
            },
        };

        let result = runner.execute(request).await.expect("Execution failed");
        assert!(result.error.contains("error"));
    }

    #[tokio::test]
    async fn test_stdin() {
        let runner = SimpleRunner::default();
        let command = if cfg!(target_os = "windows") {
            "findstr".to_string()
        } else {
            "cat".to_string()
        };
        let args = if cfg!(target_os = "windows") {
            vec!["^".to_string()]
        } else {
            vec![]
        };

        let request = ExecutionRequest {
            command,
            args,
            input: "hello from stdin\n".to_string(),
            options: ExecutionOptions {
                timeout: None,
                memory_limit: None,
            },
        };

        let result = runner.execute(request).await.expect("Execution failed");
        assert!(result.output.contains("hello from stdin"));
    }

    #[tokio::test]
    async fn test_timeout() {
        let runner = SimpleRunner::default();
        let script = if cfg!(target_os = "windows") {
            "powershell -noprofile -command \"Start-Sleep -Seconds 2\""
        } else {
            "sleep 2"
        };

        let request = ExecutionRequest {
            command: get_base_cmd(),
            args: get_args(script),
            input: String::new(),
            options: ExecutionOptions {
                timeout: Some(100), // 100ms timeout
                memory_limit: None,
            },
        };

        let result = runner.execute(request).await;
        match result {
            Err(ExecutionError::TLE(_)) => {} // Expected
            _ => panic!("Expected Time Limit Exceeded error"),
        }
    }
}
