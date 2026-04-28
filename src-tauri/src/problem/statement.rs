use std::fs;

use log::debug;
use serde::{Deserialize, Serialize};

use super::ProblemModule;

#[derive(Serialize, Deserialize)]
pub struct ProblemStatement {
    pub name: String,
    pub legend: String,
    pub input: String,
    pub output: String,
    pub notes: String,
    pub tutorial: String,
}

impl ProblemStatement {
    pub fn new(name: &str) -> Self {
        ProblemStatement {
            name: name.to_string(),
            legend: String::new(),
            input: String::new(),
            output: String::new(),
            notes: String::new(),
            tutorial: String::new(),
        }
    }
}

impl ProblemModule for ProblemStatement {
    fn save(&self, base_path: &std::path::Path) -> Result<(), String> {
        let statement_dir = base_path.join("statement");
        debug!("Creating tex files at {:?}", statement_dir);

        fs::write(statement_dir.join("name.tex"), &self.name).map_err(|e| e.to_string())?;
        fs::write(statement_dir.join("legend.tex"), &self.legend).map_err(|e| e.to_string())?;
        fs::write(statement_dir.join("input.tex"), &self.input).map_err(|e| e.to_string())?;
        fs::write(statement_dir.join("output.tex"), &self.output).map_err(|e| e.to_string())?;
        fs::write(statement_dir.join("notes.tex"), &self.notes).map_err(|e| e.to_string())?;
        fs::write(statement_dir.join("tutorial.tex"), &self.tutorial).map_err(|e| e.to_string())?;

        debug!("Files created");

        Ok(())
    }

    fn load(base_path: &std::path::Path) -> Result<Self, String> {
        let statement_dir = base_path.join("statement");

        let name = fs::read_to_string(statement_dir.join("name.tex")).map_err(|e| e.to_string())?;
        let legend =
            fs::read_to_string(statement_dir.join("legend.tex")).map_err(|e| e.to_string())?;
        debug!("Loaded legend with content {}", legend);
        let input =
            fs::read_to_string(statement_dir.join("input.tex")).map_err(|e| e.to_string())?;
        let output =
            fs::read_to_string(statement_dir.join("output.tex")).map_err(|e| e.to_string())?;
        let notes =
            fs::read_to_string(statement_dir.join("notes.tex")).map_err(|e| e.to_string())?;
        let tutorial =
            fs::read_to_string(statement_dir.join("tutorial.tex")).map_err(|e| e.to_string())?;

        Ok(ProblemStatement {
            name,
            legend,
            input,
            output,
            notes,
            tutorial,
        })
    }
}
