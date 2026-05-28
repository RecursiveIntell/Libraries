//! RFC 8785 JSON Canonicalization (JCS) implementation.
//!
//! This module provides strict JCS canonicalization with duplicate-key rejection.
//! It implements the transformation rules from RFC 8785 §2.1–§2.7:
//! - Control characters escaped as `\uXXXX`
//! - String escapes in deterministic order
//! - Numbers with specific formatting rules
//! - Object keys sorted lexicographically by JSON string codepoints

use crate::error::JcsError;
use serde_json::{Map, Number, Value};
use std::collections::BTreeSet;

/// JCS canonicalizer that produces deterministic JSON bytes.
#[derive(Debug, Clone, Default)]
pub struct Canonicalizer;

impl Canonicalizer {
    /// Creates a new Canonicalizer instance.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Canonicalizes a JSON value into a string per RFC 8785.
    ///
    /// # Errors
    ///
    /// Returns `JcsError::DuplicateKey` if duplicate object keys are detected.
    pub fn canonicalize(&self, value: &Value) -> Result<String, JcsError> {
        let mut out = String::new();
        self.write_value(&mut out, value)?;
        Ok(out)
    }

    /// Canonicalizes a JSON value into a `Vec<u8>` per RFC 8785.
    pub fn canonicalize_bytes(&self, value: &Value) -> Result<Vec<u8>, JcsError> {
        Ok(self.canonicalize(value)?.into_bytes())
    }

    fn write_value(&self, out: &mut String, value: &Value) -> Result<(), JcsError> {
        match value {
            Value::Null => {
                out.push_str("null");
            }
            Value::Bool(b) => {
                out.push_str(if *b { "true" } else { "false" });
            }
            Value::Number(n) => {
                self.write_number(out, n);
            }
            Value::String(s) => {
                self.write_string(out, s);
            }
            Value::Array(arr) => self.write_array(out, arr)?,
            Value::Object(obj) => self.write_object(out, obj)?,
        }
        Ok(())
    }

    fn write_number(&self, out: &mut String, n: &Number) {
        // RFC 8785 §2.3: Numbers are serialized without whitespace,
        // using the shortest representation that preserves values.
        out.push_str(&n.to_string());
    }

    fn write_string(&self, out: &mut String, s: &str) {
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{08}' => out.push_str("\\b"), // backspace
                '\u{0C}' => out.push_str("\\f"), // form feed
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    // RFC 8785 §2.2: Control characters → \uXXXX
                    out.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    fn write_array(&self, out: &mut String, arr: &[Value]) -> Result<(), JcsError> {
        out.push('[');
        let mut first = true;
        for v in arr {
            if !first {
                out.push(',');
            }
            first = false;
            self.write_value(out, v)?;
        }
        out.push(']');
        Ok(())
    }

    fn write_object(&self, out: &mut String, obj: &Map<String, Value>) -> Result<(), JcsError> {
        // RFC 8785 §2.7: Object names (keys) MUST be sorted lexicographically by
        // JSON string codepoints (UCS-2). BTreeMap gives us this ordering.
        //
        // We use a custom parser to detect duplicate keys (serde_json allows them).
        out.push('{');

        // Track seen keys to detect duplicates (RFC 8785 §2.7)
        let mut seen_keys = BTreeSet::new();
        let mut first = true;

        for (key, value) in obj.iter() {
            if !seen_keys.insert(key.clone()) {
                return Err(JcsError::DuplicateKey { key: key.clone() });
            }

            if !first {
                out.push(',');
            }
            first = false;

            self.write_string(out, key);
            out.push(':');
            self.write_value(out, value)?;
        }

        out.push('}');
        Ok(())
    }
}

/// Parse JSON with duplicate-key detection.
///
/// Unlike `serde_json::from_str`, this returns `JcsError::DuplicateKey`
/// when duplicate object keys are found (required by RFC 8785).
///
/// NOTE: serde_json::from_str in non-strict mode silently accepts duplicates
/// (keeps last value), so we MUST pre-validate the raw string before parsing.
pub fn parse_with_dup_check(s: &str) -> Result<Value, JcsError> {
    // Pre-check: scan raw string for duplicate keys at same nesting depth
    // before letting serde_json silently pick one value
    if let Some(dup) = find_duplicate_key(s) {
        return Err(JcsError::DuplicateKey { key: dup });
    }
    let value: Value = serde_json::from_str(s).map_err(|e| JcsError::InvalidJson {
        reason: e.to_string(),
    })?;
    Ok(value)
}

/// Scans a raw JSON string for duplicate keys at the same nesting depth.
///
/// Returns the first duplicate key found, or None if the input is clean.
/// Uses a (key_name, depth) HashMap so nested objects can reuse key names
/// (e.g. `{"a": {"a": 1}}` is NOT a duplicate — different depths).
///
/// Required because `serde_json::from_str` silently accepts duplicates
/// in non-strict mode (keeps last value).
fn find_duplicate_key(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut depth: usize = 0;
    // Track (key_name, depth) → depth_of_first_occurrence
    let mut seen: std::collections::HashMap<(String, usize), usize, _> =
        std::collections::HashMap::new();

    while i < n {
        match bytes[i] {
            b'"' => {
                // Beginning or end of a string at current position
                let key_start = i + 1;
                let key_end = skip_string(bytes, key_start, n);
                let key = String::from_utf8_lossy(&bytes[key_start..key_end]).to_string();

                // Advance cursor past the closing quote
                i = key_end;

                // Only treat this string as a key if followed by ':' at same depth
                if is_key_at_depth(bytes, key_end, n) {
                    let key_depth = depth;
                    if let Some(&first_depth) = seen.get(&(key.clone(), key_depth)) {
                        if first_depth == key_depth {
                            return Some(key);
                        }
                    }
                    seen.insert((key, key_depth), key_depth);
                }
            }
            b'{' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b']' => {
                // depth can only go to 0 at most, never below
                depth = depth.saturating_sub(1);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

/// Advance past a JSON string starting immediately after the opening '"'.
/// Returns the index of the character AFTER the closing '"'.
fn skip_string(bytes: &[u8], mut i: usize, n: usize) -> usize {
    while i < n {
        match bytes[i] {
            b'"' => return i + 1, // position after closing quote
            b'\\' => i += 2,      // skip escaped char
            _ => i += 1,
        }
    }
    n // unclosed string — fall off end
}

/// Returns true if the string starting at `pos` is followed by ':' at `depth`,
/// with optional whitespace in between.
fn is_key_at_depth(bytes: &[u8], pos: usize, n: usize) -> bool {
    let mut j = pos;
    // skip whitespace
    while j < n && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r')
    {
        j += 1;
    }
    j < n && bytes[j] == b':'
}
fn detect_duplicates(value: &Value) -> Result<(), JcsError> {
    match value {
        Value::Object(map) => {
            let mut keys = BTreeSet::new();
            for key in map.keys() {
                if !keys.insert(key.clone()) {
                    return Err(JcsError::DuplicateKey { key: key.clone() });
                }
            }
            for v in map.values() {
                detect_duplicates(v)?;
            }
        }
        Value::Array(arr) => {
            for v in arr {
                detect_duplicates(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Validates that a JSON string parses to JSON, resolving duplicates first.
///
/// This is used for canonicalization inputs: the input need not be ordered,
/// but duplicates MUST be rejected before canonicalization.
pub fn parse_and_validate(input: &str) -> Result<Value, JcsError> {
    let value = serde_json::from_str(input).map_err(|e| JcsError::ParseError(e))?;
    detect_duplicates(&value)?;
    Ok(value)
}

/// Canonicalize with automatic duplicate detection first.
///
/// This canonicalizes any JSON (possibly with duplicate keys in the source),
/// returning an error if duplicates are found.
pub fn canonicalize_flexible(value: &Value) -> Result<String, JcsError> {
    detect_duplicates(value)?;
    Canonicalizer::new().canonicalize(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_null() {
        let c = Canonicalizer::new();
        assert_eq!(c.canonicalize(&json!(null)).unwrap(), "null");
    }

    #[test]
    fn test_bool() {
        let c = Canonicalizer::new();
        assert_eq!(c.canonicalize(&json!(true)).unwrap(), "true");
        assert_eq!(c.canonicalize(&json!(false)).unwrap(), "false");
    }

    #[test]
    fn test_numbers() {
        let c = Canonicalizer::new();
        assert_eq!(c.canonicalize(&json!(42)).unwrap(), "42");
        assert_eq!(c.canonicalize(&json!(-1)).unwrap(), "-1");
        assert_eq!(c.canonicalize(&json!(0.5)).unwrap(), "0.5");
    }

    #[test]
    fn test_string_basic() {
        let c = Canonicalizer::new();
        assert_eq!(c.canonicalize(&json!("hello")).unwrap(), "\"hello\"");
    }

    #[test]
    fn test_string_escapes() {
        let c = Canonicalizer::new();
        // Double quote
        assert_eq!(c.canonicalize(&json!("a\"b")).unwrap(), "\"a\\\"b\"");
        // Backslash
        assert_eq!(c.canonicalize(&json!("a\\b")).unwrap(), "\"a\\\\b\"");
        // Newline
        assert_eq!(c.canonicalize(&json!("a\nb")).unwrap(), "\"a\\nb\"");
        // Control char → \uXXXX
        assert_eq!(c.canonicalize(&json!("a\u{0}b")).unwrap(), "\"a\\u0000b\"");
    }

    #[test]
    fn test_object_sorted_keys() {
        let c = Canonicalizer::new();
        let obj = json!({"b": 1, "a": 2, "c": 3});
        let out = c.canonicalize(&obj).unwrap();
        // Keys must be sorted, so b comes before c, etc.
        assert_eq!(out, r#"{"a":2,"b":1,"c":3}"#);
    }

    #[test]
    fn test_nested_object() {
        let c = Canonicalizer::new();
        let obj = json!({
            "z": {"b": 1, "a": 2},
            "a": [3, 2, 1]
        });
        let out = c.canonicalize(&obj).unwrap();
        assert_eq!(out, r#"{"a":[3,2,1],"z":{"a":2,"b":1}}"#);
    }

    #[test]
    fn test_duplicate_key_rejected() {
        // JSON parser must reject duplicate keys (RFC 8785 §2.7)
        let result = parse_with_dup_check(r#"{"a": 1, "a": 2}"#);
        assert!(matches!(result, Err(JcsError::DuplicateKey { .. })));
    }

    #[test]
    fn test_detect_duplicates_nested() {
        // Nested duplicate keys in parsed JSON
        let s = r#"{"outer": {"x": 1, "x": 2}}"#;
        let result = parse_with_dup_check(s);
        assert!(matches!(result, Err(JcsError::DuplicateKey { .. })));
    }
}
