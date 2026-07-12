//! Integration tests for boundary-compiler RFC 8785 JCS canonicalization.

use boundary_compiler::{parse_with_dup_check, Canonicalizer, ContentDigest};
use serde_json::json;

#[test]
fn decoded_duplicate_keys_are_object_scoped() {
    assert!(parse_with_dup_check(r#"{"a":1,"\u0061":2}"#).is_err());
    assert!(parse_with_dup_check(r#"{"left":{"x":1},"right":{"x":2}}"#).is_ok());
}

#[test]
fn jcs_utf16_and_ecmascript_number_vectors() {
    let value = parse_with_dup_check(r#"{"\uE000":1,"\uD800\uDC00":2}"#).unwrap();
    assert_eq!(
        Canonicalizer::new().canonicalize(&value).unwrap(),
        "{\"𐀀\":2,\"\":1}"
    );
    assert_eq!(
        Canonicalizer::new()
            .canonicalize(&parse_with_dup_check("-0").unwrap())
            .unwrap(),
        "0"
    );
    assert_eq!(
        Canonicalizer::new()
            .canonicalize(&parse_with_dup_check("9007199254740993").unwrap())
            .unwrap(),
        "9007199254740992"
    );
}

#[test]
fn schema_validator_fails_closed_without_schema() {
    assert!(boundary_compiler::SchemaValidator::new()
        .validate(&json!({}))
        .is_err());
}

/// Test 1: rfc8785_basic_object — {"b":1,"a":2} → keys sorted to {"a":2,"b":1}.
#[test]
fn rfc8785_basic_object() {
    let c = Canonicalizer::new();
    let val = json!({"b": 1, "a": 2});
    let canonical = c.canonicalize(&val).unwrap();
    assert_eq!(canonical, r#"{"a":2,"b":1}"#);
}

/// Test 2: rfc8785_nested — nested objects sort recursively.
#[test]
fn rfc8785_nested() {
    let c = Canonicalizer::new();
    let val = json!({
        "z": {"b": 1, "a": 2},
        "m": [3, 2, 1],
        "a": "hello"
    });
    let canonical = c.canonicalize(&val).unwrap();
    // Keys at top level: a, m, z (alphabetical)
    // "z" nested has keys: a, b (alphabetical)
    assert_eq!(canonical, r#"{"a":"hello","m":[3,2,1],"z":{"a":2,"b":1}}"#);
}

/// Test 3: rfc8785_arrays_preserve_order — [3,1,2] stays [3,1,2].
#[test]
fn rfc8785_arrays_preserve_order() {
    let c = Canonicalizer::new();
    let val = json!([3, 1, 2]);
    let canonical = c.canonicalize(&val).unwrap();
    // Arrays are NOT sorted — order is preserved per RFC 8785
    assert_eq!(canonical, r#"[3,1,2]"#);
}

/// Test 4: rfc8785_number_formatting — standard number serialization.
#[test]
fn rfc8785_number_formatting() {
    let c = Canonicalizer::new();
    // Integer
    assert_eq!(c.canonicalize(&json!(42)).unwrap(), "42");
    // Negative
    assert_eq!(c.canonicalize(&json!(-1)).unwrap(), "-1");
    // Float
    assert_eq!(c.canonicalize(&json!(0.5)).unwrap(), "0.5");
    // Scientific notation if serde_json produces it
    let scientific = json!(1e10);
    let result = c.canonicalize(&scientific).unwrap();
    assert!(result.contains("1e") || result.contains("10"));
}

/// Test 5: duplicate_key_rejection — {"a":1,"a":2} → Err.
#[test]
fn duplicate_key_rejection() {
    // JSON with duplicate keys at same nesting level
    let result = parse_with_dup_check(r#"{"a": 1, "a": 2}"#);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("duplicate") || err.to_string().contains("DuplicateKey"));
}

/// Test: nested duplicate keys are also rejected.
#[test]
fn duplicate_key_nested_rejection() {
    let result = parse_with_dup_check(r#"{"outer": {"x": 1, "x": 2}}"#);
    assert!(result.is_err());
}

/// Test: ContentDigest computes for canonical JSON.
#[test]
fn content_digest_computation() {
    let val = json!({"b": 2, "a": 1});
    let digest = ContentDigest::compute(&val).unwrap();
    // Digest hex should be non-empty
    let hex = digest.hex();
    assert!(!hex.is_empty());
    // Should be deterministic — same input always gives same digest
    let digest2 = ContentDigest::compute(&val).unwrap();
    assert_eq!(digest.hex(), digest2.hex());
}

/// Test: canonicalization with parse_and_validate accepts valid JSON.
#[test]
fn parse_and_validate_accepts_valid() {
    let input = r#"{"z": 1, "a": 2, "m": 3}"#;
    let val = boundary_compiler::parse_and_validate(input).unwrap();
    let c = Canonicalizer::new();
    let canonical = c.canonicalize(&val).unwrap();
    assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
}
