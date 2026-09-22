use std::path::PathBuf;

use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to {operation} {path:?}: {source}")]
    Fs {
        operation: FsOperation,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{0}")]
    Default(String),
}

/// The filesystem operation an `AppError::Fs` failed during, so the message
/// (and, later, any retry/atomic-write logic) can tell "couldn't read" apart
/// from "couldn't write" apart from "couldn't delete".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsOperation {
    Read,
    Write,
    Create,
    CreateDir,
    Delete,
    RemoveDir,
    Rename,
    Copy,
}

impl std::fmt::Display for FsOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Create => "create",
            Self::CreateDir => "create directory",
            Self::Delete => "delete",
            Self::RemoveDir => "remove directory",
            Self::Rename => "rename",
            Self::Copy => "copy",
        };
        write!(f, "{s}")
    }
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
    fn fs_error_display_includes_operation_path_and_source() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file or directory");
        let err = AppError::Fs {
            operation: FsOperation::Delete,
            path: PathBuf::from("/tmp/foo.txt"),
            source,
        };

        let message = err.to_string();
        assert!(message.contains("Failed to delete"));
        assert!(message.contains("/tmp/foo.txt"));
        assert!(message.contains("no such file or directory"));
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
