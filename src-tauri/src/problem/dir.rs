use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemDir {
    Checkers,
    Validators,
    Generators,
    Solutions,
    Tests,
}

impl Display for ProblemDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Checkers => "checkers",
            Self::Validators => "validators",
            Self::Generators => "generators",
            Self::Solutions => "solutions",
            Self::Tests => "tests",
        };

        write!(f, "{}", s)
    }
}

impl AsRef<str> for ProblemDir {
    fn as_ref(&self) -> &str {
        match self {
            Self::Checkers => "checkers",
            Self::Validators => "validators",
            Self::Generators => "generators",
            Self::Solutions => "solutions",
            Self::Tests => "tests",
        }
    }
}
