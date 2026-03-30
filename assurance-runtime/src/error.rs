use thiserror::Error;

/// Validation failures for assurance-runtime owned artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssuranceValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
}

impl AssuranceValidationError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::InvalidState(..) => "invalid_state",
        }
    }
}

pub type AssuranceValidationResult = Result<(), AssuranceValidationError>;

pub(crate) fn require_non_empty(value: &str, field: &'static str) -> AssuranceValidationResult {
    if value.trim().is_empty() {
        return Err(AssuranceValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_non_empty_slice<T>(
    values: &[T],
    field: &'static str,
) -> AssuranceValidationResult {
    if values.is_empty() {
        return Err(AssuranceValidationError::MissingField(field));
    }
    Ok(())
}
