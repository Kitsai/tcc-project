use serde::{Deserialize, Serialize};

use crate::util::SerdePersistant;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDefinition {
    pub name: String,
    pub checker: Option<String>,
    pub validator: Option<String>,
    pub main_solution: Option<String>,
    #[serde(default)]
    pub generators: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ProblemDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            checker: None,
            validator: None,
            main_solution: None,
            generators: Vec::new(),
            author: None,
            tags: Vec::new(),
        }
    }
}

impl ProblemDefinition {
    /// Sets `author` to `fallback` only when none is set (or it is blank),
    /// so an author that came with an imported problem is never overwritten.
    pub fn fill_missing_author(&mut self, fallback: &str) {
        let missing = self.author.as_deref().is_none_or(|a| a.trim().is_empty());
        if missing {
            self.author = Some(fallback.to_string());
        }
    }
}

impl SerdePersistant for ProblemDefinition {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_missing_author_sets_it_when_none_or_blank() {
        for author in [None, Some("  ".to_string())] {
            let mut def = ProblemDefinition::new("p");
            def.author = author;

            def.fill_missing_author("Global");

            assert_eq!(def.author.as_deref(), Some("Global"));
        }
    }

    #[test]
    fn fill_missing_author_keeps_an_existing_author() {
        let mut def = ProblemDefinition::new("p");
        def.author = Some("Daniel Porto".into());

        def.fill_missing_author("Global");

        assert_eq!(def.author.as_deref(), Some("Daniel Porto"));
    }
}
