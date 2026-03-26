use async_trait::async_trait;

use crate::lsp::server::LspServer;

#[derive(Clone)]
pub struct ClangdServer;

impl ClangdServer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LspServer for ClangdServer {
    fn name(&self) -> &str {
        "clangd"
    }

    fn language_id(&self) -> &str {
        "cpp"
    }

    fn file_extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "c", "h", "hpp", "hxx", "hpp"]
    }

    fn binary_name(&self) -> &str {
        "clangd"
    }

    fn args(&self) -> Vec<String> {
        vec![
            "--background-index".to_string(),
            "--header-insertion=never".to_string(),
            "--completion-style=detailed".to_string(),
        ]
    }
}
