use std::path::Path;

use crate::error::{AppError, AppResult};

pub fn delete_file(path: &Path) -> AppResult<()> {
    std::fs::remove_file(path).map_err(|_| AppError::FailedToDelete(path.to_owned()))
}
