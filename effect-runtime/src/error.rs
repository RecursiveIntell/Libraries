//! Validation error types and helper functions for effect-runtime artifact invariant checks.

use thiserror::Error;

/// Validation failures for effect-runtime owned constitutional artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EffectRuntimeValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid schema version for {artifact}: expected {expected}, found {found}")]
    InvalidSchemaVersion {
        artifact: &'static str,
        expected: &'static str,
        found: String,
    },
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
    #[error("invalid timestamp in field {field}: {value}")]
    InvalidTimestamp { field: &'static str, value: String },
}

impl EffectRuntimeValidationError {
    /// Returns a stable string discriminant for this validation error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingField(..) => "missing_field",
            Self::InvalidSchemaVersion { .. } => "invalid_schema_version",
            Self::InvalidState(..) => "invalid_state",
            Self::InvalidTimestamp { .. } => "invalid_timestamp",
        }
    }
}

/// Shorthand result type for artifact validation methods.
pub type EffectValidationResult = Result<(), EffectRuntimeValidationError>;

pub(crate) fn require_non_empty(value: &str, field: &'static str) -> EffectValidationResult {
    if value.trim().is_empty() {
        return Err(EffectRuntimeValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_id(value: &impl AsRef<str>, field: &'static str) -> EffectValidationResult {
    require_non_empty(value.as_ref(), field)
}

pub(crate) fn require_non_empty_slice<T>(values: &[T], field: &'static str) -> EffectValidationResult {
    if values.is_empty() {
        return Err(EffectRuntimeValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_valid_timestamp(value: &str, field: &'static str) -> EffectValidationResult {
    if value.trim().is_empty() {
        return Err(EffectRuntimeValidationError::MissingField(field));
    }
    chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
        EffectRuntimeValidationError::InvalidTimestamp {
            field,
            value: value.to_string(),
        }
    })?;
    Ok(())
}

pub(crate) fn require_schema_version(
    artifact: &'static str,
    expected: &'static str,
    found: &str,
) -> EffectValidationResult {
    if found != expected {
        return Err(EffectRuntimeValidationError::InvalidSchemaVersion {
            artifact,
            expected,
            found: found.to_string(),
        });
    }
    Ok(())
}
