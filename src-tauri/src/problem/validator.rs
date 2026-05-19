use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ValidatorTest {
    id: u16,
    input: String,
    expected: ValidatorTestResult,
    actual: ValidatorTestResult,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidatorTestResult {
    Valid,
    Invalid,
    None,
}
