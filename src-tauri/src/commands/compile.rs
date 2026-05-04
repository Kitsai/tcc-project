use tauri::State;

use crate::runner::{ExecutionOptions, ExecutionRequest, Runner, SimpleRunner};

#[tauri::command]
pub async fn check_languages(runner: State<'_, SimpleRunner>) -> Result<(), String> {
    let cpp_request = ExecutionRequest::new("g++ --version");

    let cpp_res = runner.execute(cpp_request).await;

    let py_request = ExecutionRequest::new("python3 --version");

    let py_res = runner.execute(py_request).await;

    Ok(())
}
