use serde::Serialize;

pub type TauriResult<T> = Result<T, impl Serialize>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {}
