use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("command failed: {0}")]
    Command(String),
    #[error("invalid input: {0}")]
    Invalid(String),
}
pub type Result<T> = std::result::Result<T, Error>;
