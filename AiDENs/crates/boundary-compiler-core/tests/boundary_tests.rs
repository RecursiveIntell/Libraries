//! Tests for boundary-compiler-core

use boundary_compiler_core::*;

#[test]
fn strict_json_parses_valid() {
    let input = br#"{"key": "value", "num": 42}"#;
    let result = strict_json::StrictJsonValue::parse(input);
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.total_object_keys(), 2);
}

#[test]
fn strict_json_rejects_malformed() {
    let input = br#"not valid json"#;
    let result = strict_json::StrictJsonValue::parse(input);
    assert!(result.is_err());
}

#[test]
fn digest_is_stable() {
    let input = b"{\"test\": true}";
    let d1 = digest::sha256_digest_hex(input);
    let d2 = digest::sha256_digest_hex(input);
    assert_eq!(d1, d2);
    assert!(!d1.is_empty());
}

#[test]
fn boundary_compiler_profile_has_defaults() {
    let profile = BoundaryCompilerProfileV1::strict_json_default();
    // Just verify it constructs without panic
    let _ = profile;
}

#[test]
fn compile_json_boundary_returns_result() {
    let profile = BoundaryCompilerProfileV1::strict_json_default();
    let input = br#"{"test": true}"#;
    let result = compile_json_boundary(&profile, input, None, &[]);
    // Verify it returns a result with digest
    assert!(!result.raw_digest.is_empty());
}