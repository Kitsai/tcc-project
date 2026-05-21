use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemDir {
    Files,
    Solutions,
    Tests,
}

impl Display for ProblemDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Files => "files",
            Self::Solutions => "solutions",
            Self::Tests => "tests",
        };

        write!(f, "{}", s)
    }
}

impl AsRef<str> for ProblemDir {
    fn as_ref(&self) -> &str {
        match self {
            Self::Files => "files",
            Self::Solutions => "solutions",
            Self::Tests => "tests",
        }
    }
}
