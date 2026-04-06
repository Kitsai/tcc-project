use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct Problem {
    location: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct ProblemDto {
    pub path: String,
}

mod definition;
