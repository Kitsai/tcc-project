use std::{path::PathBuf, sync::RwLock};

use super::Problem;

pub struct ProblemManager {
    pub current: RwLock<Option<Problem>>,
}

impl ProblemManager {
    pub fn new() -> Self {
        ProblemManager {
            current: RwLock::new(None),
        }
    }
}

impl Default for ProblemManager {
    fn default() -> Self {
        Self::new()
    }
}
