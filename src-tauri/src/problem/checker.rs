use std::{
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{CHECKER_TESTS_PATH, LANGUAGE_INVALID_ERR, MULT_SEPARATOR},
    problem::ProgrammingLanguage,
    runner::Runner,
    util::{next_available_id, EventEmitter, Persistant, ResultExt, SerdePersistant},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerTest {
    pub id: u16,
    pub input: String,
    pub output: String,
    pub answer: String,
    pub expected: CheckerVerdict,
    pub actual: CheckerVerdict,
    pub comment: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", try_from = "String")]
pub enum CheckerVerdict {
    Ok,
    WrongAnswer,
    PresentationError,
    Crashed,
    #[serde(rename = "")]
    None,
}

impl TryFrom<String> for CheckerVerdict {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_uppercase().trim() {
            "OK" => Ok(Self::Ok),
            "WRONG_ANSWER" | "WA" => Ok(Self::WrongAnswer),
            "PRESENTATION_ERROR" | "PE" => Ok(Self::PresentationError),
            "CRASHED" | "FL" => Ok(Self::Crashed),
            "NONE" | "" => Ok(Self::None),
            _ => Err(format!(
                "{} is not a valid verdict.\nPossible values: \"OK\", \"WRONG_ANSWER\" (\"WA\"), \"PRESENTATION_ERROR\" (\"PE\"), \"CRASHED\" (\"FL\") ",
                value
            )),
        }
    }
}

impl SerdePersistant for CheckerTest {}

impl CheckerTest {
    pub fn new(id: u16, input: &str, output: &str, answer: &str, expected: CheckerVerdict) -> Self {
        Self {
            id,
            input: input.trim().to_owned(),
            output: output.trim().to_owned(),
            answer: answer.trim().to_owned(),
            expected,
            actual: CheckerVerdict::None,
            comment: String::new(),
        }
    }

    pub fn create(dto: CheckerTestCreateDto, problem_path: &Path) -> Result<(), String> {
        let tests_path = problem_path.join(CHECKER_TESTS_PATH);

        if dto.mult {
            let inputs: Vec<&str> = dto.input.split(MULT_SEPARATOR).collect();
            let outputs: Vec<&str> = dto.output.split(MULT_SEPARATOR).collect();
            let answers: Vec<&str> = dto.answer.split(MULT_SEPARATOR).collect();
            let verdicts: Vec<&str> = dto.verdict.lines().collect();

            let len = inputs.len();
            if outputs.len() != len || answers.len() != len || verdicts.len() != len {
                return Err(
                    "Inputs, outputs, answers, and verdicts must have the same number of entries."
                        .to_string(),
                );
            }

            let mut current_id = dto.id;
            let mut current_path = tests_path.join(format!("{:02}", dto.id));

            for (((input, output), answer), verdict) in inputs
                .iter()
                .zip(outputs.iter())
                .zip(answers.iter())
                .zip(verdicts.iter())
            {
                let new_test =
                    CheckerTest::new(current_id, input, output, answer, verdict.parse()?);
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

            let new_test = Self::new(dto.id, &dto.input, &dto.output, &dto.answer, dto.verdict.parse()?);
            new_test.save(&path)?;
        }

        Ok(())
    }

    pub fn edit(&mut self, input: &str, output: &str, answer: &str, verdict: CheckerVerdict) {
        self.input = input.trim().to_string();
        self.output = output.trim().to_string();
        self.answer = answer.trim().to_string();
        self.expected = verdict;
    }

    pub fn set_actual_verdict(&mut self, actual: CheckerVerdict, comment: String) {
        self.actual = actual;
        self.comment = comment;
    }

    pub fn get_all(problem_path: &Path) -> Result<Vec<CheckerTest>, String> {
        let mut ret = Vec::new();
        let path = problem_path.join(CHECKER_TESTS_PATH);

        let dir_entries = std::fs::read_dir(path).err_to_string()?;

        for entry in dir_entries.flatten() {
            ret.push(Self::load(&entry.path())?);
        }

        Ok(ret)
    }

    pub async fn run_all(
        problem_path: &Path,
        checker_path: PathBuf,
        emitter: impl EventEmitter,
        runner: std::sync::Arc<dyn Runner>,
    ) -> Result<(), String> {
        let tests = Self::get_all(problem_path)?;
        let tests_path = problem_path.join(CHECKER_TESTS_PATH);

        let language = ProgrammingLanguage::get_from_path(&checker_path)
            .ok_or_else(|| LANGUAGE_INVALID_ERR.to_string())?;
        let request_template = language
            .resolve(&checker_path, problem_path)
            .into_request();

        log::debug!(
            "[checker run_all] checker={:?} command={:?} args={:?} tests={}",
            checker_path,
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
                let write_temp = |content: &str| -> Result<tempfile::NamedTempFile, String> {
                    let mut f = tempfile::NamedTempFile::new().err_to_string()?;
                    f.write_all(content.as_bytes()).err_to_string()?;
                    Ok(f)
                };

                let input_file = match write_temp(&test.input) {
                    Ok(f) => f,
                    Err(e) => {
                        emitter.emit(
                            "checker_test_error",
                            CheckerTestError { id: test.id, error: e },
                        );
                        return;
                    }
                };
                let output_file = match write_temp(&test.output) {
                    Ok(f) => f,
                    Err(e) => {
                        emitter.emit(
                            "checker_test_error",
                            CheckerTestError { id: test.id, error: e },
                        );
                        return;
                    }
                };
                let answer_file = match write_temp(&test.answer) {
                    Ok(f) => f,
                    Err(e) => {
                        emitter.emit(
                            "checker_test_error",
                            CheckerTestError { id: test.id, error: e },
                        );
                        return;
                    }
                };

                request.with_args(&[
                    input_file.path().to_string_lossy().to_string(),
                    output_file.path().to_string_lossy().to_string(),
                    answer_file.path().to_string_lossy().to_string(),
                ]);

                log::debug!("[checker run_all] running test id={}", test.id);

                let (actual, comment) = match runner.execute(request).await {
                    Err(e) => {
                        log::debug!(
                            "[checker run_all] test id={} runner error: {}",
                            test.id,
                            e
                        );
                        emitter.emit(
                            "checker_test_error",
                            CheckerTestError {
                                id: test.id,
                                error: e.to_string(),
                            },
                        );
                        return;
                    }
                    Ok(info) => {
                        log::debug!(
                            "[checker run_all] test id={} exit={} stdout={:?}",
                            test.id,
                            info.exit_code,
                            info.stdout.trim()
                        );
                        let verdict = match info.exit_code {
                            0 => CheckerVerdict::Ok,
                            1 => CheckerVerdict::WrongAnswer,
                            2 => CheckerVerdict::PresentationError,
                            _ => CheckerVerdict::Crashed,
                        };
                        let comment = if !info.stdout.trim().is_empty() {
                            info.stdout.trim().to_string()
                        } else {
                            info.stderr.trim().to_string()
                        };
                        (verdict, comment)
                    }
                };

                // keep temp files alive until after execution
                drop(input_file);
                drop(output_file);
                drop(answer_file);

                let mut updated = test;
                updated.set_actual_verdict(actual, comment);

                let path = tests_path.join(format!("{:02}", updated.id));
                updated.save(&path).ok();

                emitter.emit("checker_test_result", updated);
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
pub struct CheckerTestError {
    pub id: u16,
    pub error: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerTestCreateDto {
    pub id: u16,
    pub mult: bool,
    pub input: String,
    pub output: String,
    pub answer: String,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerTestEditDto {
    pub id: u16,
    pub input: String,
    pub output: String,
    pub answer: String,
    pub verdict: String,
}

impl std::str::FromStr for CheckerVerdict {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}
