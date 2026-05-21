use std::fs;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::util::{Persistant, ResultExt};

#[derive(Clone, Serialize, Deserialize)]
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

impl Persistant for ProblemStatement {
    fn save(&self, base_path: &std::path::Path) -> Result<(), String> {
        let statement_dir = base_path.join("statement");
        debug!("Creating tex files at {:?}", statement_dir);

        fs::write(statement_dir.join("name.tex"), &self.name).err_to_string()?;
        fs::write(statement_dir.join("legend.tex"), &self.legend).err_to_string()?;
        fs::write(statement_dir.join("input.tex"), &self.input).err_to_string()?;
        fs::write(statement_dir.join("output.tex"), &self.output).err_to_string()?;
        fs::write(statement_dir.join("notes.tex"), &self.notes).err_to_string()?;
        fs::write(statement_dir.join("tutorial.tex"), &self.tutorial).err_to_string()?;

        debug!("Files created");

        Ok(())
    }

    fn load(base_path: &std::path::Path) -> Result<Self, String> {
        let statement_dir = base_path.join("statement");

        let name = fs::read_to_string(statement_dir.join("name.tex")).err_to_string()?;
        let legend = fs::read_to_string(statement_dir.join("legend.tex")).err_to_string()?;
        debug!("Loaded legend with content {}", legend);
        let input = fs::read_to_string(statement_dir.join("input.tex")).err_to_string()?;
        let output = fs::read_to_string(statement_dir.join("output.tex")).err_to_string()?;
        let notes = fs::read_to_string(statement_dir.join("notes.tex")).err_to_string()?;
        let tutorial = fs::read_to_string(statement_dir.join("tutorial.tex")).err_to_string()?;

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
