use std::path::PathBuf;

use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to delete file {0}")]
    FailedToDelete(PathBuf),
    #[error("{0}")]
    Default(String),
}

/// Serializes as its Display string rather than the default externally-tagged
/// enum representation (`{"Default": "msg"}`), so commands returning
/// `AppResult` surface a plain error string to the frontend instead of an
/// object that stringifies to `[object Object]`.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Default(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        AppError::Default(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_display_is_the_inner_message() {
        let err = AppError::Default("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn failed_to_delete_display_includes_path() {
        let err = AppError::FailedToDelete(PathBuf::from("/tmp/foo.txt"));
        assert_eq!(err.to_string(), "Failed to delete file /tmp/foo.txt");
    }

    #[test]
    fn serializes_as_a_bare_string_not_an_object() {
        let err = AppError::Default("boom".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"boom\"");
    }

    #[test]
    fn from_string_wraps_as_default() {
        let err: AppError = "oops".to_string().into();
        assert!(matches!(err, AppError::Default(msg) if msg == "oops"));
    }

    #[test]
    fn from_str_wraps_as_default() {
        let err: AppError = "oops".into();
        assert!(matches!(err, AppError::Default(msg) if msg == "oops"));
    }
}
