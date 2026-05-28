use thiserror::Error;

/// Errors returned by ComfyUI operations.
#[derive(Error, Debug)]
pub enum ComfyError {
    /// ComfyUI returned a non-success HTTP status.
    #[error("ComfyUI returned HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// The response from ComfyUI was missing expected fields.
    #[error("{0}")]
    InvalidResponse(String),

    /// The queued workflow had node-level errors.
    #[error("Workflow node errors: {0}")]
    NodeErrors(String),

    /// Timed out waiting for generation to complete.
    #[error("Generation timed out")]
    Timeout,

    /// ComfyUI reported an execution error during generation.
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// Download exceeded the configured limit.
    #[error("Download for '{filename}' exceeded {limit_bytes} bytes (received {actual_bytes})")]
    OutputTooLarge {
        filename: String,
        limit_bytes: usize,
        actual_bytes: usize,
    },

    /// Network-level request failure with context.
    #[error("{context}: {source}")]
    Network {
        context: String,
        source: reqwest::Error,
    },

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl ComfyError {
    /// Returns a stable string discriminant for programmatic matching.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Http { .. } => "http",
            Self::InvalidResponse(_) => "invalid_response",
            Self::NodeErrors(_) => "node_errors",
            Self::Timeout => "timeout",
            Self::GenerationFailed(_) => "generation_failed",
            Self::OutputTooLarge { .. } => "output_too_large",
            Self::Network { .. } => "network",
            Self::Json(_) => "json",
        }
    }
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, ComfyError>;
