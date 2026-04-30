use std::{fs, io::BufWriter};

use log::debug;
use serde::{Deserialize, Serialize};

use super::ProblemModule;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProblemDefinition {
    name: String,
}

impl ProblemDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl ProblemModule for ProblemDefinition {
    fn save(&self, base_path: &std::path::Path) -> Result<(), String> {
        let problem_path = base_path.join(format!("{}.prblm", self.name));

        let file = fs::File::create(problem_path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn load(base_path: &std::path::Path) -> Result<Self, String> {
        let content = fs::read(base_path).map_err(|e| e.to_string())?;

        serde_json::from_slice::<Self>(&content).map_err(|e| e.to_string())
    }
}
