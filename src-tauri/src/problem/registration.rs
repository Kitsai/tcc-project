use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProblemRegistration {
    path: PathBuf,
    last_opened: String,
}
