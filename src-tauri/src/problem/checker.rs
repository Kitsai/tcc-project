use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    constants::CHECKER_TESTS_PATH,
    util::{Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerTest {
    pub id: u16,
    pub input: String,
    pub output: String,
    pub answer: String,
    pub expected: CheckerVerdict,
    pub actual: CheckerVerdict,
    pub comment: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", try_from = "String")]
pub enum CheckerVerdict {
    Ok,
    WrongAnswer,
    PresentationError,
    Crashed,
    #[serde(rename = "")]
    None,
}

impl TryFrom<String> for CheckerVerdict {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_uppercase().trim() {
                "OK" => Ok(Self::Ok),
                "WRONG_ANSWER" | "WA" => Ok(Self::WrongAnswer),
                "PRESENTATION_ERROR" | "PE" => Ok(Self::PresentationError),
                "CRASHED" | "FL" => Ok(Self::Crashed),
                "NONE" | "" => Ok(Self::None),
                _ => return Err(format!("{} is not a valid verdict.\nPossible values: \"OK\", \"WRONG_ANSWER\" (\"WA\"), \"PRESENTATION_ERROR\" (\"PE\"), \"CRASHED\" (\"FL\") ", value))
            }
    }
}

impl SerdePersistant for CheckerTest {}

impl CheckerTest {
    pub fn new(id: u16, input: &str, output: &str, answer: &str, expected: CheckerVerdict) -> Self {
        Self {
            id,
            input: input.trim().to_owned(),
            output: output.trim().to_owned(),
            answer: answer.trim().to_owned(),
            expected,
            actual: CheckerVerdict::None,
            comment: String::new(),
        }
    }

    pub fn get_all(problem_path: &Path) -> Result<Vec<CheckerTest>, String> {
        let mut ret = Vec::new();
        let path = problem_path.join(CHECKER_TESTS_PATH);

        let dir_entries = std::fs::read_dir(path).err_to_string()?;

        for entry in dir_entries.flatten() {
            ret.push(Self::load(&entry.path())?);
        }

        Ok(ret)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerTestCreateDto {
    pub id: u16,
    mult: bool,
    input: String,
    output: String,
    answer: String,
    verdict: String,
}
