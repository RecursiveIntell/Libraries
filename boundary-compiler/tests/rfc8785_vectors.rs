//! RED conformance witnesses for the Phase 2 RFC 8785 boundary.

use boundary_compiler::{parse_and_validate, Canonicalizer, ContentDigest};
use serde_json::Value;

#[test]
fn rfc8785_sorts_object_keys_by_utf16_code_units() {
    let value: Value = serde_json::from_str(r#"{"\uE000":1,"\uD834\uDD1E":2}"#).unwrap();
    let canonical = Canonicalizer::new().canonicalize(&value).unwrap();

    // U+1D11E is represented by the UTF-16 pair D834 DD1E and therefore
    // sorts before U+E000 under RFC 8785's UTF-16 ordering.
    assert_eq!(canonical, r#"{"𝄞":2,"":1}"#);
}

#[test]
fn rfc8785_formats_ecmascript_number_boundaries() {
    let cases = [
        ("-0", "0"),
        ("1e-6", "0.000001"),
        ("1e-7", "1e-7"),
        ("1e20", "100000000000000000000"),
        ("1e21", "1e+21"),
    ];

    for (input, expected) in cases {
        let value: Value = serde_json::from_str(input).unwrap();
        assert_eq!(
            Canonicalizer::new().canonicalize(&value).unwrap(),
            expected,
            "RFC 8785 number mismatch for {input}"
        );
    }
}

#[test]
fn parse_and_validate_rejects_duplicate_object_keys() {
    let result = parse_and_validate(r#"{"a":1,"a":2}"#);
    assert!(result.is_err(), "duplicate keys must fail closed");
}

#[test]
fn parse_and_validate_rejects_unpaired_unicode_surrogates() {
    let result = parse_and_validate(r#""\uD800""#);
    assert!(
        result.is_err(),
        "unpaired UTF-16 surrogate must fail closed"
    );
}

#[test]
fn content_digest_is_shared_and_hashes_canonical_bytes() {
    let value: Value = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
    let canonical = Canonicalizer::new().canonicalize_bytes(&value).unwrap();
    let digest = ContentDigest::compute(&canonical);

    assert_eq!(digest, stack_ids::ContentDigest::compute(&canonical));
    assert_eq!(digest.hex().len(), 64);
}

#[test]
fn parse_and_validate_rejects_escaped_duplicate_aliases() {
    let result = parse_and_validate(r#"{"a":1,"\u0061":2}"#);
    assert!(result.is_err(), "escaped duplicate keys must fail closed");
}

#[test]
fn non_finite_numbers_are_not_constructible_as_json() {
    assert!(serde_json::Number::from_f64(f64::NAN).is_none());
    assert!(serde_json::Number::from_f64(f64::INFINITY).is_none());
}

#[test]
fn rfc8785_section_3_2_2_number_vector() {
    let value: Value = serde_json::from_str(
        r#"{"numbers":[333333333.33333329,1E30,4.50,2e-3,0.000000000000000000000000001]}"#,
    )
    .unwrap();

    let canonical = Canonicalizer::new().canonicalize(&value).unwrap();
    assert_eq!(
        canonical,
        r#"{"numbers":[333333333.3333333,1e+30,4.5,0.002,1e-27]}"#
    );
}
