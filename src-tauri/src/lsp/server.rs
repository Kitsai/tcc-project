use std::path::PathBuf;

use async_trait::async_trait;
use tokio::process::Child;

#[async_trait]
pub trait LanguageServer: Send + Sync {
    fn language_id(&self) -> &str;

    fn file_extensions(&self) -> &[&str];

    async fn is_available() -> bool
    where
        Self: Sized;

    async fn spawn(&self, workspace_dir: &PathBuf) -> Result<Child, String>;

    fn name(&self) -> &str;

    fn initialization_options(&self) -> Option<serde_json::Value> {
        None
    }
}

mod clangd;
