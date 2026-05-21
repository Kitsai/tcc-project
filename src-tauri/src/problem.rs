use std::path::{Path, PathBuf};

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
            stmt,
        }
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        let file_path = self.path.join(format!("{}.prblm", self.definition.name));
        self.save(&file_path)
    }
}

impl Persistant for Problem {
    fn load(path: &Path) -> Result<Self, String> {
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
            stmt,
        })
    }

    fn save(&self, path: &Path) -> Result<(), String> {
        self.definition.save(path)?;
        debug!("Saved definition");
        self.stmt.save(&self.path)?;
        debug!("Saved statements");

        Ok(())
    }
}

mod definition;
mod dir;
mod files;
mod manager;
mod registration;
mod statement;
mod validator;

pub use definition::ProblemDefinition;
pub use dir::ProblemDir;
pub use files::ProblemFileType;
pub use manager::ProblemManager;
pub use registration::ProblemRegistration;
pub use statement::ProblemStatement;
pub use validator::ValidatorTest;

use crate::util::Persistant;
