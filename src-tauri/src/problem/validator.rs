use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{LANGUAGE_INVALID_ERR, MULT_SEPARATOR, VALIDATOR_TESTS_PATH},
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

    pub fn create(dto: ValidatorTestCreateDto, problem_path: &Path) -> Result<(), String> {
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        if dto.mult {
            let inputs: Vec<&str> = dto.input.split(MULT_SEPARATOR).collect();
            let verdicts: Vec<&str> = dto.verdict.lines().collect();

            if inputs.len() != verdicts.len() {
                return Err("Inputs and verdicts must have the same number of entries.".to_string());
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
                return Err(format!("Test with id {} already exists", dto.id));
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

    pub fn get_all(problem_path: &Path) -> Result<Vec<ValidatorTest>, String> {
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
    ) -> Result<(), String> {
        let tests = Self::get_all(problem_path)?;
        let tests_path = problem_path.join(VALIDATOR_TESTS_PATH);

        let language = ProgrammingLanguage::get_from_path(&validator_path)
            .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;
        let request_template = language.resolve(&validator_path, problem_path).into_request();

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
                let mut input = test.input.replace("\r\n", "\n");
                if cfg!(windows) {
                    input = input.replace('\n', "\r\n");
                    if !input.ends_with("\r\n") {
                        input.push_str("\r\n");
                    }
                } else if !input.ends_with('\n') {
                    input.push('\n');
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
