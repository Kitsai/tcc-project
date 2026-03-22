use async_trait::async_trait;
use tokio::process::Command;

use crate::lsp::server::LanguageServer;

pub struct ClangdServer;

impl ClangdServer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageServer for ClangdServer {
    fn language_id(&self) -> &str {
        "cpp"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "c", "h", "hpp", "hxx"]
    }

    async fn is_available() -> bool {
        Command::new("clangd")
            .arg("--version")
            .output()
            .await
            .is_ok()
    }

    async fn spawn(&self, workspace_dir: &PathBuf) -> Result<Child, String> {
        Command::new("clangd").arg("--background-index");
    }
}
