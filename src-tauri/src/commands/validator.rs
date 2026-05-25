use std::fs;

use tauri::State;

use crate::{
    constants::{MULT_SEPARATOR, VALIDATOR_TESTS_PATH},
    problem::{ProblemManager, ValidatorTest, ValidatorTestCreateDto, ValidatorTestResult},
    util::{next_available_id, Persistant, ResultExt},
};

#[tauri::command]
pub fn get_validator_tests(state: State<ProblemManager>) -> Result<Vec<ValidatorTest>, String> {
    let mut tests: Vec<ValidatorTest> = Vec::new();

    let path = state.get_current_path()?.join("tests/validator");

    let dir_entries = fs::read_dir(path).err_to_string()?;

    for entry in dir_entries.flatten() {
        tests.push(ValidatorTest::load(&entry.path())?);
    }

    Ok(tests)
}

#[tauri::command]
pub fn get_next_validator_test_id(state: State<ProblemManager>) -> Result<u16, String> {
    let path = state.get_current_path()?.join(VALIDATOR_TESTS_PATH);
    Ok(next_available_id(&path))
}

#[tauri::command]
pub fn validator_test_exists(id: u16, state: State<ProblemManager>) -> Result<bool, String> {
    let path = state
        .get_current_path()?
        .join(VALIDATOR_TESTS_PATH)
        .join(format!("{:02}", id));

    Ok(path.exists())
}

#[tauri::command]
pub fn create_validator_test(
    test: ValidatorTestCreateDto,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let base_path = state.get_current_path()?.join(VALIDATOR_TESTS_PATH);
    let path = base_path.join(format!("{:02}", test.id));

    if path.exists() {
        return Err(format!("Test with id {} already exists", test.id));
    }

    if test.mult {
        let inputs: Vec<&str> = test.input.split(MULT_SEPARATOR).collect();
        let verdicts: Vec<&str> = test.verdict.lines().collect();
        let mut current_id = test.id;
        let mut current_path = path;

        if inputs.len() != verdicts.len() {
            return Err("Inputs and verdicts must have the same number of entries.".to_string());
        }

        for (input, verdict) in inputs.iter().zip(verdicts.iter()) {
            let new_test = ValidatorTest {
                id: current_id,
                input: input.to_string(),
                expected: verdict.parse()?,
                actual: ValidatorTestResult::None,
            };

            new_test.save(&current_path)?;
            current_id += 1;
            current_path = base_path.join(format!("{:02}", current_id));
            if current_path.exists() {
                current_id = next_available_id(&base_path);
                current_path = base_path.join(format!("{:02}", current_id));
            }
        }
    } else {
        let new_test = ValidatorTest {
            id: test.id,
            input: test.input,
            expected: test.verdict.parse()?,
            actual: ValidatorTestResult::None,
        };
        new_test.save(&path)?;
    }

    Ok(())
}

#[tauri::command]
pub fn edit_validator_test(
    test: ValidatorTest,
    state: State<ProblemManager>,
) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("tests/validator/{:02}", test.id));

    if path.exists() {
        test.save(&path)
    } else {
        Err(format!("Test with id {} does not exist", test.id))
    }
}

#[tauri::command]
pub fn delete_validator_test(id: u16, state: State<ProblemManager>) -> Result<(), String> {
    let path = state
        .get_current_path()?
        .join(format!("tests/validator/{:02}", id));

    fs::remove_file(path).err_to_string()
}
