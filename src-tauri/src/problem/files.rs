use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemFileType {
    Checker,
    Validator,
    Generator,
    Solution,
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
