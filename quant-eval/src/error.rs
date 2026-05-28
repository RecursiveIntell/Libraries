//! Error types for quant-eval.

use thiserror::Error;

/// Errors that can occur during evaluation operations.
#[derive(Debug, Error)]
pub enum QuantEvalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Benchmark execution failed: {0}")]
    Execution(String),

    #[error("Git operation failed: {0}")]
    Git(String),

    #[error("No git repository found")]
    NoGitRepo,

    #[error("Invalid corpus configuration: {0}")]
    InvalidCorpus(String),

    #[error("Codec error: {0}")]
    Codec(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
}

impl From<String> for QuantEvalError {
    fn from(s: String) -> Self {
        QuantEvalError::Execution(s)
    }
}
