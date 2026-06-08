use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::{constants::NO_PRBLM_ERR, util::ResultExt};

use super::{Problem, ProblemFileType};

pub struct ProblemManager {
    pub current: RwLock<Option<Problem>>,
}

impl ProblemManager {
    pub fn new() -> Self {
        ProblemManager {
            current: RwLock::new(None),
        }
    }

    pub fn get_current_path(&self) -> Result<PathBuf, String> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem.path.clone())
        } else {
            Err(NO_PRBLM_ERR.to_string())
        }
    }

    /// Returns the validator's source path, relative to the problem's root directory.
    pub fn get_current_validator_path(&self) -> Result<Option<PathBuf>, String> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem
                .definition
                .validator
                .as_ref()
                .map(|v| Path::new(ProblemFileType::Validator.directory()).join(v)))
        } else {
            Err(NO_PRBLM_ERR.to_string())
        }
    }
}

impl Default for ProblemManager {
    fn default() -> Self {
        Self::new()
    }
}
