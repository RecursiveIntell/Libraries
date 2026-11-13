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

/// Parse JSON with duplicate-key detection before map materialization.
///
/// Keys are compared after JSON escape decoding, separately for each object.
/// This preserves legacy value/numeric semantics; it does not select JCS V2.
pub fn parse_with_dup_check(s: &str) -> Result<Value, JcsError> {
    parse_and_validate(s).map_err(|error| match error {
        JcsError::ParseError(error) => JcsError::InvalidJson {
            reason: error.to_string(),
        },
        other => other,
    })
}

// A typed side channel preserves the decoded duplicate key without interpreting
// serde error strings (which may themselves contain caller-controlled text).
struct UniqueValueSeed<'a> {
    duplicate: &'a mut Option<String>,
}

impl<'de> serde::de::DeserializeSeed<'de> for UniqueValueSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for UniqueValueSeed<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value with unique decoded object keys")
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(UniqueValueSeed {
            duplicate: self.duplicate,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                *self.duplicate = Some(key);
                return Err(serde::de::Error::custom("duplicate JSON object key"));
            }
            let value = map.next_value_seed(UniqueValueSeed {
                duplicate: self.duplicate,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
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
    use serde::de::DeserializeSeed;

    let mut duplicate = None;
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let result = UniqueValueSeed {
        duplicate: &mut duplicate,
    }
    .deserialize(&mut deserializer)
    .and_then(|value| deserializer.end().map(|()| value));
    match result {
        Ok(value) => Ok(value),
        Err(error) => match duplicate {
            Some(key) => Err(JcsError::DuplicateKey { key }),
            None => Err(JcsError::ParseError(error)),
        },
    }
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
