//! Boundary profile definitions.
//!
//! A BoundaryProfile defines the constraints and policies under which JCS operates:
//! - **Dialect**: The JSON dialect (canonical, compact, etc.)
//! - **SchemaId & SchemaVersion**: Schema identification for validation
//! - **CanonicalizationProfile**: What transformations are applied
//! - **UnknownFieldPolicy**: How to handle unknown fields during schema validation
//! - **ResourceCeilings**: Limits on object count, string length, array length, depth

use crate::error::JcsError;
use serde::{Deserialize, Serialize};

/// JSON dialect variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dialect {
    /// Canonical JSON (RFC 8785 JCS).
    Canonical,
    /// Compact form (minified, no extra whitespace).
    Compact,
    /// Pretty-printed for human readability.
    Pretty,
}

impl Default for Dialect {
    fn default() -> Self {
        Dialect::Canonical
    }
}

/// Canonicalization profile — what transformations are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalizationProfile {
    /// Strict RFC 8785 (only Unicode escapes, no sorting hints).
    Strict,
    /// RFC 8785 + normalize field ordering hints.
    Normalized,
    /// Application-specific profile.
    Custom,
}

impl Default for CanonicalizationProfile {
    fn default() -> Self {
        CanonicalizationProfile::Strict
    }
}

/// Policy for unknown fields encountered during schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownFieldPolicy {
    /// Reject with error.
    Reject,
    /// Strip unknown fields silently.
    Strip,
    /// Pass unknown fields through unchanged.
    PassThrough,
}

impl Default for UnknownFieldPolicy {
    fn default() -> Self {
        UnknownFieldPolicy::Reject
    }
}

/// Resource ceiling limits for a boundary profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResourceCeilings {
    /// Maximum number of top-level keys (default: 128).
    pub max_object_keys: usize,
    /// Maximum total string length in bytes (default: 1MB).
    pub max_string_bytes: usize,
    /// Maximum array length (default: 1024).
    pub max_array_len: usize,
    /// Maximum nesting depth (default: 32).
    pub max_depth: usize,
    /// Maximum number of semantically-relevant float digits (default: 17 for f64 precision).
    pub max_float_digits: usize,
}

impl Default for ResourceCeilings {
    fn default() -> Self {
        Self {
            max_object_keys: 128,
            max_string_bytes: 1 << 20, // 1 MiB
            max_array_len: 1024,
            max_depth: 32,
            max_float_digits: 17,
        }
    }
}

/// A boundary profile with all constraints and policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BoundaryProfile {
    /// JSON dialect for this profile.
    pub dialect: Dialect,
    /// Schema identifier (e.g., `"https://example.com/schema"`).
    pub schema_id: Option<String>,
    /// Schema version string (e.g., `"1.2.0"`).
    pub schema_version: Option<String>,
    /// Canonicalization profile.
    pub canonicalization: CanonicalizationProfile,
    /// Policy for unknown fields during schema validation.
    pub unknown_field_policy: UnknownFieldPolicy,
    /// Resource ceiling limits.
    pub resource_ceilings: ResourceCeilings,
}

impl Default for BoundaryProfile {
    fn default() -> Self {
        Self {
            dialect: Dialect::Canonical,
            schema_id: None,
            schema_version: None,
            canonicalization: CanonicalizationProfile::Strict,
            unknown_field_policy: UnknownFieldPolicy::Reject,
            resource_ceilings: ResourceCeilings::default(),
        }
    }
}

impl BoundaryProfile {
    /// Creates a default boundary profile with RFC 8785 canonicalization.
    pub fn rfc8785() -> Self {
        Self::default()
    }

    /// Creates a boundary profile with the given schema ID and version.
    pub fn with_schema(mut self, id: impl Into<String>, version: impl Into<String>) -> Self {
        self.schema_id = Some(id.into());
        self.schema_version = Some(version.into());
        self
    }

    /// Validates a parsed JSON value against resource ceilings.
    pub fn check_resources(&self, value: &serde_json::Value) -> Result<(), JcsError> {
        self.check_resources_inner(value, 0)
    }

    fn check_resources_inner(
        &self,
        value: &serde_json::Value,
        depth: usize,
    ) -> Result<(), JcsError> {
        use crate::error::JcsError::ResourceCeilingExceeded;

        if depth > self.resource_ceilings.max_depth {
            return Err(ResourceCeilingExceeded {
                resource: "depth".to_string(),
                used: depth,
                limit: self.resource_ceilings.max_depth,
            });
        }

        match value {
            serde_json::Value::Object(map) => {
                if map.len() > self.resource_ceilings.max_object_keys {
                    return Err(ResourceCeilingExceeded {
                        resource: "object_keys".to_string(),
                        used: map.len(),
                        limit: self.resource_ceilings.max_object_keys,
                    });
                }
                for (k, v) in map.iter() {
                    if k.len() > self.resource_ceilings.max_string_bytes {
                        return Err(ResourceCeilingExceeded {
                            resource: "string_bytes".to_string(),
                            used: k.len(),
                            limit: self.resource_ceilings.max_string_bytes,
                        });
                    }
                    self.check_resources_inner(v, depth + 1)?;
                }
            }
            serde_json::Value::Array(arr) => {
                if arr.len() > self.resource_ceilings.max_array_len {
                    return Err(ResourceCeilingExceeded {
                        resource: "array_len".to_string(),
                        used: arr.len(),
                        limit: self.resource_ceilings.max_array_len,
                    });
                }
                for v in arr.iter() {
                    self.check_resources_inner(v, depth + 1)?;
                }
            }
            serde_json::Value::String(s) => {
                if s.len() > self.resource_ceilings.max_string_bytes {
                    return Err(ResourceCeilingExceeded {
                        resource: "string_bytes".to_string(),
                        used: s.len(),
                        limit: self.resource_ceilings.max_string_bytes,
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Human-readable identifier for this profile.
    pub fn identifier(&self) -> String {
        match (&self.schema_id, &self.schema_version) {
            (Some(id), Some(ver)) => format!("{id}:{ver}"),
            (Some(id), None) => id.clone(),
            _ => "default".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_profile() {
        let p = BoundaryProfile::default();
        assert_eq!(p.dialect, Dialect::Canonical);
        assert!(p.schema_id.is_none());
        assert_eq!(p.canonicalization, CanonicalizationProfile::Strict);
        assert_eq!(p.unknown_field_policy, UnknownFieldPolicy::Reject);
    }

    #[test]
    fn test_profile_with_schema() {
        let p = BoundaryProfile::default().with_schema("https://example.com/s", "1.0.0");
        assert_eq!(p.schema_id.as_deref(), Some("https://example.com/s"));
        assert_eq!(p.schema_version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_check_resources_ok() {
        let p = BoundaryProfile::default();
        let val = json!({"a": "hello", "b": [1, 2, 3]});
        p.check_resources(&val).unwrap();
    }

    #[test]
    fn test_check_resources_depth_exceeded() {
        let mut p = BoundaryProfile::default();
        p.resource_ceilings.max_depth = 2;

        // Depth 3 exceeds limit of 2
        let val = json!({"a": {"b": {"c": 1}}});
        let result = p.check_resources(&val);
        assert!(matches!(
            result,
            Err(JcsError::ResourceCeilingExceeded { .. })
        ));
    }

    #[test]
    fn test_check_resources_object_keys_exceeded() {
        let mut p = BoundaryProfile::default();
        p.resource_ceilings.max_object_keys = 2;

        let val = json!({"a": 1, "b": 2, "c": 3});
        let result = p.check_resources(&val);
        assert!(matches!(
            result,
            Err(JcsError::ResourceCeilingExceeded { .. })
        ));
    }

    #[test]
    fn test_check_resources_string_bytes_exceeded() {
        let mut p = BoundaryProfile::default();
        p.resource_ceilings.max_string_bytes = 5;

        let val = json!({"toolong": "x"});
        let result = p.check_resources(&val);
        assert!(matches!(
            result,
            Err(JcsError::ResourceCeilingExceeded { .. })
        ));
    }
}
