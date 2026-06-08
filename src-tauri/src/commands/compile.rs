use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    problem::{ProblemFileType, ProblemManager, ProgrammingLanguage},
    runner::{ExecutionRequest, Runner},
};

use log::debug;

#[tauri::command]
pub async fn check_languages(
    runner: State<'_, Arc<dyn Runner>>,
) -> Result<LanguageDetails, String> {
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

#[tauri::command]
pub async fn check_file_compiles(
    file_type: ProblemFileType,
    file: String,
    problem_manager: State<'_, ProblemManager>,
    runner: State<'_, Arc<dyn Runner>>,
) -> Result<(), String> {
    let relative = PathBuf::from(file_type.directory()).join(&file);
    let lang = ProgrammingLanguage::get_from_path(&relative).ok_or("Unknown Language")?;
    let problem_path = problem_manager.get_current_path()?;

    lang.compile(&relative, &problem_path, runner.inner().as_ref())
        .await
}
