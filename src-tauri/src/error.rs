use std::path::PathBuf;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("Failed to delete file {0}")]
    FailedToDelete(PathBuf),
    #[error("{0}")]
    Default(String),
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
