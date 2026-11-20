use std::path::Path;

use std::io::Write;
use tempfile::NamedTempFile;

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

/// Writes `contents` to `path`, creating the parent directories first.
pub fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).fs_context(FsOperation::CreateDir, parent)?;
    }
    std::fs::write(path, contents).fs_context(FsOperation::Write, path)
}

/// Copies `from` to `to`, creating the parent directories of `to` first.
pub fn copy_file(from: &Path, to: &Path) -> AppResult<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).fs_context(FsOperation::CreateDir, parent)?;
    }
    std::fs::copy(from, to).fs_context(FsOperation::Copy, from)?;
    Ok(())
}

/// Names of the regular files directly inside `dir` (empty if it is missing).
pub fn file_names_in(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_file())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Windows line endings turned into `\n`.
pub fn to_lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn write_temp(content: &str) -> AppResult<NamedTempFile> {
    let mut f =
        tempfile::NamedTempFile::new().fs_context(FsOperation::Create, &std::env::temp_dir())?;
    f.write_all(content.as_bytes())
        .fs_context(FsOperation::Write, f.path())?;
    Ok(f)
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
