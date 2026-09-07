use tauri::State;

use crate::{error::AppResult, lsp::LspBridge};

#[tauri::command]
pub async fn lsp_start(
    language_id: String,
    workspace_dir: String,
    bridge: State<'_, LspBridge>,
) -> AppResult<u16> {
    bridge.start_for_language(&language_id, workspace_dir).await
}

#[tauri::command]
pub async fn lsp_stop_all(bridge: State<'_, LspBridge>) -> AppResult<()> {
    bridge.stop_all()
}
