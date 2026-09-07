use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::{
    constants::NO_PRBLM_ERR,
    error::{AppError, AppResult},
    util::ResultExt,
};

use super::{
    files::get_default_checkers_path, Problem, ProblemFileType, SolutionDescription, SolutionTag,
};

pub struct ProblemManager {
    pub current: RwLock<Option<Problem>>,
}

impl ProblemManager {
    pub fn new() -> Self {
        ProblemManager {
            current: RwLock::new(None),
        }
    }

    pub fn get_current_path(&self) -> AppResult<PathBuf> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem.path.clone())
        } else {
            Err(AppError::from(NO_PRBLM_ERR))
        }
    }

    pub fn set_main_solution(&self, file_name: Option<String>) -> AppResult<()> {
        let mut current = self.current.write().err_to_string()?;
        if let Some(problem) = current.as_mut() {
            if problem.definition.main_solution != file_name {
                problem.definition.main_solution = file_name;
                problem.save_to_disk()?;
            }
        }
        Ok(())
    }

    pub fn sync_main_solution(&self, solutions: &[SolutionDescription]) -> AppResult<()> {
        let main_file = solutions
            .iter()
            .find(|s| matches!(s.tag, SolutionTag::Main))
            .map(|s| s.file_name.clone());
        self.set_main_solution(main_file)
    }

    /// Returns the validator's source path, relative to the problem's root directory.
    pub fn get_current_validator_path(&self) -> AppResult<Option<PathBuf>> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem
                .definition
                .validator
                .as_ref()
                .map(|v| Path::new(ProblemFileType::Validator.directory()).join(v)))
        } else {
            Err(AppError::from(NO_PRBLM_ERR))
        }
    }

    /// Returns the main solution's source path, relative to the problem's
    /// root directory.
    pub fn get_main_solution_path(&self) -> AppResult<Option<PathBuf>> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem
                .definition
                .main_solution
                .as_ref()
                .map(|m| Path::new(ProblemFileType::Solution.directory()).join(m)))
        } else {
            Err(AppError::from(NO_PRBLM_ERR))
        }
    }

    /// Returns the checker's source path. For default checkers (stored with the
    /// `@default:` prefix) this is an absolute path into the bundled resources
    /// directory; for user files it is a relative path under the problem's `files/`.
    pub fn get_current_checker_path(&self) -> AppResult<Option<PathBuf>> {
        let curr = self.current.read().err_to_string()?;

        if let Some(problem) = &*curr {
            Ok(problem.definition.checker.as_ref().map(|c| {
                if let Some(name) = c.strip_prefix("@default:") {
                    get_default_checkers_path()
                        .map(|p| p.join(name))
                        .unwrap_or_else(|| PathBuf::from(name))
                } else {
                    Path::new(ProblemFileType::Checker.directory()).join(c)
                }
            }))
        } else {
            Err(AppError::from(NO_PRBLM_ERR))
        }
    }

    /// Runs `f` against the currently open problem and persists the result,
    /// erroring if no problem is open. Centralizes the
    /// read-lock/mutate/save-or-error shape shared by every command that
    /// just needs to tweak a field on `ProblemDefinition` and write it back.
    pub fn with_current_mut<F>(&self, f: F) -> AppResult<()>
    where
        F: FnOnce(&mut Problem),
    {
        let mut current = self.current.write().err_to_string()?;

        if let Some(problem) = current.as_mut() {
            f(problem);
            problem.save_to_disk()?;
            Ok(())
        } else {
            Err(AppError::from(NO_PRBLM_ERR))
        }
    }
}

impl Default for ProblemManager {
    fn default() -> Self {
        Self::new()
    }
}
