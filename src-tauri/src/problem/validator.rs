use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{LANGUAGE_INVALID_ERR, MULT_SEPARATOR, VALIDATOR_TESTS_PATH},
    error::{AppError, AppResult},
    problem::ProgrammingLanguage,
    runner::Runner,
    util::{next_available_id, EventEmitter, Persistant, ResultExt, SerdePersistant},
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

    pub fn create(dto: ValidatorTestCreateDto, problem_path: &Path) -> AppResult<()> {
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        if dto.mult {
            let inputs: Vec<&str> = dto.input.split(MULT_SEPARATOR).collect();
            let verdicts: Vec<&str> = dto.verdict.lines().collect();

            if inputs.len() != verdicts.len() {
                return Err(AppError::from(
                    "Inputs and verdicts must have the same number of entries.",
                ));
            }

            let mut current_id = dto.id;
            let mut current_path = tests_path.join(format!("{:02}", dto.id));

            for (input, verdict) in inputs.iter().zip(verdicts.iter()) {
                let new_test = ValidatorTest::new(current_id, input, verdict.parse()?);
                new_test.save(&current_path)?;

                current_id += 1;
                current_path = tests_path.join(format!("{:02}", current_id));

                if current_path.exists() {
                    current_id = next_available_id(&tests_path);
                    current_path = tests_path.join(format!("{:02}", current_id));
                }
            }
        } else {
            let path = tests_path.join(format!("{:02}", dto.id));

            if path.exists() {
                return Err(AppError::from(format!("Test with id {} already exists", dto.id)));
            }

            let new_test = Self::new(dto.id, &dto.input, dto.verdict.parse()?);
            new_test.save(&path)?;
        }

        Ok(())
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

    pub async fn run_all(
        problem_path: &Path,
        validator_path: PathBuf,
        emitter: impl EventEmitter,
        runner: std::sync::Arc<dyn Runner>,
    ) -> AppResult<()> {
        let tests = Self::get_all(problem_path)?;
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        let language = ProgrammingLanguage::get_from_path(&validator_path)
            .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;
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
                let mut input = test.input.trim().replace("\r\n", "\n");
                input.push('\n');
                if cfg!(windows) {
                    input = input.replace('\n', "\r\n");
                }
                request.with_input(&input);

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

                let path = tests_path.join(format!("{:02}", updated.id));
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
        assert!(matches!("VALID".parse::<ValidatorTestResult>(), Ok(ValidatorTestResult::Valid)));
        assert!(matches!(
            "invalid".parse::<ValidatorTestResult>(),
            Ok(ValidatorTestResult::Invalid)
        ));
        assert!(matches!("".parse::<ValidatorTestResult>(), Ok(ValidatorTestResult::None)));
    }

    #[test]
    fn result_rejects_unknown_values() {
        assert!("MAYBE".parse::<ValidatorTestResult>().is_err());
    }

    #[test]
    fn create_single_test_writes_file() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(create_dto(1, "VALID"), &problem.path).unwrap();

        let tests = ValidatorTest::get_all(&problem.path).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].id, 1);
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(create_dto(1, "VALID"), &problem.path).unwrap();

        let err = ValidatorTest::create(create_dto(1, "VALID"), &problem.path).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn create_mult_splits_batch_and_skips_taken_ids() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(create_dto(2, "VALID"), &problem.path).unwrap();

        let dto = ValidatorTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            verdict: "VALID\nINVALID".to_string(),
        };
        ValidatorTest::create(dto, &problem.path).unwrap();

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

        let err = ValidatorTest::create(dto, &problem.path).unwrap_err();
        assert!(err.to_string().contains("same number of entries"));
    }

    #[tokio::test]
    async fn run_all_maps_exit_codes_to_results_and_persists() {
        let (_dir, problem) = temp_problem("p");
        ValidatorTest::create(create_dto(1, "VALID"), &problem.path).unwrap();

        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::exit_code(1));
        let emitter = RecordingEmitter::default();

        ValidatorTest::run_all(&problem.path, PathBuf::from("validator.py"), emitter.clone(), runner)
            .await
            .unwrap();

        let tests = ValidatorTest::get_all(&problem.path).unwrap();
        assert!(matches!(tests[0].actual, ValidatorTestResult::Invalid));

        let events = emitter.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "validator_test_result");
    }
}
