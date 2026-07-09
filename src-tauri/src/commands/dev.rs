use tauri::State;

use crate::{constants::BINARY_DIR, problem::ProblemManager, util::ResultExt};

/// Deletes the current problem's compiled-binary cache (`bin/`), so a
/// developer can force a clean recompile without leaving the app. Only does
/// anything in debug builds; a release build refuses and returns an error.
#[tauri::command]
pub fn clean_binaries(state: State<ProblemManager>) -> Result<(), String> {
    if !cfg!(debug_assertions) {
        return Err("clean_binaries is only available in debug builds".to_string());
    }

    let problem_path = state.get_current_path()?;
    let bin_dir = problem_path.join(BINARY_DIR);
    if bin_dir.exists() {
        std::fs::remove_dir_all(&bin_dir).err_to_string()?;
    }
    Ok(())
}
