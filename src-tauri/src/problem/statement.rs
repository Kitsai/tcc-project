use std::fs;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppResult,
    util::{Persistant, ResultExt},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProblemStatement {
    pub name: String,
    pub legend: String,
    pub input: String,
    pub output: String,
    pub notes: String,
    pub tutorial: String,
}

impl ProblemStatement {
    pub fn new(name: &str) -> Self {
        ProblemStatement {
            name: name.to_string(),
            legend: String::new(),
            input: String::new(),
            output: String::new(),
            notes: String::new(),
            tutorial: String::new(),
        }
    }
}

/// Statement fragments, in the order they are saved. Each is stored as a
/// Markdown file (`<field>.md`) holding the TipTap Markdown of that section.
const FIELDS: [&str; 6] = ["name", "legend", "input", "output", "notes", "tutorial"];

/// Reads `<field>.md`, falling back to the legacy `<field>.tex` that older
/// projects were saved with (their content is already Markdown-compatible text).
fn read_fragment(statement_dir: &std::path::Path, field: &str) -> AppResult<String> {
    let md = statement_dir.join(format!("{field}.md"));
    if md.exists() {
        return Ok(fs::read_to_string(md).err_to_string()?);
    }
    Ok(fs::read_to_string(statement_dir.join(format!("{field}.tex"))).err_to_string()?)
}

impl Persistant for ProblemStatement {
    fn save(&self, base_path: &std::path::Path) -> AppResult<()> {
        let statement_dir = base_path.join("statement");
        debug!("Creating markdown files at {:?}", statement_dir);

        let values = [
            &self.name,
            &self.legend,
            &self.input,
            &self.output,
            &self.notes,
            &self.tutorial,
        ];
        for (field, value) in FIELDS.iter().zip(values) {
            fs::write(statement_dir.join(format!("{field}.md")), value).err_to_string()?;
            // Migration: once the .md exists, the legacy .tex is stale.
            let legacy = statement_dir.join(format!("{field}.tex"));
            if legacy.exists() {
                fs::remove_file(legacy).err_to_string()?;
            }
        }

        debug!("Files created");

        Ok(())
    }

    fn load(base_path: &std::path::Path) -> AppResult<Self> {
        let statement_dir = base_path.join("statement");

        let name = read_fragment(&statement_dir, "name")?;
        let legend = read_fragment(&statement_dir, "legend")?;
        debug!("Loaded legend with content {}", legend);
        let input = read_fragment(&statement_dir, "input")?;
        let output = read_fragment(&statement_dir, "output")?;
        let notes = read_fragment(&statement_dir, "notes")?;
        let tutorial = read_fragment(&statement_dir, "tutorial")?;

        Ok(ProblemStatement {
            name,
            legend,
            input,
            output,
            notes,
            tutorial,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("statement")).unwrap();

        let stmt = ProblemStatement {
            name: "Problem Name".to_string(),
            legend: "Legend".to_string(),
            input: "Input spec".to_string(),
            output: "Output spec".to_string(),
            notes: "Notes".to_string(),
            tutorial: "Tutorial".to_string(),
        };
        stmt.save(dir.path()).unwrap();

        let loaded = ProblemStatement::load(dir.path()).unwrap();
        assert_eq!(loaded.name, stmt.name);
        assert_eq!(loaded.legend, stmt.legend);
        assert_eq!(loaded.input, stmt.input);
        assert_eq!(loaded.output, stmt.output);
        assert_eq!(loaded.notes, stmt.notes);
        assert_eq!(loaded.tutorial, stmt.tutorial);
    }

    #[test]
    fn load_fails_when_statement_dir_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ProblemStatement::load(dir.path()).is_err());
    }

    #[test]
    fn saves_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("statement")).unwrap();
        ProblemStatement::new("P").save(dir.path()).unwrap();

        for field in FIELDS {
            assert!(dir.path().join(format!("statement/{field}.md")).exists());
            assert!(!dir.path().join(format!("statement/{field}.tex")).exists());
        }
    }

    #[test]
    fn loads_legacy_tex_and_migrates_on_save() {
        let dir = tempfile::tempdir().unwrap();
        let sdir = dir.path().join("statement");
        std::fs::create_dir_all(&sdir).unwrap();
        for field in FIELDS {
            std::fs::write(sdir.join(format!("{field}.tex")), format!("old {field}")).unwrap();
        }

        let loaded = ProblemStatement::load(dir.path()).unwrap();
        assert_eq!(loaded.legend, "old legend");

        loaded.save(dir.path()).unwrap();
        for field in FIELDS {
            assert!(sdir.join(format!("{field}.md")).exists());
            assert!(!sdir.join(format!("{field}.tex")).exists());
        }
        assert_eq!(ProblemStatement::load(dir.path()).unwrap().notes, "old notes");
    }

    #[test]
    fn md_takes_precedence_over_legacy_tex() {
        let dir = tempfile::tempdir().unwrap();
        let sdir = dir.path().join("statement");
        std::fs::create_dir_all(&sdir).unwrap();
        for field in FIELDS {
            std::fs::write(sdir.join(format!("{field}.tex")), "tex").unwrap();
            std::fs::write(sdir.join(format!("{field}.md")), "md").unwrap();
        }
        assert_eq!(ProblemStatement::load(dir.path()).unwrap().input, "md");
    }
}
