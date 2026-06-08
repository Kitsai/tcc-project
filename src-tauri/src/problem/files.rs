use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{BINARY_DIR, CPP_COMPILER, PYTHON_INTERPRETER},
    runner::ExecutionRequest,
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
                    .with_extension(""),
            },
        }
    }
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
