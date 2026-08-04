use serde::{Deserialize, Serialize};

use crate::util::SerdePersistant;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDefinition {
    pub id: u16,
    pub test_type: TestType,
    pub content: String,
    pub example: bool,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub enum TestType {
    Manual,
    Script,
}

impl SerdePersistant for TestDefinition {}

impl TestDefinition {}
