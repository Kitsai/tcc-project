use serde::{Deserialize, Serialize};

use crate::util::SerdePersistant;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProblemDefinition {
    pub name: String,
    pub checker: Option<String>,
    pub validator: Option<String>,
}

impl ProblemDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            checker: None,
            validator: None,
        }
    }
}

impl SerdePersistant for ProblemDefinition {}
