//! Both existing raw-string entry points reject before map materialization.
use boundary_compiler::{parse_and_validate, parse_with_dup_check, Canonicalizer, JcsError};
use serde_json::{json, Value};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Parser = fn(&str) -> Result<Value, JcsError>;
const PARSERS: [Parser; 2] = [parse_with_dup_check, parse_and_validate];

#[test]
fn rejects_decoded_duplicates_at_every_object_boundary() {
    for parser in PARSERS {
        for raw in [
            r#"{"x":1,"x":2}"#,
            r#"{"x":1,"\u0078":2}"#,
            r#"{"outer":{"x":1,"x":2}}"#,
            r#"[{"x":1,"\u0078":2}]"#,
            r#"{"x":[{"child":{"a":1,"a":2}}]}"#,
        ] {
            assert!(
                matches!(parser(raw), Err(JcsError::DuplicateKey { .. })),
                "accepted duplicate: {raw}"
            );
        }
    }
}

#[test]
fn legal_sibling_objects_may_reuse_decoded_names() -> TestResult {
    for parser in PARSERS {
        for raw in [r#"[{"x":1},{"x":2}]"#, r#"{"a":{"x":1},"b":{"x":2}}"#] {
            assert_eq!(parser(raw)?, serde_json::from_str::<Value>(raw)?);
        }
    }
    Ok(())
}

#[test]
fn different_unicode_keys_are_not_normalized() -> TestResult {
    for parser in PARSERS {
        assert_eq!(
            parser(r#"{"é":1,"e\u0301":2}"#)?,
            json!({"é":1,"e\u{301}":2})
        );
    }
    Ok(())
}

#[test]
fn duplicate_error_contains_decoded_key() {
    for parser in PARSERS {
        match parser(r#"{"x":1,"\u0078":2}"#) {
            Err(JcsError::DuplicateKey { key }) => assert_eq!(key, "x"),
            other => panic!("expected decoded duplicate key, got {other:?}"),
        }
    }
}

#[test]
fn legacy_nonduplicate_error_categories_are_preserved() {
    for raw in ["{", "{} {}", r#"{"\uZZZZ":1}"#] {
        assert!(matches!(
            parse_with_dup_check(raw),
            Err(JcsError::InvalidJson { .. })
        ));
        assert!(matches!(
            parse_and_validate(raw),
            Err(JcsError::ParseError(_))
        ));
    }
}

#[test]
fn malformed_input_is_not_accepted_as_a_partial_value() {
    for parser in PARSERS {
        for raw in ["{} {}", "{", "NaN", "1e9999", r#""\ud800""#, r#"{"x":1,}"#] {
            assert!(parser(raw).is_err(), "accepted malformed: {raw}");
        }
    }
}

#[test]
fn legacy_numeric_and_canonical_bytes_do_not_migrate_to_v2() -> TestResult {
    for parser in PARSERS {
        for raw in [
            "18446744073709551615",
            "9007199254740993",
            "-0.0",
            "0.5",
            "1e22",
            r#"{"s":"\u0080"}"#,
        ] {
            let expected: Value = serde_json::from_str(raw)?;
            let actual = parser(raw)?;
            assert_eq!(actual, expected);
            assert_eq!(
                Canonicalizer::new().canonicalize(&actual)?,
                Canonicalizer::new().canonicalize(&expected)?
            );
        }
    }
    Ok(())
}

#[test]
fn escaped_quotes_backslashes_and_surrogate_pairs_are_decoded() {
    for parser in PARSERS {
        for raw in [
            r#"{"a\"b":1,"a\u0022b":2}"#,
            r#"{"a\\b":1,"a\u005cb":2}"#,
            r#"{"😀":1,"\ud83d\ude00":2}"#,
        ] {
            assert!(
                matches!(parser(raw), Err(JcsError::DuplicateKey { .. })),
                "missed decoded alias: {raw}"
            );
        }
    }
}
