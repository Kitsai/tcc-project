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

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file_content(path: String, content: String) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}
