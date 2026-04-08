use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Problem {
    pub name: String,
}

impl Problem {
    pub fn create(name: String) -> Self {
        Problem { name }
    }
}

mod registration;

pub use registration::ProblemRegistration;
