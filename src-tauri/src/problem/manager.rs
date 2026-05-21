use std::{path::PathBuf, sync::RwLock};

use crate::{constants::NO_PRBLM_ERR, util::ResultExt};

use super::Problem;

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
}

impl Default for ProblemManager {
    fn default() -> Self {
        Self::new()
    }
}
