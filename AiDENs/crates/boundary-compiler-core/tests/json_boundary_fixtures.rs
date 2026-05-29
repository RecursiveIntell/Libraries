use boundary_compiler_core::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn base_profile() -> BoundaryCompilerProfileV1 {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.allowed_top_level_fields = Some(BTreeSet::from([
        "id".to_string(),
        "kind".to_string(),
        "amount".to_string(),
        "treatment".to_string(),
    ]));
    profile.unknown_field_policy = UnknownFieldPolicy::Reject;
    profile.expected_field_types = BTreeMap::from([
        ("id".to_string(), ExpectedJsonType::String),
        ("kind".to_string(), ExpectedJsonType::String),
        ("amount".to_string(), ExpectedJsonType::Number),
        ("treatment".to_string(), ExpectedJsonType::Object),
    ]);
    profile
}

#[test]
fn valid_minimal_json_is_accepted_and_gets_canonical_digest() {
    let profile = base_profile();
    let raw = br#"{"id":"evt-1","kind":"measurement","amount":42,"treatment":{"id":"trt-1"}}"#;
    let result = compile_json_boundary(&profile, raw, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Accept);
    assert!(result.canonical_digest.is_some());
    assert!(result.parse_receipt.canonical_digest.is_some());
}

#[test]
fn malformed_json_is_rejected_with_parse_receipt() {
    let profile = base_profile();
    let result = compile_json_boundary(&profile, br#"{"id":"evt-1""#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert_eq!(result.parse_receipt.status, ParseStatus::Rejected);
    assert!(result
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::MalformedJson));
}

#[test]
fn malformed_json_with_treatment_path_emits_integrity_receipt() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.treatment_critical_paths = vec!["/treatment/id".to_string()];
    let result = compile_json_boundary(&profile, br#"{"treatment":{"id":"trt-1""#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::MissingCriticalPath
        );
        assert_eq!(receipt.treatment_critical_paths, vec!["/treatment/id"]);
    }
}

#[test]
fn duplicate_key_is_rejected_or_quarantined() {
    let profile = base_profile();
    let result = compile_json_boundary(&profile, br#"{"id":"evt-1","id":"evt-2"}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.parse_receipt.ambiguity_detected);
}

#[test]
fn duplicate_key_is_not_silently_last_write_wins() {
    let profile = base_profile();
    let result = compile_json_boundary(&profile, br#"{"a":1,"a":2}"#, None, &[]);
    assert_ne!(result.decision, BoundaryDecisionV1::Accept);
    assert!(result.value.is_none());
    assert!(result
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::DuplicateKey));
}

#[test]
fn duplicate_key_with_treatment_path_emits_integrity_receipt() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.treatment_critical_paths = vec!["/a".to_string()];
    let result = compile_json_boundary(&profile, br#"{"a":1,"a":2}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::MissingCriticalPath
        );
    }
}

#[test]
fn duplicate_key_policy_can_quarantine_ambiguous_json() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.duplicate_key_policy = AmbiguityPolicy::Quarantine;
    let result = compile_json_boundary(&profile, br#"{"a":1,"a":2}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Quarantine);
    assert_eq!(result.parse_receipt.status, ParseStatus::Quarantined);
    assert!(result.parse_receipt.ambiguity_detected);
    assert!(result.value.is_none());
}

#[test]
fn unknown_field_policy_rejects_surprise_structure() {
    let profile = base_profile();
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","surprise":true}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::UnknownField));
}

#[test]
fn unknown_field_policy_can_quarantine_surprise_structure() {
    let mut profile = base_profile();
    profile.unknown_field_policy = UnknownFieldPolicy::Quarantine;
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","surprise":true}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Quarantine);
    assert_eq!(result.parse_receipt.status, ParseStatus::Quarantined);
    assert!(result
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::UnknownField));
}

#[test]
fn schema_properties_provide_allowed_fields_and_types() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.unknown_field_policy = UnknownFieldPolicy::Reject;
    let schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "amount": { "type": "number" }
        }
    });

    let wrong_type = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","amount":"42"}"#,
        Some(&schema),
        &[],
    );
    assert_eq!(wrong_type.decision, BoundaryDecisionV1::Reject);
    assert!(wrong_type
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::TypeMismatch));

    let unknown = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","amount":42,"surprise":true}"#,
        Some(&schema),
        &[],
    );
    assert_eq!(unknown.decision, BoundaryDecisionV1::Reject);
    assert!(unknown
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::UnknownField));

    let accepted = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","amount":42}"#,
        Some(&schema),
        &[],
    );
    assert_eq!(accepted.decision, BoundaryDecisionV1::Accept);
}

#[test]
fn string_number_coercion_is_rejected_by_default() {
    let profile = base_profile();
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":"42"}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result
        .errors
        .iter()
        .any(|e| e.kind == BoundaryErrorKind::TypeMismatch));
}

#[test]
fn treatment_critical_type_mismatch_marks_integrity_changed() {
    let mut profile = base_profile();
    profile.treatment_critical_paths = vec!["/amount".to_string()];
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":"42"}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::ChangedWithoutWaiver
        );
        assert!(receipt
            .differences
            .iter()
            .any(|difference| difference.path == "/amount"));
    }
}

#[test]
fn treatment_critical_unknown_field_marks_integrity_changed() {
    let mut profile = base_profile();
    profile.treatment_critical_paths = vec!["/surprise".to_string()];
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","surprise":true}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::ChangedWithoutWaiver
        );
    }
}

#[test]
fn resource_ceiling_rejects_large_input() {
    let mut profile = base_profile();
    profile.resource_ceilings.max_bytes = Some(4);
    let result = compile_json_boundary(&profile, br#"{"id":"evt-1"}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert_eq!(
        result.parse_receipt.resource_ceiling_triggered.as_deref(),
        Some("max_bytes")
    );
}

#[test]
fn resource_ceiling_large_input_with_treatment_path_emits_integrity_receipt() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.resource_ceilings.max_bytes = Some(4);
    profile.treatment_critical_paths = vec!["/id".to_string()];
    let result = compile_json_boundary(&profile, br#"{"id":"evt-1"}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::MissingCriticalPath
        );
    }
}

#[test]
fn resource_ceiling_rejects_deep_input() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.resource_ceilings.max_nesting_depth = Some(3);
    let result = compile_json_boundary(&profile, br#"{"a":{"b":{"c":{"d":1}}}}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert_eq!(
        result.parse_receipt.resource_ceiling_triggered.as_deref(),
        Some("max_nesting_depth")
    );
}

#[test]
fn resource_ceiling_rejects_too_many_object_keys() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.resource_ceilings.max_object_keys = Some(1);
    let result = compile_json_boundary(&profile, br#"{"a":1,"b":2}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert_eq!(
        result.parse_receipt.resource_ceiling_triggered.as_deref(),
        Some("max_object_keys")
    );
}

#[test]
fn treatment_critical_missing_path_requires_integrity_receipt() {
    let mut profile = base_profile();
    profile.treatment_critical_paths = vec!["/treatment/id".to_string()];
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":42}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::MissingCriticalPath
        );
    }
}

#[test]
fn function_argument_treatment_paths_override_profile_paths() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.treatment_critical_paths = vec!["/present".to_string()];
    let result = compile_json_boundary(&profile, br#"{"present":true}"#, None, &["/absent".into()]);
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(receipt.treatment_critical_paths, vec!["/absent"]);
        assert_eq!(
            receipt.decision,
            TreatmentIntegrityDecision::MissingCriticalPath
        );
    }
}

#[test]
fn json_pointer_escaping_is_honored_for_treatment_paths() {
    let profile = BoundaryCompilerProfileV1::strict_json_default();
    let result = compile_json_boundary(
        &profile,
        br#"{"a/b":1,"tilde~key":2}"#,
        None,
        &["/a~1b".into(), "/tilde~0key".into()],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Accept);
    assert!(result.treatment_integrity_receipt.is_some());
    if let Some(receipt) = result.treatment_integrity_receipt.as_ref() {
        assert_eq!(receipt.decision, TreatmentIntegrityDecision::Preserved);
        assert!(receipt
            .after_hashes
            .get("/a~1b")
            .and_then(Option::as_ref)
            .is_some());
        assert!(receipt
            .after_hashes
            .get("/tilde~0key")
            .and_then(Option::as_ref)
            .is_some());
    }
}

#[test]
fn no_repair_policy_never_emits_fake_repair_accept() {
    let profile = base_profile();
    let result = compile_json_boundary(&profile, br#"{"id":"evt-1""#, None, &[]);
    assert_ne!(result.decision, BoundaryDecisionV1::RepairedAccept);
    assert!(result.repair_receipt.is_none());
}

#[test]
fn canonical_digest_is_stable_for_equivalent_object_ordering() {
    let profile = base_profile();
    let a = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":42}"#,
        None,
        &[],
    );
    let b = compile_json_boundary(
        &profile,
        br#"{"amount":42,"kind":"measurement","id":"evt-1"}"#,
        None,
        &[],
    );
    assert_eq!(a.decision, BoundaryDecisionV1::Accept);
    assert_eq!(b.decision, BoundaryDecisionV1::Accept);
    assert_eq!(a.canonical_digest, b.canonical_digest);
}

#[test]
fn canonical_bytes_are_stable_sorted_json_v1() {
    let profile = BoundaryCompilerProfileV1::strict_json_default();
    let result = compile_json_boundary(&profile, br#"{"z":1,"a":[true,null]}"#, None, &[]);
    assert_eq!(result.decision, BoundaryDecisionV1::Accept);
    assert_eq!(
        result.canonical_bytes.as_deref(),
        Some(br#"{"a":[true,null],"z":1}"#.as_slice())
    );
}

#[test]
fn accepted_and_rejected_results_both_have_receipts() {
    let profile = base_profile();
    let accepted = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":42}"#,
        None,
        &[],
    );
    let rejected = compile_json_boundary(&profile, br#"{"id":"evt-1""#, None, &[]);
    assert_eq!(accepted.parse_receipt.status, ParseStatus::Accepted);
    assert_eq!(rejected.parse_receipt.status, ParseStatus::Rejected);
    assert!(!accepted.parse_receipt.receipt_id.is_empty());
    assert!(!rejected.parse_receipt.receipt_id.is_empty());
}

#[test]
fn compile_results_are_serializable_receipt_artifacts() -> Result<(), serde_json::Error> {
    let profile = base_profile();
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":42}"#,
        None,
        &[],
    );
    let encoded = serde_json::to_string(&result)?;
    let decoded: BoundaryCompileResultV1 = serde_json::from_str(&encoded)?;
    assert_eq!(decoded.decision, BoundaryDecisionV1::Accept);
    assert_eq!(decoded.parse_receipt.status, ParseStatus::Accepted);
    assert_eq!(decoded.canonical_digest, result.canonical_digest);
    Ok(())
}

#[test]
fn raw_digest_is_sha256_over_original_bytes_even_on_reject() {
    let mut profile = BoundaryCompilerProfileV1::strict_json_default();
    profile.resource_ceilings.max_bytes = Some(4);
    let raw = br#"{"id":"evt-1"}"#;
    let result = compile_json_boundary(&profile, raw, None, &[]);
    let mut hasher = Sha256::new();
    hasher.update(raw);
    let expected = format!("sha256:{:x}", hasher.finalize());
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert_eq!(result.raw_digest, expected);
    assert_eq!(result.parse_receipt.raw_digest, expected);
}

#[test]
fn post_parse_rejections_keep_receipt_canonical_digest_but_no_value() {
    let profile = base_profile();
    let result = compile_json_boundary(
        &profile,
        br#"{"id":"evt-1","kind":"measurement","amount":"42"}"#,
        None,
        &[],
    );
    assert_eq!(result.decision, BoundaryDecisionV1::Reject);
    assert!(result.value.is_none());
    assert!(result.canonical_digest.is_none());
    assert!(result.parse_receipt.canonical_digest.is_some());
    assert_eq!(
        result.parse_receipt.canonical_digest,
        result.parse_receipt.parsed_digest
    );
}

#[test]
fn no_repair_policy_never_emits_repair_receipts_across_terminal_decisions() {
    let mut quarantine_profile = BoundaryCompilerProfileV1::strict_json_default();
    quarantine_profile.duplicate_key_policy = AmbiguityPolicy::Quarantine;
    let quarantine = compile_json_boundary(&quarantine_profile, br#"{"a":1,"a":2}"#, None, &[]);

    let reject = compile_json_boundary(
        &BoundaryCompilerProfileV1::strict_json_default(),
        br#"{"a":1,"a":2}"#,
        None,
        &[],
    );

    let accept = compile_json_boundary(
        &BoundaryCompilerProfileV1::strict_json_default(),
        br#"{"a":1}"#,
        None,
        &[],
    );

    for result in [accept, reject, quarantine] {
        assert_ne!(result.decision, BoundaryDecisionV1::RepairedAccept);
        assert!(result.repair_receipt.is_none());
    }
}
