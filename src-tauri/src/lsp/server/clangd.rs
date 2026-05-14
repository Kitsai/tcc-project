use async_trait::async_trait;

use crate::lsp::server::LspServer;

#[derive(Clone, Default)]
pub struct ClangdServer {
    pub custom_includes: Vec<String>,
    pub compile_commands_dir: Option<String>,
}

impl ClangdServer {
    pub fn new(includes: Vec<String>, compile_commands_dir: Option<String>) -> Self {
        Self {
            custom_includes: includes,
            compile_commands_dir,
        }
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

    fn custom_includes(&self) -> Vec<String> {
        self.custom_includes.clone()
    }

    fn args(&self) -> Vec<String> {
        let mut args = vec![
            "--background-index".to_string(),
            "--header-insertion=never".to_string(),
            "--completion-style=detailed".to_string(),
            "--query-driver=*".to_string(),
            "--log=error".to_string(), // Only log errors, not every AST build
            "--offset-encoding=utf-16".to_string(), // Standard for many LSP clients
        ];

        if let Some(dir) = &self.compile_commands_dir {
            args.push(format!("--compile-commands-dir={}", dir));
        }

        args
    }
}
