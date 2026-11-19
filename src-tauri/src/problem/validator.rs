use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{MULT_SEPARATOR, VALIDATOR_TESTS_PATH},
    error::{AppError, AppResult},
    fs::delete_file,
    problem::ProgrammingLanguage,
    runner::Runner,
    util::{
        assign_ids, next_available_id, EventEmitter, PathExt, Persistant, ResultExt,
        SerdePersistant,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ValidatorTest {
    pub id: u16,
    pub input: String,
    pub expected: ValidatorTestResult,
    pub actual: ValidatorTestResult,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", try_from = "String")]
pub enum ValidatorTestResult {
    Valid,
    Invalid,
    #[serde(rename = "")]
    None,
}

impl TryFrom<String> for ValidatorTestResult {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_uppercase().trim() {
            "VALID" => Ok(Self::Valid),
            "INVALID" => Ok(Self::Invalid),
            "NONE" | "" => Ok(Self::None),
            _ => Err(format!(
                "\"{}\" is not a valid result; expected VALID, INVALID, or empty",
                s
            )),
        }
    }
}

impl std::str::FromStr for ValidatorTestResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl SerdePersistant for ValidatorTest {}

impl ValidatorTest {
    pub fn new(id: u16, input: &str, expected: ValidatorTestResult) -> Self {
        Self {
            id,
            input: input.trim().to_string(),
            expected,
            actual: ValidatorTestResult::None,
        }
    }

    pub fn create(problem_path: &Path, dto: ValidatorTestCreateDto) -> AppResult<()> {
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        if dto.mult {
            let inputs: Vec<&str> = dto.input.split(MULT_SEPARATOR).collect();
            let verdicts: Vec<ValidatorTestResult> = dto
                .verdict
                .lines()
                .map(|v| v.parse())
                .collect::<Result<_, _>>()?;

            if inputs.len() != verdicts.len() {
                return Err(AppError::from(
                    "Inputs and verdicts must have the same number of entries.",
                ));
            }

            let ids = assign_ids(&tests_path, dto.id, inputs.len())?;

            for ((id, input), verdict) in ids.into_iter().zip(inputs).zip(verdicts) {
                let new_test = ValidatorTest::new(id, input, verdict);
                new_test.save(&tests_path.join_id(id))?;
            }
        } else {
            let path = tests_path.join_id(dto.id);

            if path.exists() {
                return Err(AppError::from(format!(
                    "Test with id {} already exists",
                    dto.id
                )));
            }

            let new_test = Self::new(dto.id, &dto.input, dto.verdict.parse()?);
            new_test.save(&path)?;
        }

        Ok(())
    }

    pub fn update(problem_path: &Path, dto: ValidatorTestEditDto) -> AppResult<()> {
        if !Self::exists(problem_path, dto.id) {
            return Err(AppError::from(format!(
                "Test with id {} does not exist",
                dto.id
            )));
        }

        let path = problem_path.join(VALIDATOR_TESTS_PATH).join_id(dto.id);
        let mut current = Self::load(&path)?;

        current.edit(&dto.input, dto.verdict.parse()?);
        current.save(&path)
    }

    pub fn edit(&mut self, input: &str, verdict: ValidatorTestResult) {
        self.input = input.trim().to_string();
        self.expected = verdict;
    }

    pub fn set_actual_verdict(&mut self, actual: ValidatorTestResult) {
        self.actual = actual;
    }

    pub fn get_all(problem_path: &Path) -> AppResult<Vec<ValidatorTest>> {
        let mut ret = Vec::new();
        let path = problem_path.join(VALIDATOR_TESTS_PATH);

        let dir_entries = std::fs::read_dir(path).err_to_string()?;

        for entry in dir_entries.flatten() {
            ret.push(Self::load(&entry.path())?);
        }

        Ok(ret)
    }

    pub fn next_id(problem_path: &Path) -> u16 {
        next_available_id(&problem_path.join(VALIDATOR_TESTS_PATH))
    }

    pub fn exists(problem_path: &Path, id: u16) -> bool {
        problem_path.join(VALIDATOR_TESTS_PATH).join_id(id).exists()
    }

    pub fn delete(problem_path: &Path, id: u16) -> AppResult<()> {
        delete_file(&problem_path.join(VALIDATOR_TESTS_PATH).join_id(id))
    }

    pub async fn run_all(
        problem_path: &Path,
        validator_path: PathBuf,
        emitter: impl EventEmitter,
        runner: std::sync::Arc<dyn Runner>,
    ) -> AppResult<()> {
        let tests = Self::get_all(problem_path)?;
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        let language = ProgrammingLanguage::get_from_path(&validator_path).ok_or_else(|| {
            AppError::InvalidLanguage {
                path: validator_path.clone(),
            }
        })?;
        let request_template = language
            .resolve(&validator_path, problem_path)
            .into_request();

        log::debug!(
            "[run_all] validator={:?} command={:?} args={:?} tests={}",
            validator_path,
            request_template.command,
            request_template.args,
            tests.len()
        );

        let mut handles = Vec::new();

        for test in tests {
            let runner = runner.clone();
            let emitter = emitter.clone();
            let tests_path = tests_path.clone();
            let mut request = request_template.clone();

            let handle = tokio::spawn(async move {
                request.with_normalized_input(&test.input);

                log::debug!("[run_all] running test id={}", test.id);

                let actual = match runner.execute(request).await {
                    Err(e) => {
                        log::debug!("[run_all] test id={} runner error: {}", test.id, e);
                        emitter.emit(
                            "validator_test_error",
                            ValidatorTestError {
                                id: test.id,
                                error: e.to_string(),
                            },
                        );
                        return;
                    }
                    Ok(info) => {
                        log::debug!(
                            "[run_all] test id={} exit={} stderr={:?}",
                            test.id,
                            info.exit_code,
                            info.stderr.trim()
                        );
                        if info.exit_code == 0 {
                            ValidatorTestResult::Valid
                        } else {
                            ValidatorTestResult::Invalid
                        }
                    }
                };

                let mut updated = test;
                updated.set_actual_verdict(actual);

                let path = tests_path.join_id(updated.id);
                updated.save(&path).ok();

                emitter.emit("validator_test_result", updated);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.ok();
        }

        Ok(())
    }
}

#[derive(Clone, Serialize)]
pub struct ValidatorTestError {
    pub id: u16,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorTestCreateDto {
    pub id: u16,
    pub mult: bool,
    pub input: String,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorTestEditDto {
    pub id: u16,
    pub input: String,
    pub verdict: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{temp_problem, MockRunner, RecordingEmitter};

    fn create_dto(id: u16, verdict: &str) -> ValidatorTestCreateDto {
        ValidatorTestCreateDto {
            id,
            mult: false,
            input: "in".to_string(),
            verdict: verdict.to_string(),
        }
    }

    #[test]
    fn result_parses_known_values() {
        assert!(matches!(
            "VALID".parse::<ValidatorTestResult>(),
            Ok(ValidatorTestResult::Valid)
        ));
        assert!(matches!(
            "invalid".parse::<ValidatorTestResult>(),
            Ok(ValidatorTestResult::Invalid)
        ));
        assert!(matches!(
            "".parse::<ValidatorTestResult>(),
            Ok(ValidatorTestResult::None)
        ));
    }

    #[test]
    fn result_rejects_unknown_values() {
        assert!("MAYBE".parse::<ValidatorTestResult>().is_err());
    }

    #[test]
    fn create_single_test_writes_file() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        let tests = ValidatorTest::get_all(&problem.path).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].id, 1);
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        let err = ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn create_mult_splits_batch_and_skips_taken_ids() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(2, "VALID")).unwrap();

        let dto = ValidatorTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            verdict: "VALID\nINVALID".to_string(),
        };
        ValidatorTest::create(&problem.path, dto).unwrap();

        let mut tests = ValidatorTest::get_all(&problem.path).unwrap();
        tests.sort_by_key(|t| t.id);
        let ids: Vec<u16> = tests.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn create_mult_rejects_mismatched_lengths() {
        let (_dir, problem) = temp_problem("p");
        let dto = ValidatorTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            verdict: "VALID".to_string(),
        };

        let err = ValidatorTest::create(&problem.path, dto).unwrap_err();
        assert!(err.to_string().contains("same number of entries"));
    }

    #[test]
    fn create_mult_with_an_invalid_verdict_stores_nothing() {
        let (_dir, problem) = temp_problem("p");
        let dto = ValidatorTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b===c".to_string(),
            verdict: "VALID\nMAYBE\nINVALID".to_string(),
        };

        assert!(ValidatorTest::create(&problem.path, dto).is_err());

        // the valid first entry must not have been written before the bad one failed
        assert!(ValidatorTest::get_all(&problem.path).unwrap().is_empty());
    }

    #[test]
    fn create_mult_does_not_overwrite_a_test_at_the_first_id() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        let dto = ValidatorTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            verdict: "INVALID\nINVALID".to_string(),
        };
        ValidatorTest::create(&problem.path, dto).unwrap();

        let mut tests = ValidatorTest::get_all(&problem.path).unwrap();
        tests.sort_by_key(|t| t.id);

        // the original test is untouched and the batch went to the next free ids
        assert_eq!(tests.iter().map(|t| t.id).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(tests[0].input, "in");
        assert!(matches!(tests[0].expected, ValidatorTestResult::Valid));
        assert_eq!(tests[1].input, "a");
        assert_eq!(tests[2].input, "b");
    }

    #[tokio::test]
    async fn run_all_maps_exit_codes_to_results_and_persists() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::exit_code(1));
        let emitter = RecordingEmitter::default();

        ValidatorTest::run_all(
            &problem.path,
            PathBuf::from("validator.py"),
            emitter.clone(),
            runner,
        )
        .await
        .unwrap();

        let tests = ValidatorTest::get_all(&problem.path).unwrap();
        assert!(matches!(tests[0].actual, ValidatorTestResult::Invalid));

        let events = emitter.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "validator_test_result");
    }

    fn edit_dto(id: u16, verdict: &str) -> ValidatorTestEditDto {
        ValidatorTestEditDto {
            id,
            input: "new in".to_string(),
            verdict: verdict.to_string(),
        }
    }

    #[test]
    fn update_replaces_the_stored_fields() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        ValidatorTest::update(&problem.path, edit_dto(1, "INVALID")).unwrap();

        let test = ValidatorTest::get_all(&problem.path).unwrap().remove(0);
        assert_eq!(test.input, "new in");
        assert!(matches!(test.expected, ValidatorTestResult::Invalid));
    }

    #[test]
    fn update_of_a_missing_test_is_an_error_and_creates_nothing() {
        let (_dir, problem) = temp_problem("p");

        let err = ValidatorTest::update(&problem.path, edit_dto(5, "VALID")).unwrap_err();

        assert!(err.to_string().contains("does not exist"));
        assert!(ValidatorTest::get_all(&problem.path).unwrap().is_empty());
    }

    #[test]
    fn update_with_an_invalid_verdict_keeps_the_stored_test() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        assert!(ValidatorTest::update(&problem.path, edit_dto(1, "MAYBE")).is_err());

        let test = ValidatorTest::get_all(&problem.path).unwrap().remove(0);
        assert_eq!(test.input, "in");
        assert!(matches!(test.expected, ValidatorTestResult::Valid));
    }

    #[test]
    fn exists_tells_whether_the_test_is_stored() {
        let (_dir, problem) = temp_problem("p");
        assert!(!ValidatorTest::exists(&problem.path, 1));

        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        assert!(ValidatorTest::exists(&problem.path, 1));
        assert!(!ValidatorTest::exists(&problem.path, 2));
    }

    #[test]
    fn delete_removes_only_the_given_test() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();
        ValidatorTest::create(&problem.path, create_dto(2, "INVALID")).unwrap();

        ValidatorTest::delete(&problem.path, 1).unwrap();

        assert!(!ValidatorTest::exists(&problem.path, 1));
        assert!(ValidatorTest::exists(&problem.path, 2));
    }

    #[test]
    fn delete_of_a_missing_test_is_an_error() {
        let (_dir, problem) = temp_problem("p");

        let err = ValidatorTest::delete(&problem.path, 9).unwrap_err();

        assert!(err.to_string().contains("delete"));
    }

    #[test]
    fn next_id_is_the_first_free_id() {
        let (_dir, problem) = temp_problem("p");
        assert_eq!(ValidatorTest::next_id(&problem.path), 1);

        ValidatorTest::create(&problem.path, create_dto(1, "VALID")).unwrap();

        assert_eq!(ValidatorTest::next_id(&problem.path), 2);
    }
}
