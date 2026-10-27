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
pub const TESTS_PATH: &str = "tests/main";
pub const SOLUTIONS_PATH: &str = "solutions";
pub const EXPORT_TESTS_PATH: &str = "tests/export";
pub const EXPORT_MANIFEST_FILENAME: &str = "manifest.json";

/// EXPORT
/// Author written to the MOJ package when the problem has none set.
pub const DEFAULT_AUTHOR: &str = "Ballooner";

/// COMMANDS
pub const PYTHON_INTERPRETER: &str = "python3";
pub const CPP_COMPILER: &str = "g++";
