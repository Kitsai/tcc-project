use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionDesc {
    file_name: String,
    tag: SolutionTag,
    author: Option<String>,
    change_time: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolutionTag {
    Main,
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    TimeLimitExceededOrCorrect,
    TimeLimitExceededOrMemoryLimitExceeded,
    MemoryLimitExceeded,
    PresentationError,
    #[serde(rename = "")]
    None,
}
