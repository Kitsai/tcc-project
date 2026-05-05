use serde::{Deserialize, Serialize};
use tauri::State;

use crate::runner::{ExecutionRequest, Runner, SimpleRunner};

use log::debug;

#[tauri::command]
pub async fn check_languages(runner: State<'_, SimpleRunner>) -> Result<LanguageDetails, String> {
    let mut cpp_request = ExecutionRequest::new("g++");
    cpp_request.with_arg("--version");

    let cpp_res = runner.execute(cpp_request).await;

    let mut py_request = ExecutionRequest::new("python3");
    py_request.with_arg("--version");

    let py_res = runner.execute(py_request).await;

    debug!("Cpp result is {:?}", cpp_res);
    debug!("Python3 result is {:?}", py_res);

    Ok(LanguageDetails {
        python3: py_res.is_ok(),
        gpp: cpp_res.is_ok(),
    })
}

#[derive(Serialize, Deserialize, Default)]
pub struct LanguageDetails {
    pub python3: bool,
    pub gpp: bool,
}
