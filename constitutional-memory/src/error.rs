use thiserror::Error;

/// Validation failures for constitutional-memory owned artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConstitutionalValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
}

impl ConstitutionalValidationError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::InvalidState(..) => "invalid_state",
        }
    }
}

pub type ConstitutionalValidationResult = Result<(), ConstitutionalValidationError>;
