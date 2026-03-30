use crate::error::{ContinuityValidationError, ContinuityValidationResult};

pub(crate) fn require_non_empty(value: &str, field: &'static str) -> ContinuityValidationResult {
    if value.trim().is_empty() {
        return Err(ContinuityValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_non_empty_slice<T>(values: &[T], field: &'static str) -> ContinuityValidationResult {
    if values.is_empty() {
        return Err(ContinuityValidationError::MissingField(field));
    }
    Ok(())
}

pub(crate) fn require_schema_version(found: &str, expected: &'static str) -> ContinuityValidationResult {
    if found != expected {
        return Err(ContinuityValidationError::SchemaVersionMismatch {
            expected,
            found: found.to_string(),
        });
    }
    Ok(())
}
