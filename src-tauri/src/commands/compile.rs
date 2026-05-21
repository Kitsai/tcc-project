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

    let py_is_ok = if let Ok(res) = py_res {
        res.stderr.is_empty()
    } else {
        false
    };

    let cpp_is_ok = if let Ok(res) = cpp_res {
        res.stderr.is_empty()
    } else {
        false
    };

    Ok(LanguageDetails {
        python3: py_is_ok,
        gpp: cpp_is_ok,
    })
}

#[derive(Serialize, Deserialize, Default)]
pub struct LanguageDetails {
    pub python3: bool,
    pub gpp: bool,
}
