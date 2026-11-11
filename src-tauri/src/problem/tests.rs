use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::CompileService,
    constants::TESTS_PATH,
    error::{AppError, AppResult},
    fs,
    problem::{ProblemFileType, ProgrammingLanguage},
    runner::{ExecutionInfo, Runner},
    util::{next_available_id, PathExt, Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDefinition {
    pub id: u16,
    pub test_type: TestType,
    pub content: String,
    pub example: bool,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TestType {
    Manual,
    Script,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedFile {
    pub name: String,
    pub content: String,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum PreviewOutcome {
    Single { content: String },
    Multiple { files: Vec<GeneratedFile> },
}

impl SerdePersistant for TestDefinition {}

impl TestDefinition {
    pub fn new(id: u16, test_type: TestType, content: &str, example: bool, description: &str) -> Self {
        Self {
            id,
            test_type,
            content: content.trim().to_owned(),
            example,
            description: description.trim().to_owned(),
        }
    }

    /// The on-disk path for test `id` under `problem_path`.
    pub fn path(problem_path: &Path, id: u16) -> PathBuf {
        problem_path.join(TESTS_PATH).join_id(id)
    }

    /// The lowest id not used by any stored test.
    pub fn next_id(problem_path: &Path) -> u16 {
        next_available_id(&problem_path.join(TESTS_PATH))
    }

    pub fn create(problem_path: &Path, dto: TestDefinitionCreateDto) -> AppResult<Self> {
        let path = Self::path(problem_path, dto.id);

        if path.exists() {
            return Err(AppError::Default(format!(
                "Test with id {} already exists",
                dto.id
            )));
        }

        let test = Self::new(
            dto.id,
            dto.test_type,
            &dto.content,
            dto.example,
            &dto.description,
        );
        test.save(&path)?;

        Ok(test)
    }

    /// Applies `dto` to the stored test with the same id and returns the
    /// updated test.
    pub fn update(problem_path: &Path, dto: TestDefinitionEditDto) -> AppResult<Self> {
        if !Self::exists(problem_path, dto.id) {
            return Err(AppError::Default(format!(
                "Test with id {} does not exist",
                dto.id
            )));
        }

        let path = Self::path(problem_path, dto.id);
        let mut current = Self::load(&path)?;

        current.edit(dto);
        current.save(&path)?;

        Ok(current)
    }

    pub fn exists(problem_path: &Path, id: u16) -> bool {
        Self::path(problem_path, id).exists()
    }

    pub fn edit(&mut self, dto: TestDefinitionEditDto) {
        self.test_type = dto.test_type;
        self.content = dto.content.trim().to_owned();
        self.example = dto.example;
        self.description = dto.description.trim().to_owned();
    }

    pub fn get_all(problem_path: &Path) -> AppResult<Vec<Self>> {
        let mut ret = Vec::new();
        let path = problem_path.join(TESTS_PATH);

        let dir_entries = std::fs::read_dir(path).err_to_string()?;

        for entry in dir_entries.flatten() {
            ret.push(Self::load(&entry.path())?);
        }

        Ok(ret)
    }

    pub fn delete(problem_path: &Path, id: u16) -> AppResult<()> {
        fs::delete_file(&Self::path(problem_path, id))
    }

    /// Returns the literal test data for `Manual` tests, or compiles and runs
    /// the referenced generator for `Script` tests. Generators may either
    /// print the test to stdout, or (following testlib's `startTest`
    /// convention) `freopen` stdout into one or more files inside their
    /// working directory — so the generator is run with its cwd set to a
    /// fresh temp directory, and if stdout comes back empty, every file the
    /// generator wrote there is read back. A single file is surfaced as the
    /// test's content; multiple files (a generator that produces a whole
    /// batch in one run) are surfaced separately so the caller can import
    /// each one as its own test.
    pub async fn preview(
        &self,
        problem_path: &Path,
        runner: Arc<dyn Runner>,
        compile_service: &CompileService,
    ) -> AppResult<PreviewOutcome> {
        let TestType::Script = &self.test_type else {
            return Ok(PreviewOutcome::Single {
                content: self.content.clone(),
            });
        };

        let (language, generator_relative, args) = Self::resolve_generator(problem_path, &self.content)?;
        let (info, tempdir) = Self::run_generator(
            &language,
            &generator_relative,
            &args,
            problem_path,
            runner,
            compile_service,
        )
        .await?;

        Self::collect_generator_output(info, tempdir, self.id)
    }

    /// Parses a Script test's content (`<generator> [args...]`) into the
    /// generator's resolved language, its path relative to `problem_path`,
    /// and its CLI args.
    fn resolve_generator(
        problem_path: &Path,
        content: &str,
    ) -> AppResult<(ProgrammingLanguage, PathBuf, Vec<String>)> {
        let mut parts = content.split_whitespace();
        let generator_file = parts
            .next()
            .ok_or_else(|| AppError::from("Empty script line"))?;
        let args: Vec<String> = parts.map(String::from).collect();

        let generator_relative = Path::new(ProblemFileType::Generator.directory()).join(generator_file);
        let (language, generator_relative) = match ProgrammingLanguage::get_from_path(&generator_relative) {
            Some(language) => (language, generator_relative),
            None => ProgrammingLanguage::resolve_bare_name(problem_path, &generator_relative)
                .ok_or_else(|| AppError::InvalidLanguage { path: generator_relative.clone() })?,
        };

        Ok((language, generator_relative, args))
    }

    /// Compiles and runs the generator with its cwd set to a fresh temp
    /// directory, returning the execution result alongside that directory
    /// (kept alive so its contents can still be read back afterwards).
    async fn run_generator(
        language: &ProgrammingLanguage,
        generator_relative: &Path,
        args: &[String],
        problem_path: &Path,
        runner: Arc<dyn Runner>,
        compile_service: &CompileService,
    ) -> AppResult<(ExecutionInfo, tempfile::TempDir)> {
        {
            let _guard = compile_service.lock().await;
            compile_service
                .compile(language, generator_relative, problem_path)
                .await?;
        }

        let tempdir = tempfile::TempDir::new().err_to_string()?;

        let mut request = language.resolve(generator_relative, problem_path).into_request();
        request.with_args(args);
        request.with_cwd(tempdir.path());

        let info = runner.execute(request).await.err_to_string()?;
        if info.exit_code != 0 {
            return Err(AppError::from(format!(
                "Generator exited with code {}: {}",
                info.exit_code, info.stderr
            )));
        }

        Ok((info, tempdir))
    }

    /// Turns a finished generator run into a `PreviewOutcome`: stdout if the
    /// generator printed anything, otherwise whatever files it wrote to its
    /// temp cwd (one file surfaces as `Single`, several as `Multiple`).
    fn collect_generator_output(
        info: ExecutionInfo,
        tempdir: tempfile::TempDir,
        id: u16,
    ) -> AppResult<PreviewOutcome> {
        if !info.stdout.trim().is_empty() {
            return Ok(PreviewOutcome::Single {
                content: info.stdout,
            });
        }

        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(tempdir.path())
            .err_to_string()?
            .flatten()
            .filter(|e| e.path().is_file())
            .collect();
        entries.sort_by_key(|e| {
            e.file_name()
                .to_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(u64::MAX)
        });

        if entries.is_empty() {
            return Err(AppError::from(format!(
                "Generator did not produce output for test {}",
                id
            )));
        }

        if entries.len() == 1 {
            let content = std::fs::read_to_string(entries[0].path()).err_to_string()?;
            return Ok(PreviewOutcome::Single { content });
        }

        let mut files = Vec::with_capacity(entries.len());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(entry.path()).err_to_string()?;
            files.push(GeneratedFile { name, content });
        }

        Ok(PreviewOutcome::Multiple { files })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDefinitionCreateDto {
    pub id: u16,
    pub test_type: TestType,
    pub content: String,
    pub example: bool,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDefinitionEditDto {
    pub id: u16,
    pub test_type: TestType,
    pub content: String,
    pub example: bool,
    pub description: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_problem;

    fn create_dto(id: u16) -> TestDefinitionCreateDto {
        TestDefinitionCreateDto {
            id,
            test_type: TestType::Manual,
            content: "1 2".to_string(),
            example: false,
            description: String::new(),
        }
    }

    #[test]
    fn path_pads_the_id_under_the_tests_directory() {
        let problem_path = Path::new("p");

        assert_eq!(
            TestDefinition::path(problem_path, 3),
            problem_path.join(TESTS_PATH).join("03")
        );
    }

    #[test]
    fn next_id_is_the_first_free_id() {
        let (_dir, problem) = temp_problem("p");
        assert_eq!(TestDefinition::next_id(&problem.path), 1);

        TestDefinition::create(&problem.path, create_dto(1)).unwrap();
        TestDefinition::create(&problem.path, create_dto(3)).unwrap();

        assert_eq!(TestDefinition::next_id(&problem.path), 2);
    }

    fn edit_dto(id: u16) -> TestDefinitionEditDto {
        TestDefinitionEditDto {
            id,
            test_type: TestType::Script,
            content: "  gen 10  ".to_string(),
            example: true,
            description: "  big  ".to_string(),
        }
    }

    #[test]
    fn update_replaces_the_stored_fields_and_returns_the_new_test() {
        let (_dir, problem) = temp_problem("p");
        TestDefinition::create(&problem.path, create_dto(1)).unwrap();

        let updated = TestDefinition::update(&problem.path, edit_dto(1)).unwrap();

        assert!(matches!(updated.test_type, TestType::Script));
        assert_eq!(updated.content, "gen 10");
        assert_eq!(updated.description, "big");
        assert!(updated.example);

        let stored = TestDefinition::load(&TestDefinition::path(&problem.path, 1)).unwrap();
        assert_eq!(stored.content, "gen 10");
        assert!(stored.example);
    }

    #[test]
    fn update_of_a_missing_test_is_an_error_and_creates_nothing() {
        let (_dir, problem) = temp_problem("p");

        let err = TestDefinition::update(&problem.path, edit_dto(4)).err().unwrap();

        assert!(err.to_string().contains("does not exist"));
        assert!(TestDefinition::get_all(&problem.path).unwrap().is_empty());
    }

    #[test]
    fn exists_tells_whether_the_test_is_stored() {
        let (_dir, problem) = temp_problem("p");
        assert!(!TestDefinition::exists(&problem.path, 1));

        TestDefinition::create(&problem.path, create_dto(1)).unwrap();

        assert!(TestDefinition::exists(&problem.path, 1));
        assert!(!TestDefinition::exists(&problem.path, 2));
    }
}
