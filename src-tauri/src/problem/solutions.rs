use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::SOLUTIONS_PATH,
    util::{self, Persistant, ResultExt, SerdePersistant, StringResult},
};

#[derive(Clone, Serialize, Deserialize)]
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

/// On-disk shape of `SolutionDescription`: the `.desc` file is camelCase,
/// while the struct itself stays snake_case for Tauri IPC with the frontend.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolutionDescriptionFile {
    file_name: String,
    tag: SolutionTag,
    author: Option<String>,
    change_time: String,
}

impl SerdePersistant for SolutionDescriptionFile {}

impl From<&SolutionDescription> for SolutionDescriptionFile {
    fn from(desc: &SolutionDescription) -> Self {
        Self {
            file_name: desc.file_name.clone(),
            tag: desc.tag.clone(),
            author: desc.author.clone(),
            change_time: desc.change_time.clone(),
        }
    }
}

impl From<SolutionDescriptionFile> for SolutionDescription {
    fn from(file: SolutionDescriptionFile) -> Self {
        Self {
            file_name: file.file_name,
            tag: file.tag,
            author: file.author,
            change_time: file.change_time,
        }
    }
}

impl Persistant for SolutionDescription {
    fn save(&self, path: &Path) -> Result<(), String> {
        SolutionDescriptionFile::from(self).save(path)
    }

    fn load(path: &Path) -> Result<Self, String> {
        SolutionDescriptionFile::load(path).map(Into::into)
    }
}

impl SolutionDescription {
    pub fn new(file_name: String) -> Self {
        Self {
            file_name,
            author: None,
            tag: SolutionTag::Accepted,
            change_time: util::now(),
        }
    }

    fn desc_path(problem_path: &Path, file_name: &str) -> PathBuf {
        problem_path
            .join(SOLUTIONS_PATH)
            .join(format!("{file_name}.desc"))
    }

    pub fn save_solution(&self, problem_path: &Path) -> Result<(), String> {
        self.save(&Self::desc_path(problem_path, &self.file_name))
    }

    pub fn load_all(problem_path: &Path) -> Result<Vec<SolutionDescription>, String> {
        let solution_path = problem_path.join(SOLUTIONS_PATH);

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

        Self::verify_descriptions(problem_path, descriptions, sources)
    }

    /// Reconciles descriptors with the source files actually present on disk:
    /// sources missing a descriptor get a fresh one, descriptors whose source
    /// was deleted get their `.desc` file removed.
    fn verify_descriptions(
        problem_path: &Path,
        descriptions: Vec<SolutionDescription>,
        sources: HashSet<String>,
    ) -> Result<Vec<SolutionDescription>, String> {
        let mut verified: Vec<SolutionDescription> = vec![];

        for description in descriptions {
            if sources.contains(&description.file_name) {
                verified.push(description);
            } else {
                fs::remove_file(Self::desc_path(problem_path, &description.file_name))
                    .err_to_string()?;
            }
        }

        let matched: HashSet<String> = verified.iter().map(|d| d.file_name.clone()).collect();

        for source in sources.difference(&matched) {
            let description = SolutionDescription {
                file_name: source.clone(),
                tag: SolutionTag::None,
                author: None,
                change_time: util::now(),
            };

            description.save_solution(problem_path)?;
            verified.push(description);
        }

        Ok(verified)
    }

    pub fn delete_solution(project_path: &Path, file_name: String) -> Result<(), String> {
        let source_path = project_path.join(SOLUTIONS_PATH).join(&file_name);
        fs::remove_file(&source_path).err_to_string()?;
        fs::remove_file(Self::desc_path(project_path, &file_name)).err_to_string()
    }

    pub fn create_new(file_name: String, problem_path: &Path) -> StringResult<()> {
        let destination = problem_path.join(SOLUTIONS_PATH).join(&file_name);

        fs::File::create_new(destination).err_to_string()?;

        Self::new(file_name).save_solution(problem_path)
    }

    pub fn create_from_existing(full_path: PathBuf, problem_path: &Path) -> StringResult<()> {
        let file_name = full_path
            .file_name()
            .ok_or("Path has no file name")?
            .to_string_lossy()
            .into_owned();

        let solutions_path = problem_path.join(SOLUTIONS_PATH);
        let destination = solutions_path.join(&file_name);

        fs::copy(full_path, destination).err_to_string()?;

        Self::new(file_name).save_solution(problem_path)
    }
}
