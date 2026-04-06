use async_trait::async_trait;

use crate::lsp::server::LspServer;

#[derive(Clone)]
pub struct PyLspServer;

impl PyLspServer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LspServer for PyLspServer {
    fn name(&self) -> &str {
        "pylsp"
    }

    fn language_id(&self) -> &str {
        "python"
    }

    fn file_extensions(&self) -> &[&str] {
        &["py", "pyw"]
    }

    fn binary_name(&self) -> &str {
        "pylsp"
    }

    fn args(&self) -> Vec<String> {
        vec!["-vv".to_string()]
    }
}
