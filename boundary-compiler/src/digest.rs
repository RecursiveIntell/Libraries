//! blake3 ContentDigest over JCS bytes.
//!
//! RFC 8785 does not define a digest algorithm; this module computes a blake3
//! Content-Digest over the canonical JSON bytes, following the IETF Content-Digest
//! spec (RFC 9562, draft-ietf-httpbis-digest-headers).

use crate::error::JcsError;
use blake3::Hash;

/// A blake3 content digest of JCS bytes, in hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDigest(pub Hash);

impl ContentDigest {
    /// Computes the blake3 content digest over the JCS bytes of a JSON value.
    pub fn compute(value: &serde_json::Value) -> Result<Self, JcsError> {
        use crate::canonicalizer::Canonicalizer;
        let canonical_bytes = Canonicalizer::new().canonicalize_bytes(value)?;
        Ok(Self(blake3::hash(&canonical_bytes)))
    }

    /// Returns the hex-encoded digest.
    pub fn hex(&self) -> String {
        self.0.to_hex().to_string()
    }

    /// Returns the raw digest bytes.
    pub fn as_bytes(&self) -> [u8; 32] {
        *self.0.as_bytes()
    }
}

impl std::fmt::Display for ContentDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex())
    }
}

impl std::fmt::LowerHex for ContentDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_digest_same_input_same_output() {
        let val = json!({"b": 1, "a": 2});
        let d1 = ContentDigest::compute(&val).unwrap();
        let d2 = ContentDigest::compute(&val).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(d1.hex(), d2.hex());
    }

    #[test]
    fn test_digest_different_inputs_different_output() {
        let val1 = json!({"a": 1});
        let val2 = json!({"a": 2});
        let d1 = ContentDigest::compute(&val1).unwrap();
        let d2 = ContentDigest::compute(&val2).unwrap();
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_digest_is_deterministic() {
        let val = json!({"z": {"b": [1, 2], "a": null}, "a": true});
        let d1 = ContentDigest::compute(&val).unwrap();
        // Run multiple times
        for _ in 0..10 {
            let d2 = ContentDigest::compute(&val).unwrap();
            assert_eq!(d1, d2);
        }
    }

    #[test]
    fn test_digest_display() {
        let val = json!({"x": 1});
        let d = ContentDigest::compute(&val).unwrap();
        let display = format!("{}", d);
        let hex = format!("{:x}", d);
        assert_eq!(display, hex);
    }
}
