use std::path::Path;

use crate::error::{AppError, AppResult, FsOperation};

/// Attaches filesystem context (what operation, on what path) to a raw
/// `std::io::Result`, turning it into an `AppResult` with an `AppError::Fs`
/// that keeps the original `io::Error` as its source.
pub trait FsResultExt<T> {
    fn fs_context(self, operation: FsOperation, path: &Path) -> AppResult<T>;
}

impl<T> FsResultExt<T> for std::io::Result<T> {
    fn fs_context(self, operation: FsOperation, path: &Path) -> AppResult<T> {
        self.map_err(|source| AppError::Fs {
            operation,
            path: path.to_owned(),
            source,
        })
    }
}

pub fn delete_file(path: &Path) -> AppResult<()> {
    std::fs::remove_file(path).fs_context(FsOperation::Delete, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_file_missing_path_reports_fs_context() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.txt");

        let err = delete_file(&path).unwrap_err();

        match err {
            AppError::Fs {
                operation,
                path: err_path,
                ..
            } => {
                assert_eq!(operation, FsOperation::Delete);
                assert_eq!(err_path, path);
            }
            other => panic!("expected AppError::Fs, got {other:?}"),
        }
    }

    #[test]
    fn delete_file_removes_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("present.txt");
        std::fs::write(&path, "content").unwrap();

        delete_file(&path).unwrap();

        assert!(!path.exists());
    }
}

