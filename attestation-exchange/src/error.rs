use thiserror::Error;

/// Validation failures for attestation-exchange owned artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AttestationValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
}

impl AttestationValidationError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::InvalidState(..) => "invalid_state",
        }
    }
}

pub type AttestationValidationResult = Result<(), AttestationValidationError>;

pub(crate) fn require_non_empty(value: &str, field: &'static str) -> AttestationValidationResult {
    if value.trim().is_empty() {
        return Err(AttestationValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_non_empty_slice<T>(values: &[T], field: &'static str) -> AttestationValidationResult {
    if values.is_empty() {
        return Err(AttestationValidationError::MissingField(field));
    }
    Ok(())
}
