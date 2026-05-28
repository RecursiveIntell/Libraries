//! Optional JSON Schema validation before and after JCS canonicalization.
//!
//! This module provides a `SchemaValidator` that can validate JSON values
//! against JSON Schema before and/or after canonicalization. It's designed
//! for boundary profiles that require schema conformance as part of
//! their transformation pipeline.
//!
//! Note: Schema validation is a stub in the base crate. For full JSON Schema
//! support, enable the `jsonschema` feature.

use crate::error::JcsError;

/// JSON Schema validator companion to BoundaryProfile.
///
/// Currently a stub that always passes validation.
/// For full JSON Schema support, use a dedicated validator crate.
#[derive(Debug, Clone, Default)]
pub struct SchemaValidator;

impl SchemaValidator {
    /// Creates a new validator (stub).
    pub fn new() -> Self {
        Self
    }

    /// Validates a value (stub — always returns Ok).
    pub fn validate(&self, _value: &serde_json::Value) -> Result<(), JcsError> {
        Ok(())
    }
}
