use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    constants::TESTS_PATH,
    error::{AppError, AppResult},
    fs,
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