//! Boundary compiler primitives for AiDENs.
//!
//! LLM structured output, patch payloads, generated manifests, and repaired
//! JSON should pass through this crate before reaching tools or memory.
//!
//! Two compile paths are available:
//!
//! - `compile_json_boundary()` — the **lenient** path with repair, markdown
//!   fence stripping, JSON substring extraction, and treatment-critical field
//!   integrity checks.
//!
//! - `canonical_boundary::canonical_compile_json_boundary()` — the **strict**
//!   path via the canonical `boundary-compiler-core` microkernel. Use this
//!   when you need unforgiving RFC 8259 compliance with canonical digests.

mod canonical_boundary;

pub use boundary_compiler::{
    BoundaryProfile, Canonicalizer, ContentDigest as CanonicalContentDigest,
    JcsError as CanonicalJcsError,
};

pub use canonical_boundary::canonical_compile_json_boundary;

use aidens_contracts::{
    non_authoritative_json_display_digest, non_authoritative_text_display_digest, ArtifactId,
    BoundaryCompileOutcomeV1, BoundaryCompileRequestV1, BoundaryRepairReportV1,
    CanonicalBackpointerV1, DisplayDigestV1, DuplicateKeyFindingV1,
    JsonBoundaryRepairDisplayReportV1, SchemaValidationReportV1,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const MAX_JSON_BYTES: usize = 1_048_576;
const MAX_JSON_DEPTH: usize = 64;
const MAX_JSON_NODES: usize = 100_000;
const MAX_JSON_STRING_BYTES: usize = 262_144;
const MAX_JSON_ARRAY_ITEMS: usize = 50_000;
const MAX_JSON_OBJECT_KEYS: usize = 50_000;

#[derive(Debug, Error)]
pub enum BoundaryError {
    #[error("invalid json: {0}")]
    InvalidJson(String),
    #[error("duplicate json object keys: {0:?}")]
    DuplicateKeys(Vec<DuplicateKeyFindingV1>),
    #[error("json resource limit exceeded: {0}")]
    ResourceLimit(String),
}

pub fn parse_strict_json(input: &str) -> Result<Value, BoundaryError> {
    if input.len() > MAX_JSON_BYTES {
        return Err(BoundaryError::ResourceLimit(format!(
            "json-byte-length>{MAX_JSON_BYTES}"
        )));
    }
    let duplicate_key_findings = detect_duplicate_keys(input)?;
    if !duplicate_key_findings.is_empty() {
        return Err(BoundaryError::DuplicateKeys(duplicate_key_findings));
    }
    let value = serde_json::from_str(input)
        .map_err(|error| BoundaryError::InvalidJson(error.to_string()))?;
    check_value_resource_limits(&value)?;
    Ok(value)
}

/// Compute a canonical RFC 8785 JSON digest for any value.
pub fn canonical_digest(
    value: &serde_json::Value,
) -> Result<boundary_compiler::ContentDigest, boundary_compiler::JcsError> {
    boundary_compiler::ContentDigest::compute(value)
}

/// Canonicalize a JSON value per RFC 8785.
pub fn canonicalize_json(value: &serde_json::Value) -> Result<String, boundary_compiler::JcsError> {
    boundary_compiler::Canonicalizer::new().canonicalize(value)
}

/// Parse JSON with strict duplicate-key detection (using the canonical boundary compiler).
pub fn parse_strict_canonical(
    input: &str,
) -> Result<serde_json::Value, boundary_compiler::JcsError> {
    boundary_compiler::parse_with_dup_check(input)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoundaryRepairPolicyV1 {
    pub allow_markdown_fence_repair: bool,
    pub allow_json_substring_extract: bool,
}

impl BoundaryRepairPolicyV1 {
    pub fn permissive_degraded_repair() -> Self {
        Self {
            allow_markdown_fence_repair: true,
            allow_json_substring_extract: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryParseOutcomeV1 {
    pub value: Value,
    pub repair_receipt: BoundaryRepairReportV1,
    pub json_repair_receipt: Option<JsonBoundaryRepairDisplayReportV1>,
}

pub fn parse_json_boundary(
    input: &str,
    policy: BoundaryRepairPolicyV1,
) -> Result<BoundaryParseOutcomeV1, BoundaryError> {
    let mut request = BoundaryCompileRequestV1::new(input);
    request.allow_markdown_fence_repair = policy.allow_markdown_fence_repair;
    request.allow_json_substring_extract = policy.allow_json_substring_extract;

    let outcome = compile_json_boundary(request);
    if !outcome.accepted {
        if !outcome.duplicate_key_findings.is_empty() {
            return Err(BoundaryError::DuplicateKeys(outcome.duplicate_key_findings));
        }
        return Err(BoundaryError::InvalidJson(outcome.reason_codes.join(",")));
    }

    let value = outcome
        .value
        .ok_or_else(|| BoundaryError::InvalidJson("accepted outcome had no value".into()))?;
    let repair_receipt = outcome
        .repair_receipt
        .as_ref()
        .map(BoundaryRepairReportV1::from)
        .unwrap_or_else(no_repair_receipt);
    Ok(BoundaryParseOutcomeV1 {
        value,
        repair_receipt,
        json_repair_receipt: outcome.repair_receipt,
    })
}

pub fn compile_json_boundary(request: BoundaryCompileRequestV1) -> BoundaryCompileOutcomeV1 {
    match parse_strict_json(&request.input) {
        Ok(value) => finalize_compile_outcome(request, value, None),
        Err(BoundaryError::DuplicateKeys(findings)) => {
            duplicate_key_outcome(request.request_id, findings, None)
        }
        Err(BoundaryError::ResourceLimit(limit)) => BoundaryCompileOutcomeV1::rejected(
            request.request_id,
            vec!["boundary-resource-limit-exceeded".into(), limit],
        ),
        Err(original_error) => compile_with_repair(request, original_error),
    }
}

pub fn validate_json_schema(schema: &Value, input: &Value) -> SchemaValidationReportV1 {
    let mut errors = Vec::new();
    validate_schema_at("$", schema, schema, input, &mut errors, &mut Vec::new());
    SchemaValidationReportV1::new(Some(schema), input, errors)
}

pub fn validate_tool_input_schema(
    tool_id: &str,
    schema: &Value,
    input: &Value,
) -> SchemaValidationReportV1 {
    validate_json_schema(schema, input).with_tool_id(tool_id)
}

pub fn memory_claim_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["subject", "predicate", "object", "valid_from"],
        "properties": {
            "subject": { "type": "string" },
            "predicate": { "type": "string" },
            "object": {},
            "valid_from": { "type": "string" },
            "valid_to": { "type": "string" },
            "evidence": { "type": "object" },
            "recorded_at": { "type": "string" }
        }
    })
}

pub fn validate_memory_claim_input(input: &Value) -> SchemaValidationReportV1 {
    validate_json_schema(&memory_claim_input_schema(), input).with_tool_id("aidens:memory-write:1")
}

pub fn detect_duplicate_keys(input: &str) -> Result<Vec<DuplicateKeyFindingV1>, BoundaryError> {
    let mut scanner = JsonDuplicateScanner::new(input);
    scanner.scan()
}

fn compile_with_repair(
    request: BoundaryCompileRequestV1,
    original_error: BoundaryError,
) -> BoundaryCompileOutcomeV1 {
    if request.allow_markdown_fence_repair {
        if let Some(repaired) = strip_markdown_json_fence(&request.input) {
            return compile_repaired_candidate(
                request,
                "markdown-json-fence-stripped",
                repaired,
                vec!["removed markdown code fence before JSON boundary parse".into()],
            );
        }
    }

    if request.allow_json_substring_extract {
        if let Some(extracted) = extract_first_json_value(&request.input) {
            return compile_repaired_candidate(
                request,
                "json-substring-extracted",
                extracted,
                vec!["extracted first JSON object or array before boundary parse".into()],
            );
        }
    }

    BoundaryCompileOutcomeV1::rejected(
        request.request_id,
        vec![
            "invalid-json".into(),
            format!("boundary-parse-failed:{original_error}"),
        ],
    )
}

fn compile_repaired_candidate(
    request: BoundaryCompileRequestV1,
    repair_kind: &str,
    repaired: String,
    warnings: Vec<String>,
) -> BoundaryCompileOutcomeV1 {
    let original = request.input.clone();
    match parse_strict_json(&repaired) {
        Ok(value) => {
            let mut repair = json_repair_receipt(
                repair_kind,
                &original,
                &repaired,
                None,
                Some(&value),
                &request.treatment_critical_fields,
                warnings,
            );
            if !repair.treatment_integrity_warnings.is_empty()
                && request.hard_fail_on_treatment_change
            {
                repair.hard_failed = true;
                repair
                    .reason_codes
                    .push("treatment-integrity-hard-fail".into());
                let mut outcome = BoundaryCompileOutcomeV1::rejected(
                    request.request_id,
                    vec!["treatment-integrity-hard-fail".into()],
                );
                outcome.value = Some(value.clone());
                outcome.display_digest = Some(DisplayDigestV1::for_json_value(&value));
                outcome.repair_receipt = Some(repair);
                return outcome;
            }
            finalize_compile_outcome(request, value, Some(repair))
        }
        Err(BoundaryError::DuplicateKeys(findings)) => {
            let repair = json_repair_receipt(
                repair_kind,
                &original,
                &repaired,
                None,
                None,
                &request.treatment_critical_fields,
                warnings,
            );
            duplicate_key_outcome(request.request_id, findings, Some(repair))
        }
        Err(error) => {
            let mut outcome = BoundaryCompileOutcomeV1::rejected(
                request.request_id,
                vec![
                    "repair-produced-invalid-json".into(),
                    format!("boundary-parse-failed:{error}"),
                ],
            );
            outcome.repair_receipt = Some(json_repair_receipt(
                repair_kind,
                &original,
                &repaired,
                None,
                None,
                &request.treatment_critical_fields,
                warnings,
            ));
            outcome
        }
    }
}

fn finalize_compile_outcome(
    request: BoundaryCompileRequestV1,
    value: Value,
    repair_receipt: Option<JsonBoundaryRepairDisplayReportV1>,
) -> BoundaryCompileOutcomeV1 {
    let schema_validation = request
        .schema
        .as_ref()
        .map(|schema| validate_json_schema(schema, &value));
    let schema_valid = schema_validation
        .as_ref()
        .map(|receipt| receipt.valid)
        .unwrap_or(true);
    let mut reason_codes = Vec::new();
    if schema_valid {
        reason_codes.push("boundary-compile-accepted".into());
    } else {
        reason_codes.push("schema-validation-failed".into());
    }
    if let Some(repair) = &repair_receipt {
        reason_codes.push(format!("boundary-repair:{}", repair.repair_kind));
        reason_codes.extend(repair.reason_codes.iter().cloned());
    }
    reason_codes.sort();
    reason_codes.dedup();

    BoundaryCompileOutcomeV1 {
        outcome_id: ArtifactId::new("boundary-compile-outcome"),
        request_id: request.request_id,
        accepted: schema_valid,
        degraded: !schema_valid
            || repair_receipt
                .as_ref()
                .map(|receipt| receipt.degraded)
                .unwrap_or(false),
        value: Some(value.clone()),
        display_digest: Some(DisplayDigestV1::for_json_value(&value)),
        duplicate_key_findings: Vec::new(),
        schema_validation,
        repair_receipt,
        reason_codes,
        compiled_at: chrono::Utc::now(),
    }
}

fn duplicate_key_outcome(
    request_id: ArtifactId,
    findings: Vec<DuplicateKeyFindingV1>,
    repair_receipt: Option<JsonBoundaryRepairDisplayReportV1>,
) -> BoundaryCompileOutcomeV1 {
    BoundaryCompileOutcomeV1 {
        outcome_id: ArtifactId::new("boundary-compile-outcome"),
        request_id,
        accepted: false,
        degraded: true,
        value: None,
        display_digest: None,
        duplicate_key_findings: findings,
        schema_validation: None,
        repair_receipt,
        reason_codes: vec!["duplicate-json-object-key".into()],
        compiled_at: chrono::Utc::now(),
    }
}

fn check_value_resource_limits(value: &Value) -> Result<(), BoundaryError> {
    let mut stats = ValueResourceStats::default();
    collect_value_resource_stats(value, 0, &mut stats)?;
    if stats.nodes > MAX_JSON_NODES {
        return Err(BoundaryError::ResourceLimit(format!(
            "json-node-count>{MAX_JSON_NODES}"
        )));
    }
    Ok(())
}

#[derive(Default)]
struct ValueResourceStats {
    nodes: usize,
}

fn collect_value_resource_stats(
    value: &Value,
    depth: usize,
    stats: &mut ValueResourceStats,
) -> Result<(), BoundaryError> {
    if depth > MAX_JSON_DEPTH {
        return Err(BoundaryError::ResourceLimit(format!(
            "json-depth>{MAX_JSON_DEPTH}"
        )));
    }
    stats.nodes += 1;
    match value {
        Value::Array(items) => {
            if items.len() > MAX_JSON_ARRAY_ITEMS {
                return Err(BoundaryError::ResourceLimit(format!(
                    "json-array-items>{MAX_JSON_ARRAY_ITEMS}"
                )));
            }
            for item in items {
                collect_value_resource_stats(item, depth + 1, stats)?;
            }
        }
        Value::Object(map) => {
            if map.len() > MAX_JSON_OBJECT_KEYS {
                return Err(BoundaryError::ResourceLimit(format!(
                    "json-object-keys>{MAX_JSON_OBJECT_KEYS}"
                )));
            }
            for item in map.values() {
                collect_value_resource_stats(item, depth + 1, stats)?;
            }
        }
        Value::String(text) => {
            if text.len() > MAX_JSON_STRING_BYTES {
                return Err(BoundaryError::ResourceLimit(format!(
                    "json-string-bytes>{MAX_JSON_STRING_BYTES}"
                )));
            }
        }
        Value::Bool(_) | Value::Null | Value::Number(_) => {}
    }
    Ok(())
}

fn json_repair_receipt(
    repair_kind: &str,
    before: &str,
    after: &str,
    before_value: Option<&Value>,
    after_value: Option<&Value>,
    treatment_critical_fields: &[String],
    warnings: Vec<String>,
) -> JsonBoundaryRepairDisplayReportV1 {
    let treatment_integrity_warnings =
        treatment_integrity_warnings(after_value, treatment_critical_fields, repair_kind);
    let mut reason_codes = vec![format!("json-repair:{repair_kind}")];
    if !treatment_integrity_warnings.is_empty() {
        reason_codes.push("treatment-integrity-unverifiable".into());
    }
    JsonBoundaryRepairDisplayReportV1 {
        receipt_id: ArtifactId::new("json-repair"),
        kind: aidens_contracts::ArtifactKindV1::BoundaryRepair,
        changed: before != after,
        repair_kind: repair_kind.into(),
        degraded: before != after,
        before_raw_digest: Some(non_authoritative_text_display_digest(before)),
        after_raw_digest: Some(non_authoritative_text_display_digest(after)),
        before_display_digest: before_value.map(DisplayDigestV1::for_json_value),
        after_display_digest: after_value.map(DisplayDigestV1::for_json_value),
        treatment_critical_fields: treatment_critical_fields.to_vec(),
        treatment_integrity_warnings,
        hard_failed: false,
        warnings,
        reason_codes,
        canonical_repair_record_ids: Vec::new(),
        canonical_backpointers: vec![CanonicalBackpointerV1::owner_type(
            "verification-control",
            "BoundaryRepairRecord",
            "canonical-boundary-repair-owner",
        )],
    }
}

fn treatment_integrity_warnings(
    after_value: Option<&Value>,
    treatment_critical_fields: &[String],
    repair_kind: &str,
) -> Vec<String> {
    let Some(after_value) = after_value else {
        return Vec::new();
    };
    treatment_critical_fields
        .iter()
        .filter(|field| lookup_treatment_field(after_value, field).is_some())
        .map(|field| format!("treatment-integrity-unverifiable:{repair_kind}:{field}"))
        .collect()
}

fn lookup_treatment_field<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    if field.starts_with('/') {
        return value.pointer(field);
    }
    let mut current = value;
    for segment in field.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn validate_schema_at(
    path: &str,
    root_schema: &Value,
    schema: &Value,
    input: &Value,
    errors: &mut Vec<String>,
    ref_stack: &mut Vec<String>,
) {
    let Some(schema_object) = schema.as_object() else {
        errors.push(format!("{path}: schema must be a JSON object"));
        return;
    };
    for keyword in schema_object.keys() {
        if !SUPPORTED_SCHEMA_KEYWORDS.contains(&keyword.as_str()) {
            errors.push(format!("{path}: unsupported schema keyword '{keyword}'"));
        }
    }

    if let Some(reference) = schema_object.get("$ref") {
        let Some(reference) = reference.as_str() else {
            errors.push(format!("{path}: $ref must be a string"));
            return;
        };
        if ref_stack.iter().any(|seen| seen == reference) {
            errors.push(format!(
                "{path}: recursive $ref '{reference}' is unsupported"
            ));
            return;
        }
        let Some(resolved) = resolve_local_schema_ref(root_schema, reference) else {
            errors.push(format!(
                "{path}: unsupported or unresolved $ref '{reference}'"
            ));
            return;
        };
        ref_stack.push(reference.to_string());
        validate_schema_at(path, root_schema, resolved, input, errors, ref_stack);
        ref_stack.pop();
    }

    if let Some(any_of) = schema_object.get("anyOf") {
        validate_any_of(path, root_schema, any_of, input, errors, ref_stack);
    }

    if let Some(one_of) = schema_object.get("oneOf") {
        validate_one_of(path, root_schema, one_of, input, errors, ref_stack);
    }

    if let Some(all_of) = schema_object.get("allOf") {
        validate_all_of(path, root_schema, all_of, input, errors, ref_stack);
    }

    if let Some(type_value) = schema_object.get("type") {
        let expected = expected_types(type_value);
        if expected.is_empty() {
            errors.push(format!(
                "{path}: schema type must be string or array of strings"
            ));
        } else if !expected
            .iter()
            .any(|expected| value_matches_type(input, expected))
        {
            errors.push(format!(
                "{path}: expected type {}, got {}",
                expected.join("|"),
                value_type(input)
            ));
            return;
        }
    }

    if let Some(enum_values) = schema_object.get("enum") {
        let Some(items) = enum_values.as_array() else {
            errors.push(format!("{path}: enum must be an array"));
            return;
        };
        if !items.iter().any(|item| item == input) {
            errors.push(format!("{path}: value is not in enum"));
        }
    }

    if let Some(format_value) = schema_object.get("format") {
        validate_format(path, format_value, input, errors);
    }

    if let Some(minimum) = schema_object.get("minimum") {
        let Some(minimum) = minimum.as_f64() else {
            errors.push(format!("{path}: minimum must be numeric"));
            return;
        };
        if input.as_f64().is_some_and(|value| value < minimum) {
            errors.push(format!("{path}: value is below minimum {minimum}"));
        }
    }

    if let Some(maximum) = schema_object.get("maximum") {
        let Some(maximum) = maximum.as_f64() else {
            errors.push(format!("{path}: maximum must be numeric"));
            return;
        };
        if input.as_f64().is_some_and(|value| value > maximum) {
            errors.push(format!("{path}: value is above maximum {maximum}"));
        }
    }

    if let Some(expected) = schema_object.get("const") {
        if input != expected {
            errors.push(format!("{path}: value does not match const"));
        }
    }

    if let Some(min_length) = schema_object.get("minLength") {
        let Some(min_length) = min_length.as_u64() else {
            errors.push(format!("{path}: minLength must be an integer"));
            return;
        };
        if input
            .as_str()
            .is_some_and(|value| value.chars().count() < min_length as usize)
        {
            errors.push(format!(
                "{path}: string length is below minLength {min_length}"
            ));
        }
    }

    if let Some(max_length) = schema_object.get("maxLength") {
        let Some(max_length) = max_length.as_u64() else {
            errors.push(format!("{path}: maxLength must be an integer"));
            return;
        };
        if input
            .as_str()
            .is_some_and(|value| value.chars().count() > max_length as usize)
        {
            errors.push(format!(
                "{path}: string length is above maxLength {max_length}"
            ));
        }
    }

    if let Some(input_array) = input.as_array() {
        if let Some(min_items) = schema_object.get("minItems") {
            let Some(min_items) = min_items.as_u64() else {
                errors.push(format!("{path}: minItems must be an integer"));
                return;
            };
            if input_array.len() < min_items as usize {
                errors.push(format!(
                    "{path}: array length is below minItems {min_items}"
                ));
            }
        }
        if let Some(max_items) = schema_object.get("maxItems") {
            let Some(max_items) = max_items.as_u64() else {
                errors.push(format!("{path}: maxItems must be an integer"));
                return;
            };
            if input_array.len() > max_items as usize {
                errors.push(format!(
                    "{path}: array length is above maxItems {max_items}"
                ));
            }
        }
        if let Some(item_schema) = schema_object.get("items") {
            for (index, item) in input_array.iter().enumerate() {
                validate_schema_at(
                    &format!("{path}[{index}]"),
                    root_schema,
                    item_schema,
                    item,
                    errors,
                    ref_stack,
                );
            }
        }
    }

    let Some(input_object) = input.as_object() else {
        return;
    };

    if let Some(required) = schema_object.get("required") {
        match required.as_array() {
            Some(required) => {
                for field in required {
                    let Some(field) = field.as_str() else {
                        errors.push(format!("{path}: required entries must be strings"));
                        continue;
                    };
                    if !input_object.contains_key(field) {
                        errors.push(format!("{path}: missing required property '{field}'"));
                    }
                }
            }
            None => errors.push(format!("{path}: required must be an array")),
        }
    }

    let properties = match schema_object.get("properties") {
        Some(properties) => match properties.as_object() {
            Some(properties) => properties.clone(),
            None => {
                errors.push(format!("{path}: properties must be an object"));
                serde_json::Map::new()
            }
        },
        None => serde_json::Map::new(),
    };

    for (field, child_schema) in &properties {
        if let Some(child) = input_object.get(field) {
            validate_schema_at(
                &object_child_path(path, field),
                root_schema,
                child_schema,
                child,
                errors,
                ref_stack,
            );
        }
    }

    if let Some(additional_properties) = schema_object.get("additionalProperties") {
        let allowed = properties.keys().cloned().collect::<BTreeSet<_>>();
        match additional_properties {
            Value::Bool(false) => {
                for field in input_object.keys() {
                    if !allowed.contains(field) {
                        errors.push(format!(
                            "{path}: additional property '{field}' is not allowed"
                        ));
                    }
                }
            }
            Value::Bool(true) => {}
            Value::Object(_) => {
                for (field, child) in input_object {
                    if !allowed.contains(field) {
                        validate_schema_at(
                            &object_child_path(path, field),
                            root_schema,
                            additional_properties,
                            child,
                            errors,
                            ref_stack,
                        );
                    }
                }
            }
            _ => errors.push(format!(
                "{path}: additionalProperties must be boolean or object"
            )),
        }
    }
}

const SUPPORTED_SCHEMA_KEYWORDS: &[&str] = &[
    "$defs",
    "$ref",
    "$schema",
    "additionalProperties",
    "allOf",
    "anyOf",
    "const",
    "default",
    "definitions",
    "description",
    "enum",
    "format",
    "items",
    "maximum",
    "maxItems",
    "maxLength",
    "minimum",
    "minItems",
    "minLength",
    "properties",
    "required",
    "title",
    "type",
    "oneOf",
];

fn resolve_local_schema_ref<'a>(root_schema: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix('#')?;
    if pointer.is_empty() {
        return Some(root_schema);
    }
    if !pointer.starts_with('/') {
        return None;
    }
    root_schema.pointer(pointer)
}

fn validate_any_of(
    path: &str,
    root_schema: &Value,
    schemas: &Value,
    input: &Value,
    errors: &mut Vec<String>,
    ref_stack: &[String],
) {
    let Some(schemas) = schemas.as_array() else {
        errors.push(format!("{path}: anyOf must be an array"));
        return;
    };
    if schemas.is_empty() {
        errors.push(format!("{path}: anyOf must not be empty"));
        return;
    }
    let mut matched = false;
    for schema in schemas {
        let mut branch_errors = Vec::new();
        let mut branch_ref_stack = ref_stack.to_vec();
        validate_schema_at(
            path,
            root_schema,
            schema,
            input,
            &mut branch_errors,
            &mut branch_ref_stack,
        );
        if branch_errors.is_empty() {
            matched = true;
            break;
        }
    }
    if !matched {
        errors.push(format!("{path}: value does not match anyOf"));
    }
}

fn validate_one_of(
    path: &str,
    root_schema: &Value,
    schemas: &Value,
    input: &Value,
    errors: &mut Vec<String>,
    ref_stack: &[String],
) {
    let Some(schemas) = schemas.as_array() else {
        errors.push(format!("{path}: oneOf must be an array"));
        return;
    };
    if schemas.is_empty() {
        errors.push(format!("{path}: oneOf must not be empty"));
        return;
    }
    let mut matches = 0usize;
    for schema in schemas {
        let mut branch_errors = Vec::new();
        let mut branch_ref_stack = ref_stack.to_vec();
        validate_schema_at(
            path,
            root_schema,
            schema,
            input,
            &mut branch_errors,
            &mut branch_ref_stack,
        );
        if branch_errors.is_empty() {
            matches += 1;
        }
    }
    if matches != 1 {
        errors.push(format!(
            "{path}: value matched {matches} oneOf branches, expected exactly 1"
        ));
    }
}

fn validate_all_of(
    path: &str,
    root_schema: &Value,
    schemas: &Value,
    input: &Value,
    errors: &mut Vec<String>,
    ref_stack: &mut Vec<String>,
) {
    let Some(schemas) = schemas.as_array() else {
        errors.push(format!("{path}: allOf must be an array"));
        return;
    };
    if schemas.is_empty() {
        errors.push(format!("{path}: allOf must not be empty"));
        return;
    }
    for schema in schemas {
        validate_schema_at(path, root_schema, schema, input, errors, ref_stack);
    }
}

fn validate_format(path: &str, format_value: &Value, input: &Value, errors: &mut Vec<String>) {
    let Some(format) = format_value.as_str() else {
        errors.push(format!("{path}: format must be a string"));
        return;
    };
    match format {
        "date-time" => {
            if let Some(text) = input.as_str() {
                if chrono::DateTime::parse_from_rfc3339(text).is_err() {
                    errors.push(format!("{path}: string is not valid date-time"));
                }
            }
        }
        "uint" | "uint32" | "uint64" => {
            if let Some(value) = input.as_i64() {
                if value < 0 {
                    errors.push(format!("{path}: integer is negative for {format}"));
                }
            }
            if format == "uint32" && input.as_u64().is_some_and(|value| value > u32::MAX as u64) {
                errors.push(format!("{path}: integer is above uint32"));
            }
        }
        "int32" => {
            if input
                .as_i64()
                .is_some_and(|value| value < i32::MIN as i64 || value > i32::MAX as i64)
            {
                errors.push(format!("{path}: integer is outside int32"));
            }
        }
        "int64" => {}
        other => errors.push(format!("{path}: unsupported schema format '{other}'")),
    }
}

fn expected_types(type_value: &Value) -> Vec<String> {
    if let Some(type_name) = type_value.as_str() {
        return vec![type_name.into()];
    }
    type_value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn value_matches_type(value: &Value, expected: &str) -> bool {
    match expected {
        "array" => value.is_array(),
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "object" => value.is_object(),
        "string" => value.is_string(),
        _ => false,
    }
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "array",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::Object(_) => "object",
        Value::String(_) => "string",
    }
}

fn strip_markdown_json_fence(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let body = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))?;
    let body = body.strip_suffix("```")?;
    Some(body.trim().to_string())
}

fn extract_first_json_value(input: &str) -> Option<String> {
    let mut starts = [
        input.find('{').map(|idx| (idx, '{', '}')),
        input.find('[').map(|idx| (idx, '[', ']')),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    starts.sort_by_key(|(idx, _, _)| *idx);
    for (start, open, close) in starts {
        let Some(end) = find_matching_delimiter(&input[start..], open, close) else {
            continue;
        };
        return Some(input[start..=start + end].trim().to_string());
    }
    None
}

fn find_matching_delimiter(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn non_authoritative_boundary_json_display_digest(value: &Value) -> String {
    non_authoritative_json_display_digest(value)
}

fn text_digest(text: &str) -> String {
    non_authoritative_text_display_digest(text)
}

pub fn no_repair_receipt() -> BoundaryRepairReportV1 {
    BoundaryRepairReportV1 {
        receipt_id: ArtifactId::new("boundary-repair"),
        changed: false,
        repair_kind: "none".into(),
        before_digest: None,
        after_digest: None,
        canonical_repair_record_ids: Vec::new(),
        canonical_backpointers: vec![CanonicalBackpointerV1::owner_type(
            "verification-control",
            "BoundaryRepairRecord",
            "canonical-boundary-repair-owner",
        )],
        warnings: Vec::new(),
    }
}

pub fn repair_receipt(
    repair_kind: impl Into<String>,
    before: &str,
    after: &str,
    warnings: Vec<String>,
) -> BoundaryRepairReportV1 {
    BoundaryRepairReportV1 {
        receipt_id: ArtifactId::new("boundary-repair"),
        changed: before != after,
        repair_kind: repair_kind.into(),
        before_digest: Some(text_digest(before)),
        after_digest: Some(text_digest(after)),
        canonical_repair_record_ids: Vec::new(),
        canonical_backpointers: vec![CanonicalBackpointerV1::owner_type(
            "verification-control",
            "BoundaryRepairRecord",
            "canonical-boundary-repair-owner",
        )],
        warnings,
    }
}

struct JsonDuplicateScanner<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    node_count: usize,
    findings: Vec<DuplicateKeyFindingV1>,
}

impl<'a> JsonDuplicateScanner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            node_count: 0,
            findings: Vec::new(),
        }
    }

    fn scan(&mut self) -> Result<Vec<DuplicateKeyFindingV1>, BoundaryError> {
        self.skip_ws();
        self.parse_value("$", 0)?;
        self.skip_ws();
        if self.pos != self.bytes.len() {
            return Err(self.invalid("trailing bytes after JSON value"));
        }
        Ok(std::mem::take(&mut self.findings))
    }

    fn parse_value(&mut self, path: &str, depth: usize) -> Result<(), BoundaryError> {
        if depth > MAX_JSON_DEPTH {
            return Err(BoundaryError::ResourceLimit(format!(
                "json-depth>{MAX_JSON_DEPTH}"
            )));
        }
        self.node_count += 1;
        if self.node_count > MAX_JSON_NODES {
            return Err(BoundaryError::ResourceLimit(format!(
                "json-node-count>{MAX_JSON_NODES}"
            )));
        }
        self.skip_ws();
        let Some(byte) = self.peek() else {
            return Err(self.invalid("unexpected end of input"));
        };
        match byte {
            b'{' => self.parse_object(path, depth),
            b'[' => self.parse_array(path, depth),
            b'"' => self.parse_string().map(|_| ()),
            b't' => self.parse_literal(b"true"),
            b'f' => self.parse_literal(b"false"),
            b'n' => self.parse_literal(b"null"),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(self.invalid("unexpected byte while parsing value")),
        }
    }

    fn parse_object(&mut self, path: &str, depth: usize) -> Result<(), BoundaryError> {
        self.expect(b'{')?;
        self.skip_ws();
        if self.consume(b'}') {
            return Ok(());
        }

        let mut keys = BTreeMap::<String, usize>::new();
        let mut key_count = 0usize;
        loop {
            self.skip_ws();
            let (key, key_offset) = self.parse_string()?;
            key_count += 1;
            if key_count > MAX_JSON_OBJECT_KEYS {
                return Err(BoundaryError::ResourceLimit(format!(
                    "json-object-keys>{MAX_JSON_OBJECT_KEYS}"
                )));
            }
            if let Some(first_offset) = keys.insert(key.clone(), key_offset) {
                self.findings.push(DuplicateKeyFindingV1::new(
                    path,
                    key.clone(),
                    Some(first_offset),
                    Some(key_offset),
                ));
            }
            self.skip_ws();
            self.expect(b':')?;
            let child_path = object_child_path(path, &key);
            self.parse_value(&child_path, depth + 1)?;
            self.skip_ws();
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(())
    }

    fn parse_array(&mut self, path: &str, depth: usize) -> Result<(), BoundaryError> {
        self.expect(b'[')?;
        self.skip_ws();
        if self.consume(b']') {
            return Ok(());
        }

        let mut index = 0usize;
        loop {
            if index >= MAX_JSON_ARRAY_ITEMS {
                return Err(BoundaryError::ResourceLimit(format!(
                    "json-array-items>{MAX_JSON_ARRAY_ITEMS}"
                )));
            }
            self.parse_value(&format!("{path}[{index}]"), depth + 1)?;
            index += 1;
            self.skip_ws();
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<(String, usize), BoundaryError> {
        let start = self.pos;
        self.expect(b'"')?;
        while let Some(byte) = self.peek() {
            match byte {
                b'"' => {
                    self.pos += 1;
                    let raw = &self.input[start..self.pos];
                    if raw.len() > MAX_JSON_STRING_BYTES {
                        return Err(BoundaryError::ResourceLimit(format!(
                            "json-string-bytes>{MAX_JSON_STRING_BYTES}"
                        )));
                    }
                    let parsed = serde_json::from_str::<String>(raw)
                        .map_err(|error| self.invalid(error.to_string()))?;
                    return Ok((parsed, start));
                }
                b'\\' => {
                    self.pos += 1;
                    let escaped = self
                        .peek()
                        .ok_or_else(|| self.invalid("unterminated string escape"))?;
                    if escaped == b'u' {
                        self.pos += 1;
                        for _ in 0..4 {
                            let Some(hex) = self.peek() else {
                                return Err(self.invalid("unterminated unicode escape"));
                            };
                            if !hex.is_ascii_hexdigit() {
                                return Err(self.invalid("invalid unicode escape"));
                            }
                            self.pos += 1;
                        }
                    } else {
                        if !matches!(
                            escaped,
                            b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't'
                        ) {
                            return Err(self.invalid("invalid string escape"));
                        }
                        self.pos += 1;
                    }
                }
                0x00..=0x1f => return Err(self.invalid("control byte in string")),
                _ => self.pos += 1,
            }
        }
        Err(self.invalid("unterminated string"))
    }

    fn parse_number(&mut self) -> Result<(), BoundaryError> {
        let start = self.pos;
        if self.consume(b'-') && self.peek().is_none() {
            return Err(self.invalid("invalid number"));
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.invalid("invalid number")),
        }
        if self.consume(b'.') {
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                return Err(self.invalid("invalid fractional number"));
            }
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if self.peek().is_some_and(|byte| matches!(byte, b'e' | b'E')) {
            self.pos += 1;
            if self.peek().is_some_and(|byte| matches!(byte, b'+' | b'-')) {
                self.pos += 1;
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                return Err(self.invalid("invalid exponent"));
            }
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
        }

        let raw = &self.input[start..self.pos];
        let value = serde_json::from_str::<Value>(raw)
            .map_err(|error| self.invalid(format!("invalid number: {error}")))?;
        if !value.is_number() {
            return Err(self.invalid("invalid number"));
        }
        Ok(())
    }

    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), BoundaryError> {
        if self.bytes[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.invalid("invalid literal"))
        }
    }

    fn skip_ws(&mut self) {
        while self
            .peek()
            .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn expect(&mut self, expected: u8) -> Result<(), BoundaryError> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(self.invalid(format!("expected byte '{}'", expected as char)))
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn invalid(&self, message: impl Into<String>) -> BoundaryError {
        BoundaryError::InvalidJson(format!("{} at byte {}", message.into(), self.pos))
    }
}

fn object_child_path(parent: &str, key: &str) -> String {
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        format!("{parent}.{key}")
    } else {
        format!(
            "{parent}[{}]",
            serde_json::to_string(key).unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_json_rejects_invalid_input() {
        assert!(parse_strict_json("{not json}").is_err());
    }

    #[test]
    fn strict_json_accepts_valid_input() {
        assert!(parse_strict_json(r#"{"ok":true}"#).is_ok());
    }

    #[test]
    fn canonicalize_json_sorts_object_keys() {
        let value = serde_json::json!({"b": 2, "a": 1});

        assert_eq!(
            canonicalize_json(&value).expect("canonicalization should succeed"),
            r#"{"a":1,"b":2}"#
        );
    }

    #[test]
    fn canonical_digest_is_64_hex_characters() {
        let value = serde_json::json!({"b": 2, "a": 1});
        let digest = canonical_digest(&value).expect("digest computation should succeed");

        let hex = digest.hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn parse_strict_canonical_rejects_duplicate_keys() {
        let err = parse_strict_canonical(r#"{"a":1,"a":2}"#).unwrap_err();
        assert!(matches!(err, CanonicalJcsError::DuplicateKey { .. }));
    }

    #[test]
    fn strict_json_rejects_duplicate_object_keys() {
        let outcome = compile_json_boundary(BoundaryCompileRequestV1::new(
            r#"{"treatment":"control","treatment":"variant"}"#,
        ));

        assert!(!outcome.accepted);
        assert_eq!(outcome.duplicate_key_findings.len(), 1);
        assert_eq!(outcome.duplicate_key_findings[0].key, "treatment");
        assert!(outcome
            .reason_codes
            .contains(&"duplicate-json-object-key".into()));
    }

    #[test]
    fn markdown_fence_repair_is_receipt_bearing() {
        let outcome = parse_json_boundary(
            "```json\n{\"ok\":true}\n```",
            BoundaryRepairPolicyV1::permissive_degraded_repair(),
        )
        .expect("fenced json should repair");

        assert_eq!(outcome.value["ok"], true);
        assert!(outcome.repair_receipt.changed);
        assert_eq!(
            outcome.repair_receipt.repair_kind,
            "markdown-json-fence-stripped"
        );
        assert!(outcome.repair_receipt.before_digest.is_some());
        assert!(outcome.repair_receipt.after_digest.is_some());
        assert!(outcome
            .json_repair_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.degraded));
    }

    #[test]
    fn json_substring_extract_is_receipt_bearing() {
        let outcome = parse_json_boundary(
            "model said: {\"ok\":true,\"text\":\"brace } inside\"} done",
            BoundaryRepairPolicyV1::permissive_degraded_repair(),
        )
        .expect("embedded json should extract");

        assert_eq!(outcome.value["ok"], true);
        assert!(outcome.repair_receipt.changed);
        assert_eq!(
            outcome.repair_receipt.repair_kind,
            "json-substring-extracted"
        );
        assert!(outcome
            .json_repair_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.degraded));
    }

    #[test]
    fn default_boundary_policy_rejects_repairable_wrappers() {
        let fenced = parse_json_boundary(
            "```json\n{\"ok\":true}\n```",
            BoundaryRepairPolicyV1::default(),
        );
        let embedded = compile_json_boundary(BoundaryCompileRequestV1::new(
            "model said: {\"ok\":true} done",
        ));

        assert!(fenced.is_err());
        assert!(!embedded.accepted);
        assert!(embedded.repair_receipt.is_none());
        assert!(embedded
            .reason_codes
            .iter()
            .any(|reason| reason.starts_with("boundary-parse-failed")));
    }

    #[test]
    fn unsupported_schema_keyword_fails_closed() {
        let schema = serde_json::json!({
            "type": "string",
            "pattern": "^a"
        });
        let receipt = validate_json_schema(&schema, &serde_json::json!("abc"));

        assert!(!receipt.valid);
        assert!(receipt
            .errors
            .iter()
            .any(|error| error.contains("unsupported schema keyword 'pattern'")));
        assert!(receipt
            .reason_codes
            .contains(&"schema-validation-failed".into()));
    }

    #[test]
    fn supported_schema_array_and_additional_property_object_are_enforced() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "array",
                    "maxItems": 2,
                    "items": { "type": "string", "maxLength": 3 }
                }
            },
            "additionalProperties": { "type": "integer", "minimum": 1 }
        });
        let invalid = validate_json_schema(
            &schema,
            &serde_json::json!({
                "tags": ["ok", "toolong", "extra"],
                "count": 0
            }),
        );

        assert!(!invalid.valid);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("array length is above maxItems 2")));
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("string length is above maxLength 3")));
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("value is below minimum 1")));
    }

    #[test]
    fn local_schema_ref_and_format_are_semantic_not_silent() {
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "definitions": {
                "stamp": { "type": "string", "format": "date-time" }
            },
            "type": "object",
            "required": ["created_at"],
            "properties": {
                "created_at": { "$ref": "#/definitions/stamp" }
            },
            "additionalProperties": false
        });
        let invalid =
            validate_json_schema(&schema, &serde_json::json!({"created_at": "not-a-date"}));
        let valid = validate_json_schema(
            &schema,
            &serde_json::json!({"created_at": "2026-05-07T00:00:00Z"}),
        );

        assert!(!invalid.valid);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("not valid date-time")));
        assert!(valid.valid, "{:?}", valid.errors);
    }

    #[test]
    fn schema_invalid_input_is_blocked_by_compile_outcome() {
        let schema = serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["path"],
            "properties": {
                "path": { "type": "string" }
            }
        });
        let outcome = compile_json_boundary(
            BoundaryCompileRequestV1::new(r#"{"path":7}"#).with_schema(schema),
        );

        assert!(!outcome.accepted);
        let receipt = outcome
            .schema_validation
            .expect("schema validation receipt");
        assert_eq!(
            receipt.kind,
            aidens_contracts::ArtifactKindV1::SchemaValidation
        );
        assert!(!receipt.valid);
        assert!(receipt
            .reason_codes
            .contains(&"schema-validation-failed".into()));
    }

    #[test]
    fn memory_claim_input_validation_blocks_missing_bitemporal_fields() {
        let receipt = validate_memory_claim_input(&serde_json::json!({
            "subject": "repo",
            "predicate": "status",
            "object": "partial"
        }));

        assert!(!receipt.valid);
        assert_eq!(receipt.tool_id.as_deref(), Some("aidens:memory-write:1"));
        assert!(receipt
            .errors
            .iter()
            .any(|error| error.contains("valid_from")));
    }

    #[test]
    fn display_digest_is_stable_across_key_order_and_whitespace() {
        let left: Value = serde_json::from_str(r#"{"b":2,"a":{"z":false,"n":1}}"#).unwrap();
        let right: Value = serde_json::from_str(
            r#"{
              "a": { "n": 1, "z": false },
              "b": 2
            }"#,
        )
        .unwrap();

        assert_eq!(
            non_authoritative_boundary_json_display_digest(&left),
            non_authoritative_boundary_json_display_digest(&right)
        );
        assert!(non_authoritative_boundary_json_display_digest(&left).starts_with("blake3:"));
        assert_eq!(
            DisplayDigestV1::for_json_value(&left),
            DisplayDigestV1::for_json_value(&right)
        );
    }

    #[test]
    fn repair_warns_when_treatment_critical_fields_are_unverifiable() {
        let mut request =
            BoundaryCompileRequestV1::new("prefix {\"treatment\":\"variant\"} suffix")
                .with_treatment_critical_fields(vec!["treatment".into()]);
        request.allow_json_substring_extract = true;
        let outcome = compile_json_boundary(request);

        assert!(outcome.accepted);
        let repair = outcome.repair_receipt.expect("repair receipt");
        assert!(!repair.treatment_integrity_warnings.is_empty());
        assert!(repair
            .reason_codes
            .contains(&"treatment-integrity-unverifiable".into()));
    }

    #[test]
    fn treatment_critical_json_pointer_paths_are_honored() {
        let mut request = BoundaryCompileRequestV1::new(
            "prefix {\"doses\":[{\"mg\":10}],\"a/b\":{\"~key\":\"x\"}} suffix",
        )
        .with_treatment_critical_fields(vec!["/doses/0/mg".into(), "/a~1b/~0key".into()])
        .with_hard_fail_on_treatment_change(true);
        request.allow_json_substring_extract = true;
        let outcome = compile_json_boundary(request);

        assert!(!outcome.accepted);
        let repair = outcome.repair_receipt.expect("repair receipt");
        assert!(repair.hard_failed);
        assert!(repair
            .treatment_integrity_warnings
            .iter()
            .any(|warning| warning.ends_with(":/doses/0/mg")));
        assert!(repair
            .treatment_integrity_warnings
            .iter()
            .any(|warning| warning.ends_with(":/a~1b/~0key")));
    }

    #[test]
    fn duplicate_key_scanner_handles_unicode_and_nested_objects() {
        let findings = detect_duplicate_keys(
            r#"{"\u0061":1,"a":2,"nested":{"x":1,"x":2},"arr":[{"k":1,"k":2}]}"#,
        )
        .unwrap();

        assert_eq!(findings.len(), 3);
        assert!(findings
            .iter()
            .any(|finding| finding.path == "$" && finding.key == "a"));
        assert!(findings
            .iter()
            .any(|finding| finding.path == "$.nested" && finding.key == "x"));
        assert!(findings
            .iter()
            .any(|finding| finding.path == "$.arr[0]" && finding.key == "k"));
    }

    #[test]
    fn resource_ceiling_rejects_excessive_depth_before_acceptance() {
        let mut input = String::new();
        for _ in 0..=MAX_JSON_DEPTH {
            input.push('[');
        }
        input.push_str("true");
        for _ in 0..=MAX_JSON_DEPTH {
            input.push(']');
        }
        let outcome = compile_json_boundary(BoundaryCompileRequestV1::new(input));

        assert!(!outcome.accepted);
        assert!(outcome
            .reason_codes
            .contains(&"boundary-resource-limit-exceeded".into()));
        assert!(outcome
            .reason_codes
            .iter()
            .any(|reason| reason == &format!("json-depth>{MAX_JSON_DEPTH}")));
    }

    #[test]
    fn treatment_integrity_warning_can_hard_fail_repair() {
        let mut request =
            BoundaryCompileRequestV1::new("prefix {\"treatment\":\"variant\"} suffix")
                .with_treatment_critical_fields(vec!["treatment".into()])
                .with_hard_fail_on_treatment_change(true);
        request.allow_json_substring_extract = true;
        let outcome = compile_json_boundary(request);

        assert!(!outcome.accepted);
        assert!(outcome
            .repair_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.hard_failed));
        assert!(outcome
            .reason_codes
            .contains(&"treatment-integrity-hard-fail".into()));
    }

    #[test]
    fn p28_boundary_compile_receipt_rejects_duplicate_keys() {
        let profile = aidens_contracts::BoundaryCompilerProfileV1::strict_json("schema:test");
        let outcome =
            compile_json_boundary(BoundaryCompileRequestV1::new(r#"{"path":"a","path":"b"}"#));
        assert!(!outcome.accepted);
        assert_eq!(outcome.duplicate_key_findings.len(), 1);
        let receipt = aidens_contracts::BoundaryCompileReceiptV1::rejected_duplicate_key(
            &profile,
            &serde_json::json!({"raw": "{\"path\":\"a\",\"path\":\"b\"}"}),
        );

        assert!(!receipt.accepted);
        assert!(receipt.duplicate_keys_rejected);
        assert!(receipt
            .reason_codes
            .contains(&"duplicate-json-object-key".into()));
    }

    #[test]
    fn p28_treatment_integrity_receipt_blocks_silent_treatment_change() {
        let profile = aidens_contracts::BoundaryCompilerProfileV1::strict_json("schema:treatment");
        let before = serde_json::json!({"treatment":"control","value":1});
        let after = serde_json::json!({"treatment":"variant","value":1});
        let treatment = aidens_contracts::TreatmentIntegrityReceiptV1::compare(
            &profile,
            &before,
            &after,
            vec!["/treatment".into()],
        );
        let repair = aidens_contracts::BoundaryRepairReceiptV1::new(
            &profile,
            "json-substring-extracted",
            &before,
            &after,
            &treatment,
        );

        assert!(treatment.treatment_changed);
        assert!(treatment.hard_failed);
        assert!(!repair.accepted);
        assert_eq!(repair.treatment_integrity_receipt_id, treatment.receipt_id);
    }

    #[test]
    fn boundary_repair_matches_reference_interpreter() {
        for case in aidens_testkit::reference_cases()
            .into_iter()
            .filter(|case| case.domain == aidens_contracts::ReferenceDomainV1::BoundaryRepair)
        {
            let mut request = BoundaryCompileRequestV1::new(
                case.input["raw"].as_str().expect("raw reference input"),
            );
            request.allow_markdown_fence_repair = case.input["allow_markdown_fence_repair"]
                .as_bool()
                .unwrap_or(true);
            request.allow_json_substring_extract = case.input["allow_json_substring_extract"]
                .as_bool()
                .unwrap_or(true);
            let outcome = compile_json_boundary(request);
            let actual = serde_json::json!({
                "accepted": outcome.accepted,
                "degraded": outcome.degraded,
                "repair_kind": outcome.repair_receipt.as_ref().map(|receipt| receipt.repair_kind.clone()),
                "reason_codes": outcome.reason_codes
            });
            let report = aidens_testkit::compare_case_to_actual(
                &case,
                "aidens-boundary-kit::compile_json_boundary",
                actual,
            );

            assert!(
                report.passed,
                "{}",
                report
                    .findings
                    .iter()
                    .map(|finding| finding.human_diff.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }
}
