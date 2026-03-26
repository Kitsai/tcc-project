use tauri::State;

use crate::lsp::LspBridge;

#[tauri::command]
pub async fn lsp_start(
    language_id: String,
    workspace_dir: String,
    bridge: State<'_, LspBridge>,
) -> Result<u16, String> {
    bridge.start_for_language(&language_id, workspace_dir).await
}

#[tauri::command]
pub async fn lsp_stop_all(bridge: State<'_, LspBridge>) -> Result<(), String> {
    bridge.stop_all()
}
