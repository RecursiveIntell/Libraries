//! RFC 8785 JSON Canonicalization (JCS) implementation.
//!
//! This module provides strict JCS canonicalization with duplicate-key rejection.
//! It implements the transformation rules from RFC 8785 §2.1–§2.7:
//! - U+0000 through U+001F escaped per ECMAScript `JSON.stringify`
//! - String escapes in deterministic order
//! - Numbers with specific formatting rules
//! - Object keys sorted lexicographically by UTF-16 code units

use crate::error::JcsError;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

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
                self.write_number(out, n)?;
            }
            Value::String(s) => {
                self.write_string(out, s);
            }
            Value::Array(arr) => self.write_array(out, arr)?,
            Value::Object(obj) => self.write_object(out, obj)?,
        }
        Ok(())
    }

    fn write_number(&self, out: &mut String, n: &Number) -> Result<(), JcsError> {
        let value = n.as_f64().ok_or_else(|| JcsError::InvalidJson {
            reason: "number is outside the IEEE-754 binary64 JCS domain".into(),
        })?;
        out.push_str(ryu_js::Buffer::new().format_finite(value));
        Ok(())
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
                c if c <= '\u{1f}' => {
                    // RFC 8785 §3.2.2.2: ASCII controls use lowercase Unicode escapes.
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
        // RFC 8785 §3.2.3: compare decoded property names as unsigned UTF-16
        // code-unit sequences. Rust's native `str` order is UTF-8 and differs
        // for supplementary-plane characters.
        out.push('{');

        let mut first = true;

        let mut entries: Vec<_> = obj.iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
        for (key, value) in entries {
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
    let value: StrictValue = serde_json::from_str(s).map_err(|e| {
        let reason = e.to_string();
        if let Some(encoded_key) = reason
            .strip_prefix("duplicate object key: ")
            .and_then(|rest| rest.split(" at line ").next())
        {
            let key = serde_json::from_str(encoded_key)
                .unwrap_or_else(|_| encoded_key.trim_matches('"').to_owned());
            JcsError::DuplicateKey { key }
        } else {
            JcsError::InvalidJson { reason }
        }
    })?;
    Ok(value.0)
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictVisitor)
    }
}

struct StrictVisitor;
impl<'de> Visitor<'de> for StrictVisitor {
    type Value = StrictValue;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(v)))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(v.into())))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(v.into())))
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Number::from_f64(v)
            .map(|n| StrictValue(Value::Number(n)))
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.into())
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(v)))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        StrictValue::deserialize(d)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate object key: {key:?}"
                )));
            }
            values.insert(key, map.next_value::<StrictValue>()?.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

/// Validates that a JSON string parses to JSON, resolving duplicates first.
///
/// This is used for canonicalization inputs: the input need not be ordered,
/// but duplicates MUST be rejected before canonicalization.
pub fn parse_and_validate(input: &str) -> Result<Value, JcsError> {
    parse_with_dup_check(input)
}

/// Canonicalize with automatic duplicate detection first.
///
/// This canonicalizes any JSON (possibly with duplicate keys in the source),
/// returning an error if duplicates are found.
pub fn canonicalize_flexible(value: &Value) -> Result<String, JcsError> {
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
