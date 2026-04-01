use std::path::PathBuf;

use async_trait::async_trait;

use crate::lsp::binary_resolver::BinaryResolver;

#[async_trait]
pub trait LspServer: Send + Sync {
    fn name(&self) -> &str;

    fn language_id(&self) -> &str;

    fn file_extensions(&self) -> &[&str];

    fn binary_name(&self) -> &str;

    fn args(&self) -> Vec<String> {
        Vec::new()
    }

    fn custom_includes(&self) -> Vec<String> {
        Vec::new()
    }

    async fn is_available(&self) -> bool {
        BinaryResolver::resolve_binary(self.binary_name())
            .await
            .is_some()
    }

    async fn get_binary_path(&self) -> Option<PathBuf> {
        BinaryResolver::resolve_binary(self.binary_name()).await
    }
}

mod clangd;
mod pylsp;

pub use clangd::ClangdServer;
pub use pylsp::PyLspServer;
