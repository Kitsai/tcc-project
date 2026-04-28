use std::path::Path;

use log::debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Problem {
    pub definition: ProblemDefinition,
    pub stmt: ProblemStatement,
}

impl Problem {
    pub fn create(name: &str) -> Self {
        let definition = ProblemDefinition::new(name);
        let stmt = ProblemStatement::new(name);

        Problem { definition, stmt }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let base = path
            .parent()
            .ok_or(String::from("Failed to get base problem path"))?;
        debug!("Loading problem at dir {:?}", base);

        let definition: ProblemDefinition = ProblemDefinition::load(path)?;
        debug!("Loaded problem definition");

        let stmt: ProblemStatement = ProblemStatement::load(base)?;
        debug!("Loaded problem statement");

        Ok(Problem { definition, stmt })
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.definition.save(path)?;
        debug!("Saved definition");
        self.stmt.save(path)?;
        debug!("Saved statements");

        Ok(())
    }
}

pub trait ProblemModule: Sized {
    fn save(&self, base_path: &Path) -> Result<(), String>;
    fn load(base_path: &Path) -> Result<Self, String>;
}

mod definition;
mod registration;
mod statement;

pub use registration::ProblemRegistration;

pub use definition::ProblemDefinition;
pub use statement::ProblemStatement;
