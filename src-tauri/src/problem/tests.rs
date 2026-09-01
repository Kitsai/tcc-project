use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::CompileService,
    constants::{TESTS_PATH, LANGUAGE_INVALID_ERR},
    error::{AppError, AppResult},
    fs,
    problem::{ProblemFileType, ProgrammingLanguage},
    runner::{ExecutionInfo, Runner},
    util::{Persistant, ResultExt, SerdePersistant},
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
        problem_path.join(TESTS_PATH).join(format!("{:02}", id))
    }

    pub fn create(dto: TestDefinitionCreateDto, problem_path: &Path) -> AppResult<Self> {
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

    pub fn delete(id: u16, problem_path: &Path) -> AppResult<()> {
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
                .ok_or_else(|| AppError::from(LANGUAGE_INVALID_ERR))?,
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