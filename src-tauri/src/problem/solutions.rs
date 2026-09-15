use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::SOLUTIONS_PATH,
    error::{AppError, AppResult},
    util::{self, Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SolutionDescription {
    pub file_name: String,
    pub tag: SolutionTag,
    pub author: Option<String>,
    pub change_time: String,
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
    fn save(&self, path: &Path) -> AppResult<()> {
        SolutionDescriptionFile::from(self).save(path)
    }

    fn load(path: &Path) -> AppResult<Self> {
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

    pub fn save_solution(&self, problem_path: &Path) -> AppResult<()> {
        self.save(&Self::desc_path(problem_path, &self.file_name))
    }

    fn load_descs(problem_path: &Path) -> AppResult<Vec<SolutionDescription>> {
        let solution_path = problem_path.join(SOLUTIONS_PATH);
        let mut descriptions: Vec<SolutionDescription> = vec![];

        let dir_entries = fs::read_dir(solution_path).err_to_string()?;
        for entry in dir_entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(".desc") {
                descriptions.push(Self::load(&entry.path())?);
            }
        }

        Ok(descriptions)
    }

    pub fn load_all(problem_path: &Path) -> AppResult<Vec<SolutionDescription>> {
        Self::load_descs(problem_path)
    }

    pub fn verify_and_load(problem_path: &Path) -> AppResult<Vec<SolutionDescription>> {
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

        let mut verified = Self::verify_descriptions(problem_path, descriptions, sources)?;
        Self::enforce_single_main(problem_path, &mut verified)?;
        Ok(verified)
    }

    fn enforce_single_main(problem_path: &Path, descriptions: &mut Vec<SolutionDescription>) -> AppResult<()> {
        let mut found_main = false;
        for desc in descriptions.iter_mut() {
            if matches!(desc.tag, SolutionTag::Main) {
                if found_main {
                    desc.tag = SolutionTag::Accepted;
                    desc.save_solution(problem_path)?;
                } else {
                    found_main = true;
                }
            }
        }
        Ok(())
    }

    pub fn change_tag(
        problem_path: &Path,
        file_name: &str,
        new_tag: SolutionTag,
    ) -> AppResult<Vec<SolutionDescription>> {
        if matches!(new_tag, SolutionTag::Main) {
            let mut descriptions = Self::load_descs(problem_path)?;

            for desc in &mut descriptions {
                if matches!(desc.tag, SolutionTag::Main) && desc.file_name != file_name {
                    desc.tag = SolutionTag::Accepted;
                    desc.save_solution(problem_path)?;
                }
            }

            match descriptions.iter_mut().find(|d| d.file_name == file_name) {
                Some(desc) => {
                    desc.tag = SolutionTag::Main;
                    desc.save_solution(problem_path)?;
                }
                None => return Err(AppError::from(format!("Solution '{file_name}' not found"))),
            }

            Ok(descriptions)
        } else {
            let path = Self::desc_path(problem_path, file_name);
            let mut desc = Self::load(&path)
                .map_err(|_| format!("Solution '{file_name}' not found"))?;
            desc.tag = new_tag;
            desc.save_solution(problem_path)?;
            Self::load_descs(problem_path)
        }
    }

    /// Reconciles descriptors with the source files actually present on disk:
    /// sources missing a descriptor get a fresh one, descriptors whose source
    /// was deleted get their `.desc` file removed.
    fn verify_descriptions(
        problem_path: &Path,
        descriptions: Vec<SolutionDescription>,
        sources: HashSet<String>,
    ) -> AppResult<Vec<SolutionDescription>> {
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

    pub fn delete_solution(project_path: &Path, file_name: String) -> AppResult<()> {
        let source_path = project_path.join(SOLUTIONS_PATH).join(&file_name);
        fs::remove_file(&source_path).err_to_string()?;
        fs::remove_file(Self::desc_path(project_path, &file_name)).err_to_string()?;
        Ok(())
    }

    pub fn create_new(file_name: String, problem_path: &Path) -> AppResult<()> {
        let destination = problem_path.join(SOLUTIONS_PATH).join(&file_name);

        fs::File::create_new(destination).err_to_string()?;

        Self::new(file_name).save_solution(problem_path)
    }

    /// Picks the file name of the solution tagged `Main`, if any.
    pub fn main_solution_file_name(solutions: &[SolutionDescription]) -> Option<String> {
        solutions
            .iter()
            .find(|s| matches!(s.tag, SolutionTag::Main))
            .map(|s| s.file_name.clone())
    }

    pub fn create_from_existing(full_path: PathBuf, problem_path: &Path) -> AppResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_problem;

    #[test]
    fn create_new_writes_source_and_desc() {
        let (_dir, problem) = temp_problem("p");
        SolutionDescription::create_new("main.cpp".to_string(), &problem.path).unwrap();

        assert!(problem.path.join(SOLUTIONS_PATH).join("main.cpp").exists());

        let all = SolutionDescription::load_all(&problem.path).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].file_name, "main.cpp");
        assert!(matches!(all[0].tag, SolutionTag::Accepted));
    }

    #[test]
    fn create_from_existing_copies_content() {
        let (_dir, problem) = temp_problem("p");
        let source_dir = tempfile::tempdir().unwrap();
        let source_file = source_dir.path().join("brute.cpp");
        std::fs::write(&source_file, "int main(){}").unwrap();

        SolutionDescription::create_from_existing(source_file, &problem.path).unwrap();

        let copied = problem.path.join(SOLUTIONS_PATH).join("brute.cpp");
        assert_eq!(std::fs::read_to_string(copied).unwrap(), "int main(){}");
    }

    #[test]
    fn verify_and_load_creates_desc_for_orphan_source_and_removes_stale_desc() {
        let (_dir, problem) = temp_problem("p");
        let solutions_dir = problem.path.join(SOLUTIONS_PATH);

        // orphan source file with no .desc
        std::fs::write(solutions_dir.join("orphan.cpp"), "").unwrap();

        // stale desc whose source no longer exists
        let stale = SolutionDescription::new("gone.cpp".to_string());
        stale.save_solution(&problem.path).unwrap();

        let verified = SolutionDescription::verify_and_load(&problem.path).unwrap();

        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].file_name, "orphan.cpp");
        assert!(matches!(verified[0].tag, SolutionTag::None));
        assert!(!solutions_dir.join("gone.cpp.desc").exists());
    }

    #[test]
    fn verify_and_load_demotes_a_second_main_to_accepted() {
        let (_dir, problem) = temp_problem("p");
        let solutions_dir = problem.path.join(SOLUTIONS_PATH);

        std::fs::write(solutions_dir.join("a.cpp"), "").unwrap();
        std::fs::write(solutions_dir.join("b.cpp"), "").unwrap();

        let mut a = SolutionDescription::new("a.cpp".to_string());
        a.tag = SolutionTag::Main;
        a.save_solution(&problem.path).unwrap();

        let mut b = SolutionDescription::new("b.cpp".to_string());
        b.tag = SolutionTag::Main;
        b.save_solution(&problem.path).unwrap();

        let verified = SolutionDescription::verify_and_load(&problem.path).unwrap();
        let mains: Vec<_> = verified
            .iter()
            .filter(|d| matches!(d.tag, SolutionTag::Main))
            .collect();
        assert_eq!(mains.len(), 1);
    }

    #[test]
    fn change_tag_to_main_demotes_previous_main() {
        let (_dir, problem) = temp_problem("p");
        SolutionDescription::create_new("a.cpp".to_string(), &problem.path).unwrap();
        SolutionDescription::create_new("b.cpp".to_string(), &problem.path).unwrap();

        SolutionDescription::change_tag(&problem.path, "a.cpp", SolutionTag::Main).unwrap();
        let descs = SolutionDescription::change_tag(&problem.path, "b.cpp", SolutionTag::Main).unwrap();

        let a = descs.iter().find(|d| d.file_name == "a.cpp").unwrap();
        let b = descs.iter().find(|d| d.file_name == "b.cpp").unwrap();
        assert!(matches!(a.tag, SolutionTag::Accepted));
        assert!(matches!(b.tag, SolutionTag::Main));
    }

    #[test]
    fn change_tag_errors_for_unknown_file() {
        let (_dir, problem) = temp_problem("p");
        match SolutionDescription::change_tag(&problem.path, "missing.cpp", SolutionTag::Main) {
            Err(err) => assert!(err.to_string().contains("not found")),
            Ok(_) => panic!("expected an error for an unknown solution file"),
        }
    }

    #[test]
    fn delete_solution_removes_source_and_desc() {
        let (_dir, problem) = temp_problem("p");
        SolutionDescription::create_new("a.cpp".to_string(), &problem.path).unwrap();

        SolutionDescription::delete_solution(&problem.path, "a.cpp".to_string()).unwrap();

        let solutions_dir = problem.path.join(SOLUTIONS_PATH);
        assert!(!solutions_dir.join("a.cpp").exists());
        assert!(!solutions_dir.join("a.cpp.desc").exists());
    }
}
