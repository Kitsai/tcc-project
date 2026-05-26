use serde::{Deserialize, Serialize};

use crate::util::SerdePersistant;

#[derive(Serialize, Deserialize)]
pub struct ValidatorTest {
    pub id: u16,
    pub input: String,
    pub expected: ValidatorTestResult,
    pub actual: ValidatorTestResult,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", try_from = "String")]
pub enum ValidatorTestResult {
    Valid,
    Invalid,
    #[serde(rename = "")]
    None,
}

impl TryFrom<String> for ValidatorTestResult {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_uppercase().trim() {
            "VALID" => Ok(Self::Valid),
            "INVALID" => Ok(Self::Invalid),
            "NONE" | "" => Ok(Self::None),
            _ => Err(format!(
                "\"{}\" is not a valid result; expected VALID, INVALID, or empty",
                s
            )),
        }
    }
}

impl std::str::FromStr for ValidatorTestResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl SerdePersistant for ValidatorTest {}

impl ValidatorTest {
    pub fn new(id: u16, input: &str, expected: ValidatorTestResult) -> Self {
        Self {
            id,
            input: input.to_string(),
            expected,
            actual: ValidatorTestResult::None,
        }
    }
    pub fn edit(&mut self, input: &str, verdict: ValidatorTestResult) {
        self.input = input.to_string();
        self.expected = verdict;
    }

    pub fn set_actual_verdict(&mut self, actual: ValidatorTestResult) {
        self.actual = actual;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorTestCreateDto {
    pub id: u16,
    pub mult: bool,
    pub input: String,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorTestEditDto {
    pub id: u16,
    pub input: String,
    pub verdict: String,
}
