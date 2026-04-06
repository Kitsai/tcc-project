use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProblemDefinition {
    name: String,
    generators: Vec<PathBuf>,
    checkers: Vec<PathBuf>,
    validators: Vec<PathBuf>,
    solutions: Vec<PathBuf>,
}
