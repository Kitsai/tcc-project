use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{BINARY_DIR, BINARY_EXTENSION, CPP_COMPILER, PYTHON_INTERPRETER},
    runner::{ExecutionRequest, Runner},
    util::ResultExt,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemFileType {
    Checker,
    Validator,
    Generator,
    Solution,
}

impl ProblemFileType {
    pub fn directory(&self) -> &str {
        match self {
            Self::Solution => "solutions",
            _ => "files",
        }
    }
}

impl Display for ProblemFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Checker => "checker",
            Self::Validator => "validator",
            Self::Generator => "generator",
            Self::Solution => "solution",
        };

        write!(f, "{}", s)
    }
}

pub enum ProgrammingLanguage {
    Cpp,
    Python3,
}

impl ProgrammingLanguage {
    pub fn get_from_extension(extension: &str) -> Option<Self> {
        match extension {
            "cpp" | "cxx" | "hpp" => Some(Self::Cpp),
            "py" => Some(Self::Python3),
            _ => None,
        }
    }

    pub fn get_from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| Self::get_from_extension(&ext.to_string_lossy()))
    }

    pub fn is_interpreted(&self) -> bool {
        match self {
            Self::Cpp => false,
            Self::Python3 => true,
        }
    }

    pub fn get_execution_command(&self) -> Option<&str> {
        match self {
            Self::Python3 => Some(PYTHON_INTERPRETER),
            _ => None,
        }
    }

    pub fn get_compiler(&self) -> Option<&str> {
        match self {
            Self::Cpp => Some(CPP_COMPILER),
            _ => None,
        }
    }

    /// Extra arguments to pass to the compiler, e.g. the C++ standard and
    /// `-I` flags pointing at the bundled headers (testlib.h, bits/stdc++.h, ...).
    pub fn get_compiler_args(&self) -> Vec<String> {
        match self {
            Self::Cpp => {
                let mut args = vec!["-std=c++17".to_string()];

                for include in get_include_paths() {
                    args.push(format!("-I{}", include.replace('\\', "/")));
                }

                args
            }
            Self::Python3 => Vec::new(),
        }
    }

    pub fn resolve(&self, relative: &Path, project_path: &Path) -> ExecutableSpec {
        match self {
            Self::Python3 => ExecutableSpec::Interpreted {
                interpreter: PYTHON_INTERPRETER,
                script: project_path.join(relative),
            },
            Self::Cpp => ExecutableSpec::Binary {
                path: project_path
                    .join(BINARY_DIR)
                    .join(relative)
                    .with_extension(BINARY_EXTENSION),
            },
        }
    }

    /// Compiles the file at `relative` into `BINARY_DIR`, creating the destination
    /// directory as needed. Interpreted languages are a no-op.
    pub async fn compile(
        &self,
        relative: &Path,
        project_path: &Path,
        runner: &dyn Runner,
    ) -> Result<(), String> {
        let Some(compiler) = self.get_compiler() else {
            return Ok(());
        };

        let source = project_path.join(relative);
        let destination = project_path
            .join(BINARY_DIR)
            .join(relative)
            .with_extension(BINARY_EXTENSION);

        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).err_to_string()?;
        }

        let mut request = ExecutionRequest::new(compiler);
        request
            .with_arg(&source.to_string_lossy())
            .with_arg("-o")
            .with_arg(&destination.to_string_lossy())
            .with_args(&self.get_compiler_args());

        runner.execute(request).await.err_to_string()?.to_result()?;

        Ok(())
    }
}

/// Resolves the directories that should be passed to the C++ compiler/LSP as
/// header search paths: user-provided headers in `~/.tcc-project/includes`
/// and the bundled headers (testlib.h, bits/stdc++.h) in `resources/includes`.
pub fn get_include_paths() -> Vec<String> {
    let mut includes = Vec::new();

    if let Some(mut home) = dirs::home_dir() {
        home.push(".tcc-project");
        home.push("includes");
        let _ = std::fs::create_dir_all(&home);
        includes.push(home.to_string_lossy().to_string());
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let mut resource_path = current_dir.clone();
        if resource_path.ends_with("src-tauri") {
            resource_path.push("resources/includes");
        } else {
            resource_path.push("src-tauri/resources/includes");
        }

        if resource_path.exists() {
            includes.push(resource_path.to_string_lossy().to_string());
        }
    }

    includes
}

pub enum ExecutableSpec {
    Binary {
        path: PathBuf,
    },
    Interpreted {
        interpreter: &'static str,
        script: PathBuf,
    },
}

impl ExecutableSpec {
    pub fn into_request(self) -> ExecutionRequest {
        match self {
            Self::Binary { path } => ExecutionRequest::new(&path.to_string_lossy()),
            Self::Interpreted {
                interpreter,
                script,
            } => {
                let mut req = ExecutionRequest::new(interpreter);
                req.with_arg(&script.to_string_lossy());
                req
            }
        }
    }
}
