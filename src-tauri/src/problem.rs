use std::path::{Path, PathBuf};

use log::debug;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

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

    pub fn save_to_disk(&self) -> AppResult<()> {
        let file_path = self.path.join(format!("{}.prblm", self.definition.name));
        self.save(&file_path)
    }
}

/// Creates the standard problem directory skeleton (`files/`, `solutions/`,
/// `tests/{validator,checker,main}/`, `statement/`) under `base_path`.
/// Shared by `create_problem` and the Polygon package importer.
pub(crate) fn create_problem_dirs(base_path: &Path) -> AppResult<()> {
    use crate::util::ResultExt;

    std::fs::create_dir(base_path.join("files")).err_to_string()?;
    std::fs::create_dir(base_path.join("solutions")).err_to_string()?;
    std::fs::create_dir(base_path.join("tests")).err_to_string()?;
    std::fs::create_dir(base_path.join("tests/validator")).err_to_string()?;
    std::fs::create_dir(base_path.join("tests/checker")).err_to_string()?;
    std::fs::create_dir(base_path.join("tests/main")).err_to_string()?;
    std::fs::create_dir(base_path.join("statement")).err_to_string()?;

    std::fs::write(base_path.join(".gitignore"), "bin/\ntests/export/\n").err_to_string()?;

    Ok(())
}

impl Persistant for Problem {
    fn load(path: &Path) -> AppResult<Self> {
        let base = path
            .parent()
            .ok_or_else(|| AppError::from("Failed to get base problem path"))?;
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

    fn save(&self, path: &Path) -> AppResult<()> {
        self.definition.save(path)?;
        debug!("Saved definition");
        self.stmt.save(&self.path)?;
        debug!("Saved statements");

        Ok(())
    }
}

#[cfg(test)]
mod problem_tests {
    use super::*;
    use crate::test_support::temp_problem;

    #[test]
    fn problem_round_trips_through_disk() {
        let (_dir, mut problem) = temp_problem("roundtrip");
        problem.definition.checker = Some("checker.cpp".to_string());
        problem.stmt.legend = "Legend text".to_string();
        problem.save_to_disk().unwrap();

        let prblm_path = problem.path.join("roundtrip.prblm");
        let loaded = Problem::load(&prblm_path).unwrap();

        assert_eq!(loaded.definition.name, "roundtrip");
        assert_eq!(loaded.definition.checker, Some("checker.cpp".to_string()));
        assert_eq!(loaded.stmt.legend, "Legend text");
        assert_eq!(loaded.path, problem.path);
    }
}

mod checker;
mod definition;
mod dir;
mod export;
mod files;
mod manager;
mod registration;
mod solutions;
mod statement;
mod tests;
mod validator;

pub use checker::{
    CheckerTest, CheckerTestCreateDto, CheckerTestEditDto, CheckerTestError, CheckerVerdict,
};
pub use definition::ProblemDefinition;
pub use dir::ProblemDir;
pub use files::{
    get_default_checkers_path, get_include_paths, ExecutableSpec, ProblemFileType,
    ProgrammingLanguage,
};
pub use manager::ProblemManager;
pub use registration::ProblemRegistration;
pub use solutions::{SolutionDescription, SolutionTag};
pub use statement::ProblemStatement;
pub use tests::{
    GeneratedFile, PreviewOutcome, TestDefinition, TestDefinitionCreateDto, TestDefinitionEditDto,
    TestType,
};
pub use validator::{
    ValidatorTest, ValidatorTestCreateDto, ValidatorTestEditDto, ValidatorTestResult,
};

use crate::util::Persistant;
