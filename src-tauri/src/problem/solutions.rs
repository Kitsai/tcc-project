use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::webview::cookie::time::UtcDateTime;

use crate::{
    constants::SOLUTIONS_PATH,
    util::{Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionDescription {
    file_name: String,
    tag: SolutionTag,
    author: Option<String>,
    change_time: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolutionTag {
    Main,
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    TimeLimitExceededOrAccepted,
    TimeLimitExceededOrMemoryLimitExceeded,
    MemoryLimitExceeded,
    #[serde(rename = "")]
    None,
}

impl SerdePersistant for SolutionDescription {}

impl SolutionDescription {
    fn desc_path(project_path: &Path, file_name: &str) -> PathBuf {
        project_path
            .join(SOLUTIONS_PATH)
            .join(format!("{file_name}.desc"))
    }

    pub fn save_solution(&self, project_path: &Path) -> Result<(), String> {
        self.save(&Self::desc_path(project_path, &self.file_name))
    }

    pub fn load_all(project_path: &Path) -> Result<Vec<SolutionDescription>, String> {
        let solution_path = project_path.join(SOLUTIONS_PATH);

        let mut descriptions: Vec<SolutionDescription> = vec![];
        let mut sources: HashSet<String> = HashSet::new();

        let dir_entries = fs::read_dir(solution_path).err_to_string()?;

        for entry in dir_entries.flatten() {
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();

            if file_name.ends_with(".desc") {
                descriptions.push(Self::load(&entry.path())?);
            } else {
                sources.insert(file_name.into_owned());
            }
        }

        Self::verify_descriptions(project_path, descriptions, sources)
    }

    /// Reconciles descriptors with the source files actually present on disk:
    /// sources missing a descriptor get a fresh one, descriptors whose source
    /// was deleted get their `.desc` file removed.
    fn verify_descriptions(
        project_path: &Path,
        descriptions: Vec<SolutionDescription>,
        sources: HashSet<String>,
    ) -> Result<Vec<SolutionDescription>, String> {
        let mut verified: Vec<SolutionDescription> = vec![];

        for description in descriptions {
            if sources.contains(&description.file_name) {
                verified.push(description);
            } else {
                fs::remove_file(Self::desc_path(project_path, &description.file_name))
                    .err_to_string()?;
            }
        }

        let matched: HashSet<String> = verified.iter().map(|d| d.file_name.clone()).collect();

        for source in sources.difference(&matched) {
            let description = SolutionDescription {
                file_name: source.clone(),
                tag: SolutionTag::None,
                author: None,
                change_time: UtcDateTime::now().to_string(),
            };

            description.save_solution(project_path)?;
            verified.push(description);
        }

        Ok(verified)
    }

    pub fn delete_solution(project_path: &Path, file_name: String) -> Result<(), String> {
        let source_path = project_path.join(SOLUTIONS_PATH).join(&file_name);
        fs::remove_file(&source_path).err_to_string()?;
        fs::remove_file(Self::desc_path(project_path, &file_name)).err_to_string()
    }
}
