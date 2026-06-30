use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::{constants::NO_PRBLM_ERR, util::ResultExt};

use super::{Problem, ProblemFileType, SolutionDescription, SolutionTag};

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

    pub fn set_main_solution(&self, file_name: Option<String>) -> Result<(), String> {
        let mut current = self.current.write().err_to_string()?;
        if let Some(problem) = current.as_mut() {
            if problem.definition.main_solution != file_name {
                problem.definition.main_solution = file_name;
                problem.save_to_disk()?;
            }
        }
        Ok(())
    }

    pub fn sync_main_solution(&self, solutions: &[SolutionDescription]) -> Result<(), String> {
        let main_file = solutions
            .iter()
            .find(|s| matches!(s.tag, SolutionTag::Main))
            .map(|s| s.file_name.clone());
        self.set_main_solution(main_file)
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

    /// Returns the checker's source path, relative to the problem's root directory.
    pub fn get_current_checker_path(&self) -> Result<Option<PathBuf>, String> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem
                .definition
                .checker
                .as_ref()
                .map(|c| Path::new(ProblemFileType::Checker.directory()).join(c)))
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
