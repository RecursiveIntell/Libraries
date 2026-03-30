use thiserror::Error;

/// Validation failures for continuity-runtime owned artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContinuityValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid schema version: expected {expected}, found {found}")]
    SchemaVersionMismatch {
        expected: &'static str,
        found: String,
    },
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
}

impl ContinuityValidationError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::SchemaVersionMismatch { .. } => "schema_version_mismatch",
            Self::InvalidState(..) => "invalid_state",
        }
    }
}

pub type ContinuityValidationResult = Result<(), ContinuityValidationError>;
