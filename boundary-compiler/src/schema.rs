//! Optional JSON Schema validation before and after JCS canonicalization.
//!
//! NOTE: `boundary-compiler` does NOT implement JSON Schema validation. This
//! module historically exposed a `SchemaValidator` whose `validate` returned
//! `Ok(())` unconditionally (a stub), while these docs referenced a
//! `jsonschema` feature that this crate does not define. A silently-passing
//! validator is a false assurance at an admission boundary, so `validate` now
//! FAILS CLOSED: it returns a typed error rather than implying that input was
//! checked.
//!
//! If a boundary profile needs schema conformance, validate with a dedicated
//! JSON Schema implementation at the call site, keeping syntax parsing, schema
//! checking, canonical byte encoding and hash/signature verification separate.

use crate::error::JcsError;

/// Companion type retained for API compatibility. It performs NO validation.
///
/// `validate` fails closed (returns an error) so a caller can never mistake an
/// unchecked value for a schema-validated one, and no caller can rely on a
/// feature that this crate does not provide.
#[derive(Debug, Clone, Default)]
pub struct SchemaValidator;

impl SchemaValidator {
    /// Creates a new validator. No validation is performed by this crate; see
    /// the module docs.
    pub fn new() -> Self {
        Self
    }

    /// Always fails closed.
    ///
    /// `boundary-compiler` provides no JSON Schema validation, so this returns
    /// an error rather than a false `Ok(())`. Callers that need schema
    /// conformance must validate with a dedicated implementation before
    /// canonicalizing.
    pub fn validate(&self, _value: &serde_json::Value) -> Result<(), JcsError> {
        Err(JcsError::SchemaError(
            "JSON Schema validation is not implemented by boundary-compiler; \
             validate with a dedicated JSON Schema implementation at the call site \
             (no `jsonschema` feature exists in this crate)"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_fails_closed_and_never_silently_passes() {
        let v = SchemaValidator::new();
        let err = v
            .validate(&serde_json::json!({"any": "thing"}))
            .expect_err("validate must fail closed, never return a false Ok(())");
        assert!(matches!(err, JcsError::SchemaError(_)));
    }
}
