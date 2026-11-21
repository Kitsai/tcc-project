use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{
    constants::{CHECKER_TESTS_PATH, MULT_SEPARATOR},
    error::{AppError, AppResult},
    fs::{delete_file, write_temp},
    problem::meta_tests::{self, MetaTest},
    runner::Runner,
    util::{
        assign_ids, next_available_id, EventEmitter, PathExt, Persistant, ResultExt,
        SerdePersistant,
    },
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

    pub fn create(problem_path: &Path, dto: CheckerTestCreateDto) -> AppResult<()> {
        let tests_path = problem_path.join(CHECKER_TESTS_PATH);

        if dto.mult {
            let inputs: Vec<&str> = dto.input.split(MULT_SEPARATOR).collect();
            let outputs: Vec<&str> = dto.output.split(MULT_SEPARATOR).collect();
            let answers: Vec<&str> = dto.answer.split(MULT_SEPARATOR).collect();
            let verdicts: Vec<CheckerVerdict> = dto
                .verdict
                .lines()
                .map(|v| v.parse())
                .collect::<Result<_, _>>()?;

            let len = inputs.len();
            if outputs.len() != len || answers.len() != len || verdicts.len() != len {
                return Err(AppError::from(
                    "Inputs, outputs, answers, and verdicts must have the same number of entries.",
                ));
            }

            let ids = assign_ids(&tests_path, dto.id, inputs.len())?;

            for ((((id, input), output), answer), verdict) in ids
                .into_iter()
                .zip(inputs)
                .zip(outputs)
                .zip(answers)
                .zip(verdicts)
            {
                let new_test = CheckerTest::new(id, input, output, answer, verdict);
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

            let new_test = Self::new(
                dto.id,
                &dto.input,
                &dto.output,
                &dto.answer,
                dto.verdict.parse()?,
            );
            new_test.save(&path)?;
        }

        Ok(())
    }

    pub fn update(problem_path: &Path, dto: CheckerTestEditDto) -> AppResult<()> {
        let path = problem_path.join(CHECKER_TESTS_PATH).join_id(dto.id);
        if path.exists() {
            let mut current = Self::load(&path)?;

            current.edit(&dto.input, &dto.output, &dto.answer, dto.verdict.parse()?);
            current.save(&path)
        } else {
            Err(AppError::from(format!(
                "Test with id {} does not exist",
                dto.id
            )))
        }
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

    pub fn get_all(problem_path: &Path) -> AppResult<Vec<CheckerTest>> {
        let mut ret = Vec::new();
        let path = problem_path.join(CHECKER_TESTS_PATH);

        let dir_entries = std::fs::read_dir(path).err_to_string()?;

        for entry in dir_entries.flatten() {
            ret.push(Self::load(&entry.path())?);
        }

        Ok(ret)
    }

    pub fn next_id(problem_path: &Path) -> u16 {
        let path = problem_path.join(CHECKER_TESTS_PATH);
        next_available_id(&path)
    }

    pub async fn run_all(
        problem_path: &Path,
        checker_path: PathBuf,
        emitter: impl EventEmitter,
        runner: std::sync::Arc<dyn Runner>,
    ) -> AppResult<()> {
        let tests = Self::get_all(problem_path)?;

        meta_tests::run_all(problem_path, tests, checker_path, emitter, runner).await
    }

    fn write_temp_files(test: &CheckerTest) -> AppResult<[NamedTempFile; 3]> {
        Ok([
            write_temp(&test.input)?,
            write_temp(&test.output)?,
            write_temp(&test.answer)?,
        ])
    }

    pub fn exists(problem_path: &Path, id: u16) -> bool {
        problem_path.join(CHECKER_TESTS_PATH).join_id(id).exists()
    }

    pub fn delete(problem_path: &Path, id: u16) -> AppResult<()> {
        delete_file(&problem_path.join(CHECKER_TESTS_PATH).join_id(id))
    }
}

impl MetaTest for CheckerTest {
    const EVENT_PREFIX: &'static str = "checker";

    const TESTS_PATH: &'static str = CHECKER_TESTS_PATH;

    fn id(&self) -> u16 {
        self.id
    }

    fn prepare(
        &self,
        request: &mut crate::runner::ExecutionRequest,
    ) -> AppResult<Vec<NamedTempFile>> {
        let files = Self::write_temp_files(self)?;

        request.with_args(
            &files
                .iter()
                .map(|f| f.path().to_string_lossy().to_string())
                .collect::<Vec<_>>(),
        );

        Ok(files.into())
    }

    fn record(&mut self, result: &crate::runner::ExecutionInfo) {
        let verdict = match result.exit_code {
            0 => CheckerVerdict::Ok,
            1 => CheckerVerdict::WrongAnswer,
            2 => CheckerVerdict::PresentationError,
            _ => CheckerVerdict::Crashed,
        };
        let comment = if !result.stdout.trim().is_empty() {
            result.stdout.trim().to_string()
        } else {
            result.stderr.trim().to_string()
        };

        self.set_actual_verdict(verdict, comment);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{temp_problem, MockRunner, RecordingEmitter};

    fn create_dto(id: u16, verdict: &str) -> CheckerTestCreateDto {
        CheckerTestCreateDto {
            id,
            mult: false,
            input: "in".to_string(),
            output: "out".to_string(),
            answer: "ans".to_string(),
            verdict: verdict.to_string(),
        }
    }

    #[test]
    fn verdict_parses_known_aliases() {
        assert!(matches!(
            "OK".parse::<CheckerVerdict>(),
            Ok(CheckerVerdict::Ok)
        ));
        assert!(matches!(
            "WA".parse::<CheckerVerdict>(),
            Ok(CheckerVerdict::WrongAnswer)
        ));
        assert!(matches!(
            "PE".parse::<CheckerVerdict>(),
            Ok(CheckerVerdict::PresentationError)
        ));
        assert!(matches!(
            "FL".parse::<CheckerVerdict>(),
            Ok(CheckerVerdict::Crashed)
        ));
        assert!(matches!(
            "".parse::<CheckerVerdict>(),
            Ok(CheckerVerdict::None)
        ));
    }

    #[test]
    fn verdict_rejects_unknown_values() {
        assert!("NOT_A_VERDICT".parse::<CheckerVerdict>().is_err());
    }

    #[test]
    fn create_single_test_writes_file() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let tests = CheckerTest::get_all(&problem.path).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].id, 1);
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let err = CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn create_mult_splits_batch_and_skips_taken_ids() {
        let (_dir, problem) = temp_problem("p");
        // pre-occupy id 2 so the batch below has to skip past it
        CheckerTest::create(&problem.path, create_dto(2, "OK")).unwrap();

        let dto = CheckerTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            output: "oa===ob".to_string(),
            answer: "aa===ab".to_string(),
            verdict: "OK\nWA".to_string(),
        };
        CheckerTest::create(&problem.path, dto).unwrap();

        let mut tests = CheckerTest::get_all(&problem.path).unwrap();
        tests.sort_by_key(|t| t.id);
        let ids: Vec<u16> = tests.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn create_mult_rejects_mismatched_lengths() {
        let (_dir, problem) = temp_problem("p");
        let dto = CheckerTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            output: "oa".to_string(),
            answer: "aa===ab".to_string(),
            verdict: "OK\nWA".to_string(),
        };

        let err = CheckerTest::create(&problem.path, dto).unwrap_err();
        assert!(err.to_string().contains("same number of entries"));
    }

    #[test]
    fn create_mult_with_an_invalid_verdict_stores_nothing() {
        let (_dir, problem) = temp_problem("p");
        let dto = CheckerTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b===c".to_string(),
            output: "oa===ob===oc".to_string(),
            answer: "aa===ab===ac".to_string(),
            verdict: "OK\nNOT_A_VERDICT\nWA".to_string(),
        };

        assert!(CheckerTest::create(&problem.path, dto).is_err());

        // the valid first entry must not have been written before the bad one failed
        assert!(CheckerTest::get_all(&problem.path).unwrap().is_empty());
    }

    #[test]
    fn create_mult_does_not_overwrite_a_test_at_the_first_id() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let dto = CheckerTestCreateDto {
            id: 1,
            mult: true,
            input: "a===b".to_string(),
            output: "oa===ob".to_string(),
            answer: "aa===ab".to_string(),
            verdict: "WA\nWA".to_string(),
        };
        CheckerTest::create(&problem.path, dto).unwrap();

        let mut tests = CheckerTest::get_all(&problem.path).unwrap();
        tests.sort_by_key(|t| t.id);

        // the original test is untouched and the batch went to the next free ids
        assert_eq!(tests.iter().map(|t| t.id).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(tests[0].input, "in");
        assert!(matches!(tests[0].expected, CheckerVerdict::Ok));
        assert_eq!(tests[1].input, "a");
        assert_eq!(tests[2].input, "b");
    }

    #[tokio::test]
    async fn run_all_maps_exit_codes_to_verdicts_and_persists() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::exit_code(1));
        let emitter = RecordingEmitter::default();

        CheckerTest::run_all(
            &problem.path,
            PathBuf::from("checker.py"),
            emitter.clone(),
            runner,
        )
        .await
        .unwrap();

        let tests = CheckerTest::get_all(&problem.path).unwrap();
        assert!(matches!(tests[0].actual, CheckerVerdict::WrongAnswer));

        let events = emitter.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "checker_test_result");
    }

    #[tokio::test]
    async fn run_all_reports_a_runner_failure_as_an_error_event_and_keeps_the_test() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::new(|_| {
            Err(crate::runner::ExecutionError::OTHER("boom".to_string()))
        }));
        let emitter = RecordingEmitter::default();

        CheckerTest::run_all(
            &problem.path,
            PathBuf::from("checker.py"),
            emitter.clone(),
            runner,
        )
        .await
        .unwrap();

        let events = emitter.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "checker_test_error");
        assert_eq!(events[0].1["id"], 1);
        assert_eq!(events[0].1["error"], "boom");

        // no verdict was recorded for the test that could not run
        let tests = CheckerTest::get_all(&problem.path).unwrap();
        assert!(matches!(tests[0].actual, CheckerVerdict::None));
    }

    /// Runs the checker once over a single stored test, with a runner that
    /// always returns the given result, and returns the test as persisted.
    async fn run_with(exit_code: i32, stdout: &str, stderr: &str) -> CheckerTest {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        let (stdout, stderr) = (stdout.to_string(), stderr.to_string());
        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::new(move |_| {
            Ok(crate::runner::ExecutionInfo {
                stdout: stdout.clone(),
                stderr: stderr.clone(),
                execution_time: std::time::Duration::default(),
                exit_code,
            })
        }));

        CheckerTest::run_all(
            &problem.path,
            PathBuf::from("checker.py"),
            RecordingEmitter::default(),
            runner,
        )
        .await
        .unwrap();

        CheckerTest::get_all(&problem.path).unwrap().remove(0)
    }

    #[tokio::test]
    async fn run_all_maps_every_exit_code_to_its_verdict() {
        assert!(matches!(
            run_with(0, "", "").await.actual,
            CheckerVerdict::Ok
        ));
        assert!(matches!(
            run_with(1, "", "").await.actual,
            CheckerVerdict::WrongAnswer
        ));
        assert!(matches!(
            run_with(2, "", "").await.actual,
            CheckerVerdict::PresentationError
        ));
        // testlib's `fail` exits with 3; anything unexpected counts as a crash
        assert!(matches!(
            run_with(3, "", "").await.actual,
            CheckerVerdict::Crashed
        ));
        assert!(matches!(
            run_with(-1, "", "").await.actual,
            CheckerVerdict::Crashed
        ));
    }

    #[tokio::test]
    async fn run_all_uses_stdout_as_the_comment_and_falls_back_to_stderr() {
        let both = run_with(1, "  wrong answer  \n", "from stderr").await;
        assert_eq!(both.comment, "wrong answer");

        let only_stderr = run_with(3, "  ", " from stderr \n").await;
        assert_eq!(only_stderr.comment, "from stderr");

        let neither = run_with(0, "", "").await;
        assert_eq!(neither.comment, "");
    }

    #[tokio::test]
    async fn run_all_keeps_the_input_files_alive_while_the_checker_runs() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        // the runner stands in for the checker: it must still find all three
        // files (input, output, answer) with the stored contents
        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::new(|request| {
            // `python3 checker.py <input> <output> <answer>`: the last three
            // arguments are the files
            let contents: Vec<String> = request.args[request.args.len() - 3..]
                .iter()
                .map(|path| std::fs::read_to_string(path).unwrap_or_else(|_| "<missing>".into()))
                .collect();
            assert_eq!(contents, ["in", "out", "ans"]);

            Ok(crate::runner::ExecutionInfo {
                stdout: String::new(),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code: 0,
            })
        }));
        let emitter = RecordingEmitter::default();

        CheckerTest::run_all(
            &problem.path,
            PathBuf::from("checker.py"),
            emitter.clone(),
            runner,
        )
        .await
        .unwrap();

        // a panic inside the task (failed assertion) would show up as no event
        let events = emitter.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "checker_test_result");
    }

    #[tokio::test]
    async fn run_all_runs_every_stored_test_with_its_own_data() {
        let (_dir, problem) = temp_problem("p");
        for (id, input) in [(1, "good"), (2, "bad")] {
            let dto = CheckerTestCreateDto {
                id,
                mult: false,
                input: input.to_string(),
                output: "out".to_string(),
                answer: "ans".to_string(),
                verdict: "OK".to_string(),
            };
            CheckerTest::create(&problem.path, dto).unwrap();
        }

        // the "checker" answers WA only for the test whose input file says "bad"
        // (`python3 checker.py <input> <output> <answer>`: input is the third-to-last arg)
        let runner: std::sync::Arc<dyn Runner> = std::sync::Arc::new(MockRunner::new(|request| {
            let input_path = &request.args[request.args.len() - 3];
            let input = std::fs::read_to_string(input_path).unwrap_or_default();
            let exit_code = if input.contains("bad") { 1 } else { 0 };
            Ok(crate::runner::ExecutionInfo {
                stdout: String::new(),
                stderr: String::new(),
                execution_time: std::time::Duration::default(),
                exit_code,
            })
        }));
        let emitter = RecordingEmitter::default();

        CheckerTest::run_all(
            &problem.path,
            PathBuf::from("checker.py"),
            emitter.clone(),
            runner,
        )
        .await
        .unwrap();

        // one result event per test (they run concurrently, so order is not fixed)
        let events = emitter.events();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.0 == "checker_test_result"));
        let mut event_ids: Vec<u64> = events.iter().map(|e| e.1["id"].as_u64().unwrap()).collect();
        event_ids.sort();
        assert_eq!(event_ids, [1, 2]);

        // and each stored test got the verdict of its own run
        let mut tests = CheckerTest::get_all(&problem.path).unwrap();
        tests.sort_by_key(|t| t.id);
        assert!(matches!(tests[0].actual, CheckerVerdict::Ok));
        assert!(matches!(tests[1].actual, CheckerVerdict::WrongAnswer));
    }

    fn edit_dto(id: u16, verdict: &str) -> CheckerTestEditDto {
        CheckerTestEditDto {
            id,
            input: "new in".to_string(),
            output: "new out".to_string(),
            answer: "new ans".to_string(),
            verdict: verdict.to_string(),
        }
    }

    #[test]
    fn update_replaces_the_stored_fields() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        CheckerTest::update(&problem.path, edit_dto(1, "WA")).unwrap();

        let test = CheckerTest::get_all(&problem.path).unwrap().remove(0);
        assert_eq!(test.input, "new in");
        assert_eq!(test.output, "new out");
        assert_eq!(test.answer, "new ans");
        assert!(matches!(test.expected, CheckerVerdict::WrongAnswer));
    }

    #[test]
    fn update_of_a_missing_test_is_an_error_and_creates_nothing() {
        let (_dir, problem) = temp_problem("p");

        let err = CheckerTest::update(&problem.path, edit_dto(5, "OK")).unwrap_err();

        assert!(err.to_string().contains("does not exist"));
        assert!(CheckerTest::get_all(&problem.path).unwrap().is_empty());
    }

    #[test]
    fn update_with_an_invalid_verdict_keeps_the_stored_test() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        assert!(CheckerTest::update(&problem.path, edit_dto(1, "NOT_A_VERDICT")).is_err());

        let test = CheckerTest::get_all(&problem.path).unwrap().remove(0);
        assert_eq!(test.input, "in");
        assert!(matches!(test.expected, CheckerVerdict::Ok));
    }

    #[test]
    fn exists_tells_whether_the_test_is_stored() {
        let (_dir, problem) = temp_problem("p");
        assert!(!CheckerTest::exists(&problem.path, 1));

        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        assert!(CheckerTest::exists(&problem.path, 1));
        assert!(!CheckerTest::exists(&problem.path, 2));
    }

    #[test]
    fn delete_removes_only_the_given_test() {
        let (_dir, problem) = temp_problem("p");
        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();
        CheckerTest::create(&problem.path, create_dto(2, "WA")).unwrap();

        CheckerTest::delete(&problem.path, 1).unwrap();

        assert!(!CheckerTest::exists(&problem.path, 1));
        assert!(CheckerTest::exists(&problem.path, 2));
    }

    #[test]
    fn delete_of_a_missing_test_is_an_error() {
        let (_dir, problem) = temp_problem("p");

        let err = CheckerTest::delete(&problem.path, 9).unwrap_err();

        assert!(err.to_string().contains("delete"));
    }

    #[test]
    fn next_id_is_the_first_free_id() {
        let (_dir, problem) = temp_problem("p");
        assert_eq!(CheckerTest::next_id(&problem.path), 1);

        CheckerTest::create(&problem.path, create_dto(1, "OK")).unwrap();

        assert_eq!(CheckerTest::next_id(&problem.path), 2);
    }
}
