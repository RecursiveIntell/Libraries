//! Conformance run integration test — exercises boundary-compiler-core fixtures
//! through ConformanceRunReceiptV1.

use aidens_contracts::{
    ArtifactId, ConformanceEnvironmentV1, ConformanceFixtureResultV1, ConformanceRunReceiptV1,
    DisplayDigestV1,
};
use boundary_compiler_core::compile_json_boundary;
use std::collections::{BTreeMap, BTreeSet};

fn base_profile() -> boundary_compiler_core::BoundaryCompilerProfileV1 {
    let mut profile = boundary_compiler_core::BoundaryCompilerProfileV1::strict_json_default();
    profile.allowed_top_level_fields = Some(BTreeSet::from([
        "id".to_string(),
        "kind".to_string(),
        "amount".to_string(),
        "treatment".to_string(),
    ]));
    profile.unknown_field_policy = boundary_compiler_core::UnknownFieldPolicy::Reject;
    profile.expected_field_types = BTreeMap::from([
        (
            "id".to_string(),
            boundary_compiler_core::ExpectedJsonType::String,
        ),
        (
            "kind".to_string(),
            boundary_compiler_core::ExpectedJsonType::String,
        ),
        (
            "amount".to_string(),
            boundary_compiler_core::ExpectedJsonType::Number,
        ),
        (
            "treatment".to_string(),
            boundary_compiler_core::ExpectedJsonType::Object,
        ),
    ]);
    profile
}

fn digest_to_display(opt: Option<String>) -> Option<DisplayDigestV1> {
    opt.map(DisplayDigestV1::from_hex)
}

#[test]
fn conformance_run_receipt_records_all_fixture_results() {
    let profile = base_profile();
    let fixtures: Vec<(&str, &[u8])> = vec![
        ("bc-fixture-001-valid-minimal", br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1"}}"#),
        ("bc-fixture-002-malformed", br#"{"id":"evt-1""#),
        ("bc-fixture-003-unknown-field", br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1"},"unknown":true}"#),
        ("bc-fixture-004-duplicate-key", br#"{"id":"evt-1","kind":"measurement","amount":42,"amount":99,"treatment":{"id":"trt-1"}}"#),
        ("bc-fixture-005-type-mismatch", br#"{"id":"evt-1","kind":"measurement","amount":"not-a-number","treatment":{"id":"trt-1"}}"#),
        ("bc-fixture-008-nested-treatment", br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1","dose":5.as_str()}}"#),
    ];

    let mut results = Vec::new();
    for (fixture_id, input) in &fixtures {
        let result = compile_json_boundary(&profile, input, None, &[]);
        let passed = matches!(
            result.decision,
            boundary_compiler_core::BoundaryDecisionV1::Accept
        );
        let digest = result.parse_receipt.canonical_digest.clone();
        results.push(ConformanceFixtureResultV1 {
            fixture_id: fixture_id.to_string(),
            passed,
            input_digest: digest_to_display(digest),
            expected_digest: None,
            actual_digest: digest_to_display(result.parse_receipt.canonical_digest.clone()),
            reason_codes: vec![format!("{:?}", result.decision)],
        });
    }

    let profile_id = ArtifactId::new(format!("boundary-compiler-profile:{}", profile.profile_id));
    let receipt = ConformanceRunReceiptV1::new(
        profile_id,
        results,
        Some(ConformanceEnvironmentV1 {
            rustc_version: String::new(),
            target_triple: String::new(),
            ci_run_id: None,
        }),
    );

    // Verify receipt structure, not specific pass/fail counts (those depend on profile config)
    assert_eq!(receipt.fixture_count, 6);
    assert!(receipt.passed_count + receipt.failed_count == receipt.fixture_count);
    assert!(
        receipt.passed_count >= 1,
        "at least valid_minimal should pass"
    );
    assert!(receipt.receipt_id.as_str().starts_with("conformance-run:"));
}

#[test]
fn conformance_run_receipt_all_passes() {
    let profile = base_profile();
    let input = br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1"}}"#;
    let result = compile_json_boundary(&profile, input, None, &[]);
    let digest = result.parse_receipt.canonical_digest.clone();

    let fixture_result = ConformanceFixtureResultV1 {
        fixture_id: "bc-fixture-001".to_string(),
        passed: true,
        input_digest: digest_to_display(digest),
        expected_digest: None,
        actual_digest: digest_to_display(result.parse_receipt.canonical_digest.clone()),
        reason_codes: vec!["boundary-compile-accepted".to_string()],
    };

    let profile_id = ArtifactId::new(format!("boundary-compiler-profile:{}", profile.profile_id));
    let receipt = ConformanceRunReceiptV1::new(profile_id, vec![fixture_result], None);

    assert_eq!(receipt.fixture_count, 1);
    assert_eq!(receipt.passed_count, 1);
    assert_eq!(receipt.failed_count, 0);
    assert!(receipt
        .reason_codes
        .contains(&"conformance-run-passed".to_string()));
}

#[test]
fn conformance_run_receipt_serializes_round_trip() {
    let profile = base_profile();
    let input = br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1"}}"#;
    let result = compile_json_boundary(&profile, input, None, &[]);
    let digest = result.parse_receipt.canonical_digest.clone();

    let fixture_result = ConformanceFixtureResultV1 {
        fixture_id: "bc-fixture-001".to_string(),
        passed: true,
        input_digest: digest_to_display(digest),
        expected_digest: None,
        actual_digest: digest_to_display(result.parse_receipt.canonical_digest.clone()),
        reason_codes: vec!["boundary-compile-accepted".to_string()],
    };

    let profile_id = ArtifactId::new(format!("boundary-compiler-profile:{}", profile.profile_id));
    let receipt = ConformanceRunReceiptV1::new(
        profile_id,
        vec![fixture_result],
        Some(ConformanceEnvironmentV1 {
            rustc_version: "1.85.as_str()".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            ci_run_id: Some("test-ci-001".to_string()),
        }),
    );

    let json = serde_json::to_string(&receipt).expect("serialize receipt");
    let deserialized: ConformanceRunReceiptV1 =
        serde_json::from_str(&json).expect("deserialize receipt");

    assert_eq!(receipt.receipt_id, deserialized.receipt_id);
    assert_eq!(receipt.fixture_count, deserialized.fixture_count);
    assert_eq!(receipt.passed_count, deserialized.passed_count);
    assert_eq!(receipt.failed_count, deserialized.failed_count);
}
