use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use log::debug;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Problem {
    pub path: PathBuf,
    pub definition: ProblemDefinition,
    pub stmt: ProblemStatement,
}

impl Problem {
    pub fn create(name: &str, base_path: PathBuf) -> Self {
        let definition = ProblemDefinition::new(name);
        let stmt = ProblemStatement::new(name);

        Problem { 
            path: base_path, 
            definition, 
            stmt 
        }
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

        Ok(Problem { 
            path: base.to_path_buf(),
            definition, 
            stmt 
        })
    }

    pub fn save(&self) -> Result<(), String> {
        self.definition.save(&self.path)?;
        debug!("Saved definition");
        self.stmt.save(&self.path)?;
        debug!("Saved statements");

        Ok(())
    }
}

pub trait ProblemModule: Sized {
    fn save(&self, base_path: &Path) -> Result<(), String>;
    fn load(base_path: &Path) -> Result<Self, String>;
}

mod definition;
mod manager;
mod registration;
mod statement;

pub use manager::ProblemManager;

pub use registration::ProblemRegistration;

pub use definition::ProblemDefinition;
pub use statement::ProblemStatement;
