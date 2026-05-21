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
#[serde(rename_all = "UPPERCASE")]
pub enum ValidatorTestResult {
    Valid,
    Invalid,
    None,
}

impl SerdePersistant for ValidatorTest {}
