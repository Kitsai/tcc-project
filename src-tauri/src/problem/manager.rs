use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::{
    constants::NO_PRBLM_ERR,
    error::{AppError, AppResult},
    util::ResultExt,
};

use super::{files::get_default_checkers_path, Problem, ProblemFileType, SolutionDescription};

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
        self.set_main_solution(SolutionDescription::main_solution_file_name(solutions))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{problem::SolutionTag, test_support::temp_problem, util::Persistant};

    fn manager_with(problem: Problem) -> ProblemManager {
        ProblemManager {
            current: RwLock::new(Some(problem)),
        }
    }

    #[test]
    fn get_current_path_errors_when_no_problem_open() {
        let manager = ProblemManager::new();
        assert!(manager.get_current_path().is_err());
    }

    #[test]
    fn get_current_path_returns_open_problem_path() {
        let (_dir, problem) = temp_problem("p");
        let expected = problem.path.clone();
        let manager = manager_with(problem);

        assert_eq!(manager.get_current_path().unwrap(), expected);
    }

    #[test]
    fn with_current_mut_errors_when_no_problem_open() {
        let manager = ProblemManager::new();
        assert!(manager.with_current_mut(|_| {}).is_err());
    }

    #[test]
    fn with_current_mut_mutates_and_persists() {
        let (_dir, problem) = temp_problem("p");
        let prblm_path = problem.path.join("p.prblm");
        let manager = manager_with(problem);

        manager
            .with_current_mut(|p| p.definition.checker = Some("checker.cpp".to_string()))
            .unwrap();

        let reloaded = Problem::load(&prblm_path).unwrap();
        assert_eq!(reloaded.definition.checker, Some("checker.cpp".to_string()));
    }

    #[test]
    fn set_main_solution_updates_definition() {
        let (_dir, problem) = temp_problem("p");
        let manager = manager_with(problem);

        manager.set_main_solution(Some("main.cpp".to_string())).unwrap();

        let curr = manager.current.read().unwrap();
        assert_eq!(
            curr.as_ref().unwrap().definition.main_solution,
            Some("main.cpp".to_string())
        );
    }

    #[test]
    fn sync_main_solution_picks_the_tagged_main() {
        let (_dir, problem) = temp_problem("p");
        let manager = manager_with(problem);

        let solutions = vec![
            SolutionDescription {
                file_name: "a.cpp".to_string(),
                tag: SolutionTag::Accepted,
                author: None,
                change_time: String::new(),
            },
            SolutionDescription {
                file_name: "b.cpp".to_string(),
                tag: SolutionTag::Main,
                author: None,
                change_time: String::new(),
            },
        ];
        manager.sync_main_solution(&solutions).unwrap();

        let curr = manager.current.read().unwrap();
        assert_eq!(
            curr.as_ref().unwrap().definition.main_solution,
            Some("b.cpp".to_string())
        );
    }

    #[test]
    fn get_current_checker_path_resolves_plain_filename() {
        let (_dir, mut problem) = temp_problem("p");
        problem.definition.checker = Some("checker.cpp".to_string());
        let manager = manager_with(problem);

        let path = manager.get_current_checker_path().unwrap().unwrap();
        assert_eq!(
            path,
            Path::new(ProblemFileType::Checker.directory()).join("checker.cpp")
        );
    }

    #[test]
    fn get_current_checker_path_resolves_default_prefix_by_filename() {
        let (_dir, mut problem) = temp_problem("p");
        problem.definition.checker = Some("@default:sample.cpp".to_string());
        let manager = manager_with(problem);

        let path = manager.get_current_checker_path().unwrap().unwrap();
        assert_eq!(path.file_name().unwrap(), "sample.cpp");
    }

    #[test]
    fn validator_and_main_solution_paths_are_none_when_unset() {
        let (_dir, problem) = temp_problem("p");
        let manager = manager_with(problem);

        assert!(manager.get_current_validator_path().unwrap().is_none());
        assert!(manager.get_main_solution_path().unwrap().is_none());
    }
}
