//! Compatibility projection for the historical `boundary_compiler::digest` path.
//!
//! The canonical digest owner is now `stack_ids::ContentDigest`. This module
//! preserves the former value-oriented API for existing consumers while
//! canonicalizing through this crate and delegating the final BLAKE3 hash to
//! `stack_ids`.

use crate::canonicalizer::Canonicalizer;
use crate::error::JcsError;
use serde_json::Value;

/// A BLAKE3 digest over RFC 8785 canonical JSON bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDigest(stack_ids::ContentDigest);

impl ContentDigest {
    /// Compute the digest over the canonical bytes of a JSON value.
    pub fn compute(value: &Value) -> Result<Self, JcsError> {
        let bytes = Canonicalizer::new().canonicalize_bytes(value)?;
        Ok(Self(stack_ids::ContentDigest::compute(&bytes)))
    }

    /// Return the 64-character lowercase hexadecimal digest.
    pub fn hex(&self) -> String {
        self.0.hex().to_owned()
    }

    /// Return the raw 32-byte BLAKE3 digest.
    pub fn as_bytes(&self) -> [u8; 32] {
        let hex = self.0.hex().as_bytes();
        let mut bytes = [0_u8; 32];
        for (index, output) in bytes.iter_mut().enumerate() {
            let high = hex_nibble(hex[index * 2]);
            let low = hex_nibble(hex[index * 2 + 1]);
            *output = (high << 4) | low;
        }
        bytes
    }
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}

impl std::fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.hex())
    }
}

impl std::fmt::LowerHex for ContentDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::ContentDigest;
    use serde_json::json;

    #[test]
    fn historical_value_api_matches_root_digest_bytes() {
        let value = json!({"b": 1, "a": 2});
        let digest = match ContentDigest::compute(&value) {
            Ok(digest) => digest,
            Err(error) => panic!("valid JSON canonicalization failed: {error}"),
        };
        assert_eq!(digest.as_bytes().len(), 32);
        assert_eq!(format!("{digest}"), format!("{digest:x}"));
        assert_eq!(digest.hex().len(), 64);
    }
}
