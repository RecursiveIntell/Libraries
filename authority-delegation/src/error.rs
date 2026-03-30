use thiserror::Error;

/// Validation failures for authority-delegation owned artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthorityValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
}

impl AuthorityValidationError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::InvalidState(..) => "invalid_state",
        }
    }
}

pub type AuthorityValidationResult = Result<(), AuthorityValidationError>;

pub(crate) fn require_non_empty(value: &str, field: &'static str) -> AuthorityValidationResult {
    if value.trim().is_empty() {
        return Err(AuthorityValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_non_empty_slice<T>(
    values: &[T],
    field: &'static str,
) -> AuthorityValidationResult {
    if values.is_empty() {
        return Err(AuthorityValidationError::MissingField(field));
    }
    Ok(())
}
