pub const NO_PRBLM_ERR: &str = "No problem is open";
pub const MULT_SEPARATOR: &str = "===";

/// OS
#[cfg(target_os = "windows")]
pub const BINARY_EXTENSION: &str = "exe";

#[cfg(not(target_os = "windows"))]
pub const BINARY_EXTENSION: &str = "";

/// PATHS
pub const BINARY_DIR: &str = "bin";
pub const VALIDATOR_TESTS_PATH: &str = "tests/validator";
pub const CHECKER_TESTS_PATH: &str = "tests/checker";

/// COMMANDS
pub const PYTHON_INTERPRETER: &str = "python3";
pub const CPP_COMPILER: &str = "g++";
