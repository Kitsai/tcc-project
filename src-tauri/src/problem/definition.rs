use serde::{Deserialize, Serialize};

use crate::util::SerdePersistant;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDefinition {
    pub name: String,
    pub checker: Option<String>,
    pub validator: Option<String>,
    pub main_solution: Option<String>,
}

impl ProblemDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            checker: None,
            validator: None,
            main_solution: None,
        }
    }
}

impl SerdePersistant for ProblemDefinition {}
