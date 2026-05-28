use crate::canonical::canonicalize_stable_sorted_json;
use crate::digest::sha256_digest_hex;
use crate::strict_json::StrictJsonValue;
use crate::treatment::{treatment_receipt_for_paths, treatment_receipt_for_unparsed_boundary};
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn compile_json_boundary(
    profile: &BoundaryCompilerProfileV1,
    raw: &[u8],
    schema: Option<&serde_json::Value>,
    treatment_critical_paths: &[JsonPointerLikePath],
) -> BoundaryCompileResultV1 {
    let raw_digest = sha256_digest_hex(raw);
    let mut errors = Vec::new();
    let mut ambiguity_detected = false;
    let mut resource_ceiling_triggered = None;
    let treatment_paths = selected_treatment_paths(profile, treatment_critical_paths);

    if let Some(max_bytes) = profile.resource_ceilings.max_bytes {
        if raw.len() > max_bytes {
            resource_ceiling_triggered = Some("max_bytes".to_string());
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::ResourceCeiling,
                path: None,
                message: format!("input length {} exceeds max_bytes {}", raw.len(), max_bytes),
            });
            let treatment_integrity_receipt = treatment_receipt_for_unparsed_boundary(
                &raw_digest,
                &treatment_paths,
                "treatment-critical path could not be proven because max_bytes was exceeded before parsing",
            );
            return rejected_result_with_treatment(
                profile,
                raw_digest,
                errors,
                TerminalMetadata {
                    ambiguity_detected,
                    resource_ceiling_triggered,
                    treatment_integrity_receipt,
                    parsed_digest: None,
                    canonical_digest: None,
                },
            );
        }
    }

    if profile.language != BoundaryLanguage::Json {
        errors.push(BoundaryErrorRecordV1 {
            kind: BoundaryErrorKind::UnsupportedLanguage,
            path: None,
            message: "only JSON is implemented by the P31 scaffold".to_string(),
        });
        let treatment_integrity_receipt = treatment_receipt_for_unparsed_boundary(
            &raw_digest,
            &treatment_paths,
            "treatment-critical path could not be proven because the boundary language was unsupported",
        );
        return rejected_result_with_treatment(
            profile,
            raw_digest,
            errors,
            TerminalMetadata {
                ambiguity_detected,
                resource_ceiling_triggered,
                treatment_integrity_receipt,
                parsed_digest: None,
                canonical_digest: None,
            },
        );
    }

    let strict_value = match StrictJsonValue::parse(raw) {
        Ok(value) => value,
        Err(err) => {
            let message = err.to_string();
            let is_duplicate = message.contains("duplicate key");
            if is_duplicate {
                ambiguity_detected = true;
                errors.push(BoundaryErrorRecordV1 {
                    kind: BoundaryErrorKind::DuplicateKey,
                    path: None,
                    message,
                });
                let treatment_integrity_receipt = treatment_receipt_for_unparsed_boundary(
                    &raw_digest,
                    &treatment_paths,
                    "treatment-critical path could not be proven because duplicate-key ambiguity was detected",
                );
                return match profile.duplicate_key_policy {
                    AmbiguityPolicy::Reject => rejected_result_with_treatment(
                        profile,
                        raw_digest,
                        errors,
                        TerminalMetadata {
                            ambiguity_detected,
                            resource_ceiling_triggered,
                            treatment_integrity_receipt,
                            parsed_digest: None,
                            canonical_digest: None,
                        },
                    ),
                    AmbiguityPolicy::Quarantine => quarantine_result(
                        profile,
                        raw_digest,
                        errors,
                        TerminalMetadata {
                            ambiguity_detected,
                            resource_ceiling_triggered,
                            treatment_integrity_receipt,
                            parsed_digest: None,
                            canonical_digest: None,
                        },
                    ),
                };
            }
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::MalformedJson,
                path: None,
                message,
            });
            let treatment_integrity_receipt = treatment_receipt_for_unparsed_boundary(
                &raw_digest,
                &treatment_paths,
                "treatment-critical path could not be proven because JSON parsing failed",
            );
            return rejected_result_with_treatment(
                profile,
                raw_digest,
                errors,
                TerminalMetadata {
                    ambiguity_detected,
                    resource_ceiling_triggered,
                    treatment_integrity_receipt,
                    parsed_digest: None,
                    canonical_digest: None,
                },
            );
        }
    };

    if let Some(max_depth) = profile.resource_ceilings.max_nesting_depth {
        let depth = strict_value.max_depth();
        if depth > max_depth {
            resource_ceiling_triggered = Some("max_nesting_depth".to_string());
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::ResourceCeiling,
                path: None,
                message: format!(
                    "nesting depth {} exceeds max_nesting_depth {}",
                    depth, max_depth
                ),
            });
        }
    }

    if let Some(max_keys) = profile.resource_ceilings.max_object_keys {
        let key_count = strict_value.total_object_keys();
        if key_count > max_keys {
            resource_ceiling_triggered = Some("max_object_keys".to_string());
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::ResourceCeiling,
                path: None,
                message: format!(
                    "object key count {} exceeds max_object_keys {}",
                    key_count, max_keys
                ),
            });
        }
    }

    let schema_rules = schema.and_then(minimal_schema_rules);
    check_unknown_fields(profile, schema_rules.as_ref(), &strict_value, &mut errors);
    check_expected_types(profile, schema_rules.as_ref(), &strict_value, &mut errors);

    let treatment_integrity_receipt =
        treatment_receipt_for_paths(&raw_digest, &strict_value, &treatment_paths);

    if let Some(receipt) = &treatment_integrity_receipt {
        if receipt.decision == TreatmentIntegrityDecision::MissingCriticalPath {
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::TreatmentCriticalMissing,
                path: None,
                message: "one or more treatment-critical paths are missing".to_string(),
            });
        }
    }

    if !errors.is_empty() {
        let terminal = terminal_post_parse_result(
            profile,
            &raw_digest,
            &strict_value,
            errors,
            ambiguity_detected,
            resource_ceiling_triggered,
            treatment_integrity_receipt,
        );
        return terminal;
    }

    let canonical_bytes = match canonicalize_stable_sorted_json(&strict_value) {
        Ok(bytes) => bytes,
        Err(err) => {
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::CanonicalizationFailure,
                path: None,
                message: err.to_string(),
            });
            return rejected_result_with_treatment(
                profile,
                raw_digest,
                errors,
                TerminalMetadata {
                    ambiguity_detected,
                    resource_ceiling_triggered,
                    treatment_integrity_receipt,
                    parsed_digest: None,
                    canonical_digest: None,
                },
            );
        }
    };
    let canonical_digest = sha256_digest_hex(&canonical_bytes);
    let value = strict_value.to_json_value();
    let parsed_digest = canonical_digest.clone();

    let parse_receipt = ParseReceiptV1 {
        receipt_id: format!("parse:{}", raw_digest.trim_start_matches("sha256:")),
        raw_digest: raw_digest.clone(),
        parsed_digest: Some(parsed_digest),
        canonical_digest: Some(canonical_digest.clone()),
        parser: "strict-json-boundary-v1".to_string(),
        dialect: profile.dialect.clone(),
        status: ParseStatus::Accepted,
        errors: Vec::new(),
        ambiguity_detected,
        resource_ceiling_triggered,
    };

    BoundaryCompileResultV1 {
        decision: BoundaryDecisionV1::Accept,
        value: Some(value),
        canonical_bytes: Some(canonical_bytes),
        raw_digest,
        canonical_digest: Some(canonical_digest),
        parse_receipt,
        repair_receipt: None,
        treatment_integrity_receipt,
        errors: Vec::new(),
    }
}

fn check_unknown_fields(
    profile: &BoundaryCompilerProfileV1,
    schema_rules: Option<&MinimalSchemaRules>,
    value: &StrictJsonValue,
    errors: &mut Vec<BoundaryErrorRecordV1>,
) {
    let allowed = profile
        .allowed_top_level_fields
        .as_ref()
        .or_else(|| schema_rules.and_then(|rules| rules.allowed_top_level_fields.as_ref()));
    let Some(allowed) = allowed else {
        return;
    };
    let StrictJsonValue::Object(map) = value else {
        return;
    };
    if profile.unknown_field_policy == UnknownFieldPolicy::Allow {
        return;
    }
    for key in map.keys() {
        if !allowed.contains(key) {
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::UnknownField,
                path: Some(format!("/{key}")),
                message: format!("unknown top-level field: {key}"),
            });
        }
    }
}

fn check_expected_types(
    profile: &BoundaryCompilerProfileV1,
    schema_rules: Option<&MinimalSchemaRules>,
    value: &StrictJsonValue,
    errors: &mut Vec<BoundaryErrorRecordV1>,
) {
    if profile.coercion_policy != CoercionPolicy::RejectByDefault {
        return;
    }
    let StrictJsonValue::Object(map) = value else {
        return;
    };
    let expected_field_types = if profile.expected_field_types.is_empty() {
        schema_rules.map(|rules| &rules.expected_field_types)
    } else {
        Some(&profile.expected_field_types)
    };
    let Some(expected_field_types) = expected_field_types else {
        return;
    };
    for (field, expected) in expected_field_types {
        let Some(actual) = map.get(field) else {
            continue;
        };
        if !matches_expected_type(actual, expected) {
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::TypeMismatch,
                path: Some(format!("/{field}")),
                message: format!(
                    "field {field} has type {}, expected {:?}; coercion is disabled",
                    actual.json_type_name(),
                    expected
                ),
            });
        }
    }
}

fn matches_expected_type(value: &StrictJsonValue, expected: &ExpectedJsonType) -> bool {
    matches!(
        (value, expected),
        (StrictJsonValue::Null, ExpectedJsonType::Null)
            | (StrictJsonValue::Bool(_), ExpectedJsonType::Bool)
            | (StrictJsonValue::Number(_), ExpectedJsonType::Number)
            | (StrictJsonValue::String(_), ExpectedJsonType::String)
            | (StrictJsonValue::Array(_), ExpectedJsonType::Array)
            | (StrictJsonValue::Object(_), ExpectedJsonType::Object)
    )
}

fn selected_treatment_paths(
    profile: &BoundaryCompilerProfileV1,
    treatment_critical_paths: &[JsonPointerLikePath],
) -> Vec<JsonPointerLikePath> {
    if treatment_critical_paths.is_empty() {
        profile.treatment_critical_paths.clone()
    } else {
        treatment_critical_paths.to_vec()
    }
}

fn rejected_result_with_treatment(
    profile: &BoundaryCompilerProfileV1,
    raw_digest: DigestHex,
    errors: Vec<BoundaryErrorRecordV1>,
    metadata: TerminalMetadata,
) -> BoundaryCompileResultV1 {
    let parse_receipt = ParseReceiptV1 {
        receipt_id: format!("parse:{}", raw_digest.trim_start_matches("sha256:")),
        raw_digest: raw_digest.clone(),
        parsed_digest: metadata.parsed_digest,
        canonical_digest: metadata.canonical_digest,
        parser: "strict-json-boundary-v1".to_string(),
        dialect: profile.dialect.clone(),
        status: ParseStatus::Rejected,
        errors: errors.clone(),
        ambiguity_detected: metadata.ambiguity_detected,
        resource_ceiling_triggered: metadata.resource_ceiling_triggered,
    };
    BoundaryCompileResultV1 {
        decision: BoundaryDecisionV1::Reject,
        value: None,
        canonical_bytes: None,
        raw_digest,
        canonical_digest: None,
        parse_receipt,
        repair_receipt: None,
        treatment_integrity_receipt: metadata.treatment_integrity_receipt,
        errors,
    }
}

fn quarantine_result(
    profile: &BoundaryCompilerProfileV1,
    raw_digest: DigestHex,
    errors: Vec<BoundaryErrorRecordV1>,
    metadata: TerminalMetadata,
) -> BoundaryCompileResultV1 {
    let parse_receipt = ParseReceiptV1 {
        receipt_id: format!("parse:{}", raw_digest.trim_start_matches("sha256:")),
        raw_digest: raw_digest.clone(),
        parsed_digest: metadata.parsed_digest,
        canonical_digest: metadata.canonical_digest,
        parser: "strict-json-boundary-v1".to_string(),
        dialect: profile.dialect.clone(),
        status: ParseStatus::Quarantined,
        errors: errors.clone(),
        ambiguity_detected: metadata.ambiguity_detected,
        resource_ceiling_triggered: metadata.resource_ceiling_triggered,
    };
    BoundaryCompileResultV1 {
        decision: BoundaryDecisionV1::Quarantine,
        value: None,
        canonical_bytes: None,
        raw_digest,
        canonical_digest: None,
        parse_receipt,
        repair_receipt: None,
        treatment_integrity_receipt: metadata.treatment_integrity_receipt,
        errors,
    }
}

fn terminal_post_parse_result(
    profile: &BoundaryCompilerProfileV1,
    raw_digest: &DigestHex,
    strict_value: &StrictJsonValue,
    mut errors: Vec<BoundaryErrorRecordV1>,
    ambiguity_detected: bool,
    resource_ceiling_triggered: Option<String>,
    mut treatment_integrity_receipt: Option<TreatmentIntegrityReceiptV1>,
) -> BoundaryCompileResultV1 {
    mark_treatment_touched_by_errors(&mut treatment_integrity_receipt, &errors);

    let (parsed_digest, canonical_digest) = match canonicalize_stable_sorted_json(strict_value) {
        Ok(bytes) => {
            let digest = sha256_digest_hex(&bytes);
            (Some(digest.clone()), Some(digest))
        }
        Err(err) => {
            errors.push(BoundaryErrorRecordV1 {
                kind: BoundaryErrorKind::CanonicalizationFailure,
                path: None,
                message: err.to_string(),
            });
            (None, None)
        }
    };

    if should_quarantine_unknown_fields(profile, &errors) {
        return quarantine_result(
            profile,
            raw_digest.clone(),
            errors,
            TerminalMetadata {
                ambiguity_detected,
                resource_ceiling_triggered,
                treatment_integrity_receipt,
                parsed_digest,
                canonical_digest,
            },
        );
    }

    rejected_result_with_treatment(
        profile,
        raw_digest.clone(),
        errors,
        TerminalMetadata {
            ambiguity_detected,
            resource_ceiling_triggered,
            treatment_integrity_receipt,
            parsed_digest,
            canonical_digest,
        },
    )
}

fn mark_treatment_touched_by_errors(
    receipt: &mut Option<TreatmentIntegrityReceiptV1>,
    errors: &[BoundaryErrorRecordV1],
) {
    let Some(receipt) = receipt else {
        return;
    };
    if receipt.decision == TreatmentIntegrityDecision::MissingCriticalPath {
        return;
    }

    let mut touched = false;
    for critical_path in &receipt.treatment_critical_paths {
        for error in errors.iter().filter(|error| is_semantic_touch(error)) {
            let Some(error_path) = error.path.as_deref() else {
                continue;
            };
            if json_pointer_overlaps(critical_path, error_path) {
                touched = true;
                receipt.differences.push(TreatmentDifferenceV1 {
                    path: critical_path.clone(),
                    before_digest: None,
                    after_digest: receipt.after_hashes.get(critical_path).cloned().flatten(),
                    description: format!(
                        "treatment-critical path touched by boundary error {:?}: {}",
                        error.kind, error.message
                    ),
                });
            }
        }
    }

    if touched {
        receipt.decision = TreatmentIntegrityDecision::ChangedWithoutWaiver;
    }
}

fn is_semantic_touch(error: &BoundaryErrorRecordV1) -> bool {
    matches!(
        error.kind,
        BoundaryErrorKind::TypeMismatch | BoundaryErrorKind::UnknownField
    )
}

fn json_pointer_overlaps(left: &str, right: &str) -> bool {
    left == right || pointer_contains(left, right) || pointer_contains(right, left)
}

fn pointer_contains(parent: &str, child: &str) -> bool {
    if parent.is_empty() {
        return true;
    }
    child
        .strip_prefix(parent)
        .is_some_and(|remainder| remainder.starts_with('/'))
}

struct TerminalMetadata {
    ambiguity_detected: bool,
    resource_ceiling_triggered: Option<String>,
    treatment_integrity_receipt: Option<TreatmentIntegrityReceiptV1>,
    parsed_digest: Option<DigestHex>,
    canonical_digest: Option<DigestHex>,
}

fn should_quarantine_unknown_fields(
    profile: &BoundaryCompilerProfileV1,
    errors: &[BoundaryErrorRecordV1],
) -> bool {
    profile.unknown_field_policy == UnknownFieldPolicy::Quarantine
        && !errors.is_empty()
        && errors
            .iter()
            .all(|error| error.kind == BoundaryErrorKind::UnknownField)
}

struct MinimalSchemaRules {
    allowed_top_level_fields: Option<BTreeSet<String>>,
    expected_field_types: BTreeMap<String, ExpectedJsonType>,
}

fn minimal_schema_rules(schema: &serde_json::Value) -> Option<MinimalSchemaRules> {
    let properties = schema.get("properties")?.as_object()?;
    let mut allowed_top_level_fields = BTreeSet::new();
    let mut expected_field_types = BTreeMap::new();

    for (field, spec) in properties {
        allowed_top_level_fields.insert(field.clone());
        if let Some(expected) = spec
            .get("type")
            .and_then(serde_json::Value::as_str)
            .and_then(expected_type_from_schema)
        {
            expected_field_types.insert(field.clone(), expected);
        }
    }

    Some(MinimalSchemaRules {
        allowed_top_level_fields: Some(allowed_top_level_fields),
        expected_field_types,
    })
}

fn expected_type_from_schema(schema_type: &str) -> Option<ExpectedJsonType> {
    match schema_type {
        "null" => Some(ExpectedJsonType::Null),
        "boolean" => Some(ExpectedJsonType::Bool),
        "number" | "integer" => Some(ExpectedJsonType::Number),
        "string" => Some(ExpectedJsonType::String),
        "array" => Some(ExpectedJsonType::Array),
        "object" => Some(ExpectedJsonType::Object),
        _ => None,
    }
}
