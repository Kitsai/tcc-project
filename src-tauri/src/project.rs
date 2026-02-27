use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    location: PathBuf,
    polygon_id: Option<usize>,
}
