//! Error types for receipt-bench.

use thiserror::Error;

/// Errors that can occur during benchmark operations.
#[derive(Debug, Error)]
pub enum BenchError {
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
}

impl From<String> for BenchError {
    fn from(s: String) -> Self {
        BenchError::Execution(s)
    }
}