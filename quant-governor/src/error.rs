//! Error types for quant-governor.

use thiserror::Error;

/// Errors that can occur during governance evaluation.
#[derive(Debug, Error)]
pub enum GovernorError {
    /// Invalid request parameters
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Policy evaluation failed
    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),

    /// Invalid degradation threshold
    #[error("Invalid degradation threshold: {0}")]
    InvalidThreshold(String),

    /// Content type not supported
    #[error("Unsupported content type: {0}")]
    UnsupportedContentType(String),

    /// Selected codec is not admitted by the policy (no registered decoder).
    ///
    /// CMP-001: explicit reject state — the policy must never silently
    /// select a codec that has no admitted/registered decoder.
    #[error("codec '{profile}' is not admitted by policy '{policy}': {reason}")]
    UnsupportedCodec {
        /// The selected codec profile with no admitted decoder.
        profile: crate::CodecProfile,
        /// Name of the policy that refused the codec.
        policy: String,
        /// Human-readable rejection reason.
        reason: String,
    },

    /// Selected codec exceeds the request's latency budget.
    #[error(
        "codec '{profile}' estimated latency {estimated_ms}ms exceeds tolerance {requested_ms}ms"
    )]
    LatencyBudgetExceeded {
        /// The selected codec profile.
        profile: crate::CodecProfile,
        /// Latency tolerance from the request (ms).
        requested_ms: u64,
        /// Declared latency estimate for the profile (ms).
        estimated_ms: u64,
    },

    /// Requested content size exceeds the policy's byte budget.
    #[error("content size {requested_bytes} bytes exceeds byte budget {budget_bytes}")]
    ByteBudgetExceeded {
        /// Content size from the request (bytes).
        requested_bytes: u64,
        /// Configured policy byte budget (bytes).
        budget_bytes: u64,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl GovernorError {
    /// Returns true if this is a recoverable error.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            GovernorError::InvalidThreshold(_) | GovernorError::UnsupportedContentType(_)
        )
    }

    /// Returns true if this indicates a system configuration issue.
    pub fn is_configuration_error(&self) -> bool {
        matches!(
            self,
            GovernorError::InvalidThreshold(_) | GovernorError::Internal(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_recoverability() {
        let err = GovernorError::InvalidThreshold("too high".to_string());
        assert!(err.is_recoverable());

        let err = GovernorError::Internal("bug".to_string());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn error_display() {
        let err = GovernorError::InvalidRequest("bad value".to_string());
        assert_eq!(err.to_string(), "Invalid request: bad value");
    }
}
