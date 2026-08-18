use std::{path::Path, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    compile_service::CompileService,
    constants::{TESTS_PATH, LANGUAGE_INVALID_ERR},
    error::{AppError, AppResult},
    fs,
    problem::{ProblemFileType, ProgrammingLanguage},
    runner::Runner,
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

    pub fn create(dto: TestDefinitionCreateDto, problem_path: &Path) -> AppResult<()> {
        let path = problem_path.join(TESTS_PATH).join(format!("{:02}", dto.id));

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

        Ok(())
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
        let path = problem_path.join(TESTS_PATH).join(format!("{:02}", id));

        fs::delete_file(&path)
    }

    /// Returns the literal test data for `Manual` tests, or compiles and runs
    /// the referenced generator for `Script` tests. Generators may either
    /// print the test to stdout, or (following testlib's `startTest`
    /// convention) `freopen` stdout into a file named after the test number
    /// inside their working directory — so the generator is run with its cwd
    /// set to a fresh temp directory, and if stdout comes back empty, this
    /// reads back the file named after `self.id` from that directory.
    pub async fn preview(
        &self,
        problem_path: &Path,
        runner: Arc<dyn Runner>,
        compile_service: &CompileService,
    ) -> AppResult<String> {
        let TestType::Script = &self.test_type else {
            return Ok(self.content.clone());
        };

        let mut parts = self.content.split_whitespace();
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

        {
            let _guard = compile_service.lock().await;
            compile_service
                .compile(&language, &generator_relative, problem_path)
                .await?;
        }

        let tempdir = tempfile::TempDir::new().err_to_string()?;

        let mut request = language
            .resolve(&generator_relative, problem_path)
            .into_request();
        request.with_args(&args);
        request.with_cwd(tempdir.path());

        let info = runner.execute(request).await.err_to_string()?;
        if info.exit_code != 0 {
            return Err(AppError::from(format!(
                "Generator exited with code {}: {}",
                info.exit_code, info.stderr
            )));
        }

        if !info.stdout.trim().is_empty() {
            return Ok(info.stdout);
        }

        let output_path = tempdir.path().join(self.id.to_string());
        std::fs::read_to_string(&output_path).map_err(|_| {
            AppError::from(format!(
                "Generator did not produce output for test {}",
                self.id
            ))
        })
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