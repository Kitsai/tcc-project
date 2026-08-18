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
