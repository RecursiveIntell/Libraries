//! Independent reference interpreters for AiDENs semantic conformance tests.
//!
//! This crate intentionally depends only on contracts and general JSON support.
//! Production crates may use it from tests, but the interpreters here do not
//! call production policy, provider, boundary, tool, permit, or receipt code.

use aidens_contracts::{
    CanonicalToolSideEffectClass, DifferentialConformanceFindingV1, GoldenFixtureManifestV1,
    MemoryModeV1, ReferenceCaseV1, ReferenceDomainV1, ReferenceInterpreterReportV1, ReportLevelV1,
    ToolLifecycleStateV1,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use thiserror::Error;

pub const REFERENCE_PROVIDER_KINDS: &[&str] = &[
    "anthropic",
    "compatible",
    "disabled",
    "mock",
    "ollama",
    "openai",
    "openai-compatible",
    "openrouter",
];

#[derive(Debug, Error)]
pub enum ReferenceError {
    #[error("reference case {case_id} is missing required field {field}")]
    MissingField { case_id: String, field: String },
    #[error("reference case {case_id} has invalid field {field}: {reason}")]
    InvalidField {
        case_id: String,
        field: String,
        reason: String,
    },
    #[error("reference case {case_id} uses unsupported domain {domain}")]
    UnsupportedDomain { case_id: String, domain: String },
}

pub fn all_provider_kinds() -> Vec<String> {
    REFERENCE_PROVIDER_KINDS
        .iter()
        .map(|kind| (*kind).to_string())
        .collect()
}

pub fn all_risk_classes() -> Vec<CanonicalToolSideEffectClass> {
    vec![
        CanonicalToolSideEffectClass::ReadOnly,
        CanonicalToolSideEffectClass::Analysis,
        CanonicalToolSideEffectClass::PreviewWrite,
        CanonicalToolSideEffectClass::Write,
        CanonicalToolSideEffectClass::Admin,
    ]
}

pub fn all_memory_modes() -> Vec<MemoryModeV1> {
    vec![
        MemoryModeV1::Disabled,
        MemoryModeV1::Optional,
        MemoryModeV1::Required,
    ]
}

pub fn all_receipt_levels() -> Vec<ReportLevelV1> {
    vec![
        ReportLevelV1::Minimal,
        ReportLevelV1::Standard,
        ReportLevelV1::Full,
    ]
}

pub fn all_tool_lifecycle_states() -> Vec<ToolLifecycleStateV1> {
    vec![
        ToolLifecycleStateV1::Declared,
        ToolLifecycleStateV1::Registered,
        ToolLifecycleStateV1::Executable,
        ToolLifecycleStateV1::Exposed,
        ToolLifecycleStateV1::ExposedThisTurn,
        ToolLifecycleStateV1::Invoked,
        ToolLifecycleStateV1::Succeeded,
        ToolLifecycleStateV1::Failed,
        ToolLifecycleStateV1::Hidden,
        ToolLifecycleStateV1::Blocked,
    ]
}

pub fn interpret_case(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    match case.domain {
        ReferenceDomainV1::PlanConfig => reference_plan_config_value(case),
        ReferenceDomainV1::ProviderRoute => reference_provider_route_value(case),
        ReferenceDomainV1::ToolExposure => reference_tool_exposure_value(case),
        ReferenceDomainV1::Permit => reference_permit_value(case),
        ReferenceDomainV1::BoundaryRepair => reference_boundary_repair_value(case),
        ReferenceDomainV1::ReceiptLineage => reference_receipt_lineage_value(case),
        ReferenceDomainV1::TemporalQuery => reference_temporal_query_value(case),
        ReferenceDomainV1::ProofDebt => reference_proof_debt_value(case),
        ReferenceDomainV1::SemanticState => reference_semantic_state_value(case),
        ReferenceDomainV1::ViewDisclosure => reference_view_disclosure_value(case),
    }
}

pub fn evaluate_reference_cases(cases: &[ReferenceCaseV1]) -> ReferenceInterpreterReportV1 {
    let mut findings = Vec::new();
    for case in cases {
        match interpret_case(case) {
            Ok(actual) if actual == case.expected => {}
            Ok(actual) => findings.push(DifferentialConformanceFindingV1::mismatch(
                case,
                "aidens-testkit:reference-self-check",
                "$",
                case.expected.clone(),
                actual,
            )),
            Err(error) => findings.push(DifferentialConformanceFindingV1::mismatch(
                case,
                "aidens-testkit:reference-self-check",
                "$",
                case.expected.clone(),
                json!({"error": error.to_string()}),
            )),
        }
    }
    ReferenceInterpreterReportV1::new("aidens-testkit:p09", cases.len(), findings, cases)
}

pub fn compare_case_to_actual(
    case: &ReferenceCaseV1,
    production_subject: &str,
    actual: Value,
) -> ReferenceInterpreterReportV1 {
    let expected = interpret_case(case).unwrap_or_else(|error| json!({"error": error.to_string()}));
    let findings = if expected == actual {
        Vec::new()
    } else {
        vec![DifferentialConformanceFindingV1::mismatch(
            case,
            production_subject,
            "$",
            expected,
            actual,
        )]
    };
    ReferenceInterpreterReportV1::new(production_subject, 1, findings, std::slice::from_ref(case))
}

pub fn reference_cases() -> Vec<ReferenceCaseV1> {
    let mut cases = Vec::new();
    for provider_kind in REFERENCE_PROVIDER_KINDS {
        cases.push(reference_provider_case(provider_kind));
    }
    for risk in all_risk_classes() {
        cases.push(reference_permit_case(risk));
    }
    for memory_mode in all_memory_modes() {
        for receipt_level in all_receipt_levels() {
            cases.push(reference_plan_config_case(
                memory_mode.clone(),
                receipt_level.clone(),
            ));
        }
    }
    cases.push(reference_safe_coding_exposure_case());
    cases.push(reference_tool_lifecycle_catalog_case());
    cases.push(reference_boundary_strict_case());
    cases.push(reference_boundary_markdown_case());
    cases.push(reference_boundary_substring_case());
    cases.push(reference_receipt_lineage_case());
    cases.push(reference_temporal_query_case());
    cases.extend(reference_bitemporal_fixture_cases());
    cases.push(reference_proof_waiver_debt_case());
    cases.push(reference_semantic_state_contamination_case());
    cases.push(reference_view_widening_disclosure_case());
    cases
}

pub fn golden_fixture_manifest() -> GoldenFixtureManifestV1 {
    let cases = reference_cases();
    GoldenFixtureManifestV1::new(
        vec![
            "tests/fixtures/reference/reference_case_v1.json".into(),
            "tests/fixtures/reference/reference_interpreter_report_v1.json".into(),
            "tests/fixtures/reference/differential_conformance_finding_v1.json".into(),
            "tests/fixtures/reference/golden_fixture_manifest_v1.json".into(),
        ],
        &cases,
    )
}

pub fn reference_provider_case(provider_kind: &str) -> ReferenceCaseV1 {
    let input = match provider_kind {
        "mock" => json!({"kind": provider_kind, "mock_response_configured": true}),
        "ollama" => json!({"kind": provider_kind, "model_configured": true}),
        "openai" | "openrouter" | "anthropic" | "openai-compatible" | "compatible" => {
            json!({"kind": provider_kind, "api_key_configured": true, "model_configured": true})
        }
        _ => json!({"kind": provider_kind}),
    };
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ProviderRoute,
        format!("provider route semantics for {provider_kind}"),
        input,
        json!({}),
    )
    .with_provider_kind(provider_kind)
    .with_source_fixture("tests/fixtures/reference/reference_case_v1.json");
    case.expected = reference_provider_route_value(&case).expect("valid provider reference case");
    case
}

pub fn reference_permit_case(risk_class: CanonicalToolSideEffectClass) -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::Permit,
        format!("default permit semantics for {risk_class}"),
        json!({
            "risk_class": json_string(&risk_class),
            "matching_permit": false
        }),
        json!({}),
    )
    .with_risk_class(risk_class)
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_permit_value(&case).expect("valid permit reference case");
    case
}

pub fn reference_plan_config_case(
    memory_mode: MemoryModeV1,
    receipt_level: ReportLevelV1,
) -> ReferenceCaseV1 {
    let memory_store_configured = false;
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::PlanConfig,
        format!("plan config semantics for {memory_mode}/{receipt_level}"),
        json!({
            "memory_mode": json_string(&memory_mode),
            "receipt_level": json_string(&receipt_level),
            "memory_store_configured": memory_store_configured
        }),
        json!({}),
    )
    .with_memory_mode(memory_mode)
    .with_receipt_level(receipt_level)
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_plan_config_value(&case).expect("valid plan reference case");
    case
}

pub fn reference_safe_coding_exposure_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ToolExposure,
        "safe coding exposure semantics",
        json!({
            "max_tools": 16,
            "native_tool_loop_available": false,
            "tools": [
                reference_tool(ReferenceToolSpec::new("aidens:file-write:1", "write")),
                reference_tool(ReferenceToolSpec::new("aidens:file-stat:1", "read_only")
                    .registered()
                    .executable()
                    .allowed_risk()),
                reference_tool(ReferenceToolSpec::new("aidens:memory-write:1", "write")),
                reference_tool(ReferenceToolSpec::new("aidens:network:1", "analysis")),
                reference_tool(ReferenceToolSpec::new("aidens:patch-apply:1", "write")
                    .registered()
                    .executable()),
                reference_tool(ReferenceToolSpec::new("aidens:patch-propose:1", "read_only")
                    .registered()
                    .executable()
                    .allowed_risk()),
                reference_tool(ReferenceToolSpec::new("aidens:repo-list:1", "read_only")
                    .registered()
                    .executable()
                    .allowed_risk()),
                reference_tool(ReferenceToolSpec::new("aidens:repo-read:1", "read_only")
                    .registered()
                    .executable()
                    .allowed_risk()),
                reference_tool(ReferenceToolSpec::new("aidens:repo-search:1", "read_only")
                    .registered()
                    .executable()
                    .allowed_risk()),
                reference_tool(ReferenceToolSpec::new("aidens:run-checks:1", "admin")
                    .registered()
                    .executable()),
                reference_tool(ReferenceToolSpec::new("aidens:schedule:1", "admin")),
                reference_tool(ReferenceToolSpec::new("aidens:shell:1", "admin"))
            ]
        }),
        json!({}),
    )
    .with_risk_class(CanonicalToolSideEffectClass::ReadOnly)
    .with_risk_class(CanonicalToolSideEffectClass::Write)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Declared)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Registered)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Executable)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::ExposedThisTurn)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Hidden)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Blocked)
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_tool_exposure_value(&case).expect("valid exposure reference case");
    case
}

pub fn reference_boundary_markdown_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::BoundaryRepair,
        "markdown fence repair semantics",
        json!({
            "raw": "```json\n{\"ok\":true}\n```",
            "allow_markdown_fence_repair": true,
            "allow_json_substring_extract": true
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_boundary_repair_value(&case).expect("valid boundary reference case");
    case
}

pub fn reference_temporal_query_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::TemporalQuery,
        "temporal as-of reference query semantics",
        json!({
            "valid_at": "2026-04-01T00:00:00Z",
            "recorded_at_or_before": "9999-01-01T00:00:00Z",
            "records": [
                {
                    "claim_id": "claim-phase09-temporal",
                    "claim_version_id": "claim-version-phase09-temporal-old",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": "2026-03-01T00:00:00Z",
                    "recorded_at": "2026-04-28T12:00:00Z",
                    "content": "phase nine temporal old value"
                },
                {
                    "claim_id": "claim-phase09-temporal",
                    "claim_version_id": "claim-version-phase09-temporal-new",
                    "valid_from": "2026-03-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-04-28T12:00:00Z",
                    "content": "phase nine temporal new value"
                }
            ]
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_temporal_query_value(&case).expect("valid temporal reference case");
    case
}

pub fn reference_bitemporal_fixture_cases() -> Vec<ReferenceCaseV1> {
    vec![
        reference_temporal_valid_time_only_case(),
        reference_temporal_recorded_time_only_case(),
        reference_temporal_retroactive_correction_case(),
        reference_temporal_stale_projection_case(),
        reference_temporal_degraded_disclosure_case(),
    ]
}

pub fn reference_temporal_valid_time_only_case() -> ReferenceCaseV1 {
    reference_temporal_fixture_case(
        "valid-time-only temporal reference query semantics",
        json!({
            "valid_at": "2026-02-01T00:00:00Z",
            "records": [
                {
                    "claim_id": "claim-phase09-valid-only",
                    "claim_version_id": "claim-version-phase09-valid-old",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": "2026-03-01T00:00:00Z",
                    "recorded_at": "2026-01-15T00:00:00Z",
                    "content": "valid-only old value"
                },
                {
                    "claim_id": "claim-phase09-valid-only",
                    "claim_version_id": "claim-version-phase09-valid-new",
                    "valid_from": "2026-03-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-01-15T00:00:00Z",
                    "content": "valid-only new value"
                }
            ]
        }),
    )
}

pub fn reference_temporal_recorded_time_only_case() -> ReferenceCaseV1 {
    reference_temporal_fixture_case(
        "recorded-time-only temporal reference query semantics",
        json!({
            "recorded_at_or_before": "2026-02-15T00:00:00Z",
            "records": [
                {
                    "claim_id": "claim-phase09-recorded-only",
                    "claim_version_id": "claim-version-phase09-recorded-early",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-02-01T00:00:00Z",
                    "content": "recorded-only early value"
                },
                {
                    "claim_id": "claim-phase09-recorded-only",
                    "claim_version_id": "claim-version-phase09-recorded-late",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-03-01T00:00:00Z",
                    "content": "recorded-only late value"
                }
            ]
        }),
    )
}

pub fn reference_temporal_retroactive_correction_case() -> ReferenceCaseV1 {
    reference_temporal_fixture_case(
        "retroactive correction and supersession temporal reference semantics",
        json!({
            "valid_at": "2026-04-01T00:00:00Z",
            "recorded_at_or_before": "2026-05-01T00:00:00Z",
            "records": [
                {
                    "claim_id": "claim-phase09-retro",
                    "claim_version_id": "claim-version-phase09-retro-original",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-02-01T00:00:00Z",
                    "content": "retroactive original value"
                },
                {
                    "claim_id": "claim-phase09-retro",
                    "claim_version_id": "claim-version-phase09-retro-correction",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-04-15T00:00:00Z",
                    "supersedes_claim_version_ids": ["claim-version-phase09-retro-original"],
                    "content": "retroactive corrected value"
                }
            ]
        }),
    )
}

pub fn reference_temporal_stale_projection_case() -> ReferenceCaseV1 {
    reference_temporal_fixture_case(
        "stale projection temporal disclosure semantics",
        json!({
            "valid_at": "2026-04-01T00:00:00Z",
            "recorded_at_or_before": "2026-05-01T00:00:00Z",
            "projection_recorded_at": "2026-03-01T00:00:00Z",
            "records": [
                {
                    "claim_id": "claim-phase09-stale",
                    "claim_version_id": "claim-version-phase09-stale-current",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-04-15T00:00:00Z",
                    "content": "stale projection current value"
                }
            ]
        }),
    )
}

pub fn reference_temporal_degraded_disclosure_case() -> ReferenceCaseV1 {
    reference_temporal_fixture_case(
        "degraded temporal disclosure semantics",
        json!({
            "valid_at": "2026-04-01T00:00:00Z",
            "recorded_at_or_before": "2026-05-01T00:00:00Z",
            "temporal_index_exact": false,
            "records": [
                {
                    "claim_id": "claim-phase09-degraded",
                    "claim_version_id": "claim-version-phase09-degraded-candidate",
                    "valid_from": "2026-01-01T00:00:00Z",
                    "valid_to": null,
                    "recorded_at": "2026-04-15T00:00:00Z",
                    "content": "degraded temporal candidate"
                }
            ]
        }),
    )
}

fn reference_temporal_fixture_case(title: &str, input: Value) -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(ReferenceDomainV1::TemporalQuery, title, input, json!({}))
        .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_temporal_query_value(&case).expect("valid temporal reference case");
    case
}

fn reference_boundary_strict_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::BoundaryRepair,
        "strict json boundary semantics",
        json!({"raw": "{\"ok\":true}", "allow_markdown_fence_repair": true, "allow_json_substring_extract": true}),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_boundary_repair_value(&case).expect("valid boundary reference case");
    case
}

fn reference_boundary_substring_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::BoundaryRepair,
        "substring repair semantics",
        json!({"raw": "prefix {\"ok\":true} suffix", "allow_markdown_fence_repair": true, "allow_json_substring_extract": true}),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_boundary_repair_value(&case).expect("valid boundary reference case");
    case
}

fn reference_receipt_lineage_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ReceiptLineage,
        "receipt parent-child lineage semantics",
        json!({
            "receipts": [
                {"receipt_id":"run:fixture","kind":"run","parent_receipt_ids":[]},
                {"receipt_id":"tool:fixture","kind":"tool-invocation","parent_receipt_ids":["run:fixture"]},
                {"receipt_id":"schema:fixture","kind":"schema-validation","parent_receipt_ids":["tool:fixture"]}
            ]
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_receipt_lineage_value(&case).expect("valid lineage reference case");
    case
}

pub fn reference_proof_waiver_debt_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ProofDebt,
        "proof waiver remains debt with expiry escalation semantics",
        json!({
            "now": "2026-05-07T00:00:00Z",
            "requested_use": "local-advisory-display",
            "obligations": [
                {
                    "obligation_id": "proof-obligation:phase09-waiver",
                    "satisfied_by": [],
                    "waived_by": ["proof-waiver:phase09"],
                    "expires_at": "2026-05-01T00:00:00Z"
                },
                {
                    "obligation_id": "proof-obligation:phase09-missing",
                    "satisfied_by": [],
                    "waived_by": []
                }
            ]
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected = reference_proof_debt_value(&case).expect("valid proof debt reference case");
    case
}

pub fn reference_semantic_state_contamination_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::SemanticState,
        "semantic state makes degradation contradiction and execution contamination visible",
        json!({
            "exactness": "exact",
            "degradation_record_ids": ["degradation:phase09"],
            "view_disclosure_ids": ["view-disclosure:phase09"],
            "contradiction_record_ids": ["semantic-contradiction:phase09"],
            "execution_contamination_ids": ["execution-contamination:phase09"]
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected =
        reference_semantic_state_value(&case).expect("valid semantic state reference case");
    case
}

pub fn reference_view_widening_disclosure_case() -> ReferenceCaseV1 {
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ViewDisclosure,
        "view widening and degradation are user visible before results",
        json!({
            "query_widening_receipt_ids": ["query-widening:phase09"],
            "degradation_event_ids": ["degradation-event:phase09"],
            "separates_execution_from_domain_truth": true
        }),
        json!({}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    case.expected =
        reference_view_disclosure_value(&case).expect("valid view disclosure reference case");
    case
}

fn reference_tool_lifecycle_catalog_case() -> ReferenceCaseV1 {
    let states = all_tool_lifecycle_states();
    let mut case = ReferenceCaseV1::new(
        ReferenceDomainV1::ToolExposure,
        "tool lifecycle state catalog semantics",
        json!({"catalog_lifecycle_states": states.iter().map(json_string).collect::<Vec<_>>()}),
        json!({"known_tool_lifecycle_states": states.iter().map(json_string).collect::<Vec<_>>()}),
    )
    .with_source_fixture("tests/fixtures/reference/golden_fixture_manifest_v1.json");
    for state in states {
        case = case.with_tool_lifecycle_state(state);
    }
    case
}

fn reference_provider_route_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let kind = string_field(case, "kind")?.trim().to_ascii_lowercase();
    let api_key_configured = bool_field(case, "api_key_configured", false);
    let model_configured = bool_field(case, "model_configured", false);
    let mock_response_configured = bool_field(case, "mock_response_configured", false);
    let (configured, executable, route_label, reason_codes) = match kind.as_str() {
        "" | "disabled" | "none" => (false, false, "disabled", vec!["provider-disabled"]),
        "mock" => {
            if mock_response_configured {
                (true, true, "mock", vec!["mock-response-configured"])
            } else {
                (true, false, "unavailable", vec!["mock-response-missing"])
            }
        }
        "ollama" => {
            if model_configured {
                (
                    true,
                    false,
                    "unavailable",
                    vec![
                        "ollama-chat-boundary-configured",
                        "ollama-live-probe-required",
                        "ollama-native-tool-loop-unproven",
                    ],
                )
            } else {
                (false, false, "unavailable", vec!["ollama-model-missing"])
            }
        }
        "openai-compatible" => {
            // The bounded HTTP implementation requires an explicit base URL in
            // addition to a model and environment-only bearer credential.
            // This reference case intentionally supplies no base URL.
            (
                false,
                false,
                "unavailable",
                vec!["openai-compatible-base-url-missing"],
            )
        }
        "openai" | "openrouter" | "anthropic" | "compatible" => {
            if api_key_configured {
                (
                    true,
                    false,
                    "unavailable",
                    vec!["provider-boundary-unavailable"],
                )
            } else {
                (true, false, "unavailable", vec!["api-key-missing"])
            }
        }
        _ => (
            true,
            false,
            "unavailable",
            vec!["unsupported-provider-kind"],
        ),
    };
    let native_tool_loop = false;
    Ok(json!({
        "configured": configured,
        "executable": executable,
        "route_label": route_label,
        "native_tool_loop": native_tool_loop,
        "reason_codes": reason_codes
    }))
}

fn reference_plan_config_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let memory_mode = string_field(case, "memory_mode")?;
    let receipt_level = string_field(case, "receipt_level")?;
    let memory_store_configured = bool_field(case, "memory_store_configured", false);
    let memory_blocked = memory_mode == "required" && !memory_store_configured;
    let durable_receipts_required = receipt_level != "minimal";
    let mut reason_codes = vec![
        format!("memory-mode-{memory_mode}"),
        format!("receipt-level-{receipt_level}"),
    ];
    if memory_blocked {
        reason_codes.push("memory-required-without-durable-store".into());
    }
    Ok(json!({
        "memory_mode": memory_mode,
        "receipt_level": receipt_level,
        "memory_store_configured": memory_store_configured,
        "durable_receipts_required": durable_receipts_required,
        "memory_blocked": memory_blocked,
        "reason_codes": sorted_strings(reason_codes)
    }))
}

fn reference_permit_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let risk_class = string_field(case, "risk_class")?;
    let matching_permit = bool_field(case, "matching_permit", false);
    let permit_required = risk_class != "read_only";
    let (decision, reason_codes) = if !permit_required {
        ("allow", vec!["read-only-risk"])
    } else if matching_permit {
        ("allow", vec!["permit-scope-matched"])
    } else {
        ("requires-approval", vec!["approval-required"])
    };
    Ok(json!({
        "risk_class": risk_class,
        "permit_required": permit_required,
        "decision": decision,
        "reason_codes": reason_codes
    }))
}

fn reference_tool_exposure_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    if let Some(catalog) = case.input.get("catalog_lifecycle_states") {
        return Ok(json!({"known_tool_lifecycle_states": catalog.clone()}));
    }

    let max_tools = case
        .input
        .get("max_tools")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX) as usize;
    let native_tool_loop_available = bool_field(case, "native_tool_loop_available", false);
    let tools = case
        .input
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: "tools".into(),
        })?;

    let mut declared = Vec::new();
    let mut registered = Vec::new();
    let mut executable = Vec::new();
    let mut exposed = Vec::new();
    let mut hidden = Vec::new();
    let mut blocked = Vec::new();
    let mut lifecycles = serde_json::Map::new();
    let mut reason_codes = Vec::new();

    for tool in tools {
        let tool_id = tool_field(tool, "tool_id")?;
        let risk_class = tool_field(tool, "risk_class")?;
        let is_declared = tool_bool(tool, "declared", true);
        let is_registered = tool_bool(tool, "registered", false);
        let is_executable = tool_bool(tool, "executable", false);
        let is_hidden = tool_bool(tool, "hidden", false);
        let allowed_risk = tool_bool(tool, "allowed_risk", false);
        let matching_permit = tool_bool(tool, "matching_permit", false);
        let requires_native_tool_loop = tool_bool(tool, "requires_native_tool_loop", false);
        let mut lifecycle = Vec::new();
        if is_declared {
            declared.push(tool_id.clone());
            lifecycle.push("declared");
        }
        if is_registered {
            registered.push(tool_id.clone());
            lifecycle.push("registered");
        }
        if is_executable {
            executable.push(tool_id.clone());
            lifecycle.push("executable");
        }

        let outcome = if !is_registered {
            reason_codes.push(format!("tool-declared-not-registered:{tool_id}"));
            "hidden"
        } else if is_hidden {
            reason_codes.push(format!("tool-hidden:{tool_id}"));
            "hidden"
        } else if !allowed_risk {
            reason_codes.push(format!("risk-blocked:{risk_class}"));
            "blocked"
        } else if risk_class != "read_only" && !matching_permit {
            reason_codes.push(format!("permit-required:{risk_class}"));
            "blocked"
        } else if requires_native_tool_loop && !native_tool_loop_available {
            reason_codes.push(format!("route-requires-native-tool-loop:{tool_id}"));
            "blocked"
        } else if !is_executable {
            reason_codes.push(format!("tool-executor-missing:{tool_id}"));
            "hidden"
        } else if exposed.len() >= max_tools {
            reason_codes.push("max-tools-reached".into());
            "hidden"
        } else {
            "exposed"
        };

        match outcome {
            "exposed" => {
                exposed.push(tool_id.clone());
                lifecycle.push("exposed-this-turn");
            }
            "blocked" => {
                blocked.push(tool_id.clone());
                lifecycle.push("blocked");
            }
            _ => {
                hidden.push(tool_id.clone());
                lifecycle.push("hidden");
            }
        }
        lifecycles.insert(
            tool_id,
            Value::Array(
                lifecycle
                    .into_iter()
                    .map(|state| Value::String(state.into()))
                    .collect(),
            ),
        );
    }

    Ok(json!({
        "declared_tool_ids": sorted_strings(declared),
        "registered_tool_ids": sorted_strings(registered),
        "executable_tool_ids": sorted_strings(executable),
        "exposed_tool_ids": sorted_strings(exposed),
        "hidden_tool_ids": sorted_strings(hidden),
        "blocked_tool_ids": sorted_strings(blocked),
        "lifecycles": Value::Object(lifecycles),
        "reason_codes": sorted_unique_strings(reason_codes)
    }))
}

fn reference_boundary_repair_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let raw = string_field(case, "raw")?;
    let allow_markdown = bool_field(case, "allow_markdown_fence_repair", false);
    let allow_substring = bool_field(case, "allow_json_substring_extract", false);
    if serde_json::from_str::<Value>(&raw).is_ok() {
        return Ok(json!({
            "accepted": true,
            "degraded": false,
            "repair_kind": null,
            "reason_codes": ["boundary-compile-accepted"]
        }));
    }
    if allow_markdown {
        if let Some(candidate) = strip_markdown_json_fence(&raw) {
            if serde_json::from_str::<Value>(&candidate).is_ok() {
                return Ok(repaired_boundary_value("markdown-json-fence-stripped"));
            }
        }
    }
    if allow_substring {
        if let Some(candidate) = extract_first_json_candidate(&raw) {
            if serde_json::from_str::<Value>(&candidate).is_ok() {
                return Ok(repaired_boundary_value("json-substring-extracted"));
            }
        }
    }
    Ok(json!({
        "accepted": false,
        "degraded": true,
        "repair_kind": null,
        "reason_codes": ["invalid-json"]
    }))
}

fn reference_receipt_lineage_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let receipts = case
        .input
        .get("receipts")
        .and_then(Value::as_array)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: "receipts".into(),
        })?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut orphans = Vec::new();
    for receipt in receipts {
        let receipt_id = tool_field(receipt, "receipt_id")?;
        nodes.push(receipt_id.clone());
        let parents = receipt
            .get("parent_receipt_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if parents.is_empty() {
            orphans.push(receipt_id.clone());
        }
        for parent in parents {
            if let Some(parent_id) = parent.as_str() {
                edges.push(format!("{parent_id}->{receipt_id}"));
            }
        }
    }
    Ok(json!({
        "node_count": nodes.len(),
        "edge_count": edges.len(),
        "receipt_ids": sorted_strings(nodes),
        "edges": sorted_strings(edges),
        "orphan_receipt_ids": sorted_strings(orphans)
    }))
}

fn reference_temporal_query_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let valid_at = optional_datetime_field(case, "valid_at")?;
    let recorded_at_or_before = optional_datetime_field(case, "recorded_at_or_before")?;
    let records = case
        .input
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: "records".into(),
        })?;
    let temporal_index_exact = case
        .input
        .get("temporal_index_exact")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let projection_recorded_at = optional_datetime_field(case, "projection_recorded_at")?;
    let mut candidate_claim_version_ids = BTreeSet::new();
    let mut superseded_claim_version_ids = BTreeSet::new();
    let mut max_candidate_recorded_at: Option<DateTime<Utc>> = None;
    for record in records {
        let claim_version_id = value_string_field(record, "claim_version_id", case)?;
        let valid_from = value_datetime_field(record, "valid_from", case)?;
        let valid_to = optional_value_datetime_field(record, "valid_to", case)?;
        let recorded_at = value_datetime_field(record, "recorded_at", case)?;
        let valid_at_visible = valid_at
            .map(|valid_at| {
                valid_from <= valid_at
                    && valid_to.map(|valid_to| valid_at < valid_to).unwrap_or(true)
            })
            .unwrap_or(true);
        let recorded_visible = recorded_at_or_before
            .map(|recorded_at_or_before| recorded_at <= recorded_at_or_before)
            .unwrap_or(true);
        if valid_at_visible && recorded_visible {
            if max_candidate_recorded_at
                .map(|current| recorded_at > current)
                .unwrap_or(true)
            {
                max_candidate_recorded_at = Some(recorded_at);
            }
            for superseded_id in record
                .get("supersedes_claim_version_ids")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                superseded_claim_version_ids.insert(superseded_id.to_string());
            }
            candidate_claim_version_ids.insert(claim_version_id);
        }
    }
    let mut visible_claim_version_ids = Vec::new();
    let mut hidden_claim_version_ids = Vec::new();
    for record in records {
        let claim_version_id = value_string_field(record, "claim_version_id", case)?;
        if candidate_claim_version_ids.contains(&claim_version_id)
            && !superseded_claim_version_ids.contains(&claim_version_id)
        {
            visible_claim_version_ids.push(claim_version_id);
        } else {
            hidden_claim_version_ids.push(claim_version_id);
        }
    }

    let stale_projection = projection_recorded_at
        .zip(max_candidate_recorded_at)
        .map(|(projection_recorded_at, max_recorded_at)| projection_recorded_at < max_recorded_at)
        .unwrap_or(false);
    let degraded = !temporal_index_exact || stale_projection;
    let mut reason_codes = vec!["temporal-query-reference".to_string()];
    if valid_at.is_some() {
        reason_codes.push("valid-time-filter-applied".into());
    }
    if recorded_at_or_before.is_some() {
        reason_codes.push("recorded-time-filter-applied".into());
    }
    if !superseded_claim_version_ids.is_empty() {
        reason_codes.push("supersession-applied".into());
    }
    if stale_projection {
        reason_codes.push("stale-projection-disclosed".into());
    }
    if !temporal_index_exact {
        reason_codes.push("temporal-index-degraded-disclosed".into());
    }
    if !degraded {
        reason_codes.push("temporal-query-exact-reference".into());
    }
    reason_codes.sort();
    reason_codes.dedup();

    Ok(json!({
        "accepted": true,
        "deferred": false,
        "temporal_mode": if degraded { "degraded" } else { "exact" },
        "valid_as_of": valid_at.map(|valid_at| valid_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        "recorded_as_of": recorded_at_or_before.map(|recorded_at_or_before| recorded_at_or_before.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        "visible_claim_version_ids": sorted_strings(visible_claim_version_ids),
        "hidden_claim_version_ids": sorted_strings(hidden_claim_version_ids),
        "stale_projection": stale_projection,
        "view_disclosure_required": degraded,
        "degradation_record_required": degraded,
        "reason_codes": reason_codes
    }))
}

fn reference_proof_debt_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let now = value_datetime_field(&case.input, "now", case)?;
    let requested_use = case
        .input
        .get("requested_use")
        .and_then(Value::as_str)
        .unwrap_or("local-advisory-display");
    let obligations = case
        .input
        .get("obligations")
        .and_then(Value::as_array)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: "obligations".into(),
        })?;
    let mut debt_ids = Vec::new();
    let mut restrictions = Vec::new();
    let mut expired_debt_ids = Vec::new();
    let mut escalated_debt_ids = Vec::new();
    let mut allowed_use_count = 0usize;
    let mut waiver_without_proof = false;
    for obligation in obligations {
        let obligation_id = value_string_field(obligation, "obligation_id", case)?;
        let satisfied = obligation
            .get("satisfied_by")
            .and_then(Value::as_array)
            .map(|items| !items.is_empty())
            .unwrap_or(false);
        if satisfied {
            continue;
        }
        let waived = obligation
            .get("waived_by")
            .and_then(Value::as_array)
            .map(|items| !items.is_empty())
            .unwrap_or(false);
        let debt_id = format!("proof-debt:{obligation_id}");
        let expires_at = optional_value_datetime_field(obligation, "expires_at", case)?;
        let expired = expires_at
            .map(|expires_at| now >= expires_at)
            .unwrap_or(false);
        if waived {
            waiver_without_proof = true;
        }
        if expired {
            expired_debt_ids.push(debt_id.clone());
            escalated_debt_ids.push(debt_id.clone());
            restrictions.push("block-release".to_string());
        } else if waived {
            restrictions.push("advisory-only".to_string());
            if requested_use == "local-advisory-display" || requested_use == "operator-triage" {
                allowed_use_count += 1;
            }
        } else {
            restrictions.push("block-release".to_string());
        }
        debt_ids.push(debt_id);
    }
    let blocks_promotion = !debt_ids.is_empty();
    Ok(json!({
        "debt_ids": sorted_strings(debt_ids),
        "restriction_kinds": sorted_strings(restrictions),
        "waiver_is_not_proof": true,
        "waiver_without_proof": waiver_without_proof,
        "blocks_promotion": blocks_promotion,
        "requested_use_allowed": allowed_use_count > 0 && escalated_debt_ids.is_empty(),
        "expired_debt_ids": sorted_strings(expired_debt_ids),
        "escalated_debt_ids": sorted_strings(escalated_debt_ids),
        "reason_codes": [
            "proof-debt-queryable",
            "proof-waiver-does-not-satisfy-obligation",
            "expired-proof-debt-escalates"
        ]
    }))
}

fn reference_semantic_state_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let exactness = case
        .input
        .get("exactness")
        .and_then(Value::as_str)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: "exactness".into(),
        })?;
    let degradation_ids = optional_string_array(&case.input, "degradation_record_ids");
    let view_ids = optional_string_array(&case.input, "view_disclosure_ids");
    let contradiction_ids = optional_string_array(&case.input, "contradiction_record_ids");
    let contamination_ids = optional_string_array(&case.input, "execution_contamination_ids");
    let visible_degradation = !degradation_ids.is_empty();
    let visible_contradiction = !contradiction_ids.is_empty();
    let visible_contamination = !contamination_ids.is_empty();
    let can_answer_as_exact = exactness == "exact"
        && !visible_degradation
        && !visible_contradiction
        && !visible_contamination;
    Ok(json!({
        "can_answer_as_exact": can_answer_as_exact,
        "blocks_promotion": !can_answer_as_exact,
        "degradation_visible": visible_degradation,
        "view_disclosure_visible": !view_ids.is_empty(),
        "contradiction_visible": visible_contradiction,
        "execution_contamination_visible": visible_contamination,
        "effective_exactness": if visible_contradiction { "refuted" } else if can_answer_as_exact { "exact" } else { "degraded" },
        "reason_codes": [
            "semantic-state-queryable",
            "contradiction-blocks-promotion",
            "execution-contamination-disclosed"
        ]
    }))
}

fn reference_view_disclosure_value(case: &ReferenceCaseV1) -> Result<Value, ReferenceError> {
    let widening_ids = optional_string_array(&case.input, "query_widening_receipt_ids");
    let degradation_ids = optional_string_array(&case.input, "degradation_event_ids");
    let separates_execution = case
        .input
        .get("separates_execution_from_domain_truth")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_visible_event = !widening_ids.is_empty() || !degradation_ids.is_empty();
    Ok(json!({
        "has_visible_widening_or_degradation": has_visible_event,
        "discloses_policy_events": separates_execution
            && widening_ids.iter().all(|id| !id.is_empty())
            && degradation_ids.iter().all(|id| !id.is_empty()),
        "result_exactness": if has_visible_event { "degraded" } else { "exact" },
        "query_widening_receipt_ids": sorted_strings(widening_ids),
        "degradation_event_ids": sorted_strings(degradation_ids),
        "reason_codes": [
            "view-disclosure-queryable",
            "widening-visible-before-results"
        ]
    }))
}

struct ReferenceToolSpec<'a> {
    tool_id: &'a str,
    risk_class: &'a str,
    declared: bool,
    registered: bool,
    executable: bool,
    hidden: bool,
    allowed_risk: bool,
    matching_permit: bool,
    requires_native_tool_loop: bool,
}

impl<'a> ReferenceToolSpec<'a> {
    fn new(tool_id: &'a str, risk_class: &'a str) -> Self {
        Self {
            tool_id,
            risk_class,
            declared: true,
            registered: false,
            executable: false,
            hidden: false,
            allowed_risk: false,
            matching_permit: false,
            requires_native_tool_loop: false,
        }
    }

    fn registered(mut self) -> Self {
        self.registered = true;
        self
    }

    fn executable(mut self) -> Self {
        self.executable = true;
        self
    }

    fn allowed_risk(mut self) -> Self {
        self.allowed_risk = true;
        self
    }
}

fn reference_tool(spec: ReferenceToolSpec<'_>) -> Value {
    json!({
        "tool_id": spec.tool_id,
        "risk_class": spec.risk_class,
        "declared": spec.declared,
        "registered": spec.registered,
        "executable": spec.executable,
        "hidden": spec.hidden,
        "allowed_risk": spec.allowed_risk,
        "matching_permit": spec.matching_permit,
        "requires_native_tool_loop": spec.requires_native_tool_loop
    })
}

fn repaired_boundary_value(repair_kind: &str) -> Value {
    let reason_codes = sorted_strings(vec![
        "boundary-compile-accepted".into(),
        format!("boundary-repair:{repair_kind}"),
        format!("json-repair:{repair_kind}"),
    ]);
    json!({
        "accepted": true,
        "degraded": true,
        "repair_kind": repair_kind,
        "reason_codes": reason_codes
    })
}

fn strip_markdown_json_fence(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let mut lines = trimmed.lines();
    let first = lines.next()?;
    if !first.trim_start_matches("```").trim().is_empty()
        && first.trim_start_matches("```").trim() != "json"
    {
        return None;
    }
    let mut body = lines.collect::<Vec<_>>();
    if body.last().is_some_and(|line| line.trim() == "```") {
        body.pop();
    }
    Some(body.join("\n"))
}

fn extract_first_json_candidate(raw: &str) -> Option<String> {
    let object_start = raw.find('{');
    let array_start = raw.find('[');
    let start = match (object_start, array_start) {
        (Some(left), Some(right)) => left.min(right),
        (Some(left), None) => left,
        (None, Some(right)) => right,
        (None, None) => return None,
    };
    let end = raw.rfind('}').or_else(|| raw.rfind(']'))?;
    (end >= start).then(|| raw[start..=end].to_string())
}

fn string_field(case: &ReferenceCaseV1, field: &str) -> Result<String, ReferenceError> {
    case.input
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ReferenceError::MissingField {
            case_id: case.case_id.as_str().clone(),
            field: field.into(),
        })
}

fn bool_field(case: &ReferenceCaseV1, field: &str, default: bool) -> bool {
    case.input
        .get(field)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}

fn optional_datetime_field(
    case: &ReferenceCaseV1,
    field: &str,
) -> Result<Option<DateTime<Utc>>, ReferenceError> {
    match case.input.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) => parse_datetime_field(raw, field, case).map(Some),
        Some(_) => Err(ReferenceError::InvalidField {
            case_id: case.case_id.as_str().clone(),
            field: field.into(),
            reason: "expected string or null".into(),
        }),
    }
}

fn value_string_field(
    value: &Value,
    field: &str,
    case: &ReferenceCaseV1,
) -> Result<String, ReferenceError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ReferenceError::InvalidField {
            case_id: case.case_id.as_str().clone(),
            field: field.into(),
            reason: "expected string".into(),
        })
}

fn value_datetime_field(
    value: &Value,
    field: &str,
    case: &ReferenceCaseV1,
) -> Result<DateTime<Utc>, ReferenceError> {
    parse_datetime_field(&value_string_field(value, field, case)?, field, case)
}

fn optional_value_datetime_field(
    value: &Value,
    field: &str,
    case: &ReferenceCaseV1,
) -> Result<Option<DateTime<Utc>>, ReferenceError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) => parse_datetime_field(raw, field, case).map(Some),
        Some(_) => Err(ReferenceError::InvalidField {
            case_id: case.case_id.as_str().clone(),
            field: field.into(),
            reason: "expected string or null".into(),
        }),
    }
}

fn optional_string_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn parse_datetime_field(
    raw: &str,
    field: &str,
    case: &ReferenceCaseV1,
) -> Result<DateTime<Utc>, ReferenceError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| ReferenceError::InvalidField {
            case_id: case.case_id.as_str().clone(),
            field: field.into(),
            reason: error.to_string(),
        })
}

fn tool_field(tool: &Value, field: &str) -> Result<String, ReferenceError> {
    tool.get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ReferenceError::InvalidField {
            case_id: "tool-exposure".into(),
            field: field.into(),
            reason: "expected string".into(),
        })
}

fn tool_bool(tool: &Value, field: &str, default: bool) -> bool {
    tool.get(field).and_then(Value::as_bool).unwrap_or(default)
}

pub fn json_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn sorted_unique_strings(values: Vec<String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for value in values {
        set.insert(value);
    }
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_cases_cover_required_catalogs() {
        let cases = reference_cases();
        let report = evaluate_reference_cases(&cases);
        let manifest = golden_fixture_manifest();

        assert!(report.passed, "{:?}", report.findings);
        assert_eq!(manifest.provider_kinds, all_provider_kinds());
        assert_eq!(manifest.risk_classes, all_risk_classes());
        assert_eq!(manifest.memory_modes, all_memory_modes());
        assert_eq!(manifest.receipt_levels, all_receipt_levels());
        assert_eq!(manifest.tool_lifecycle_states, all_tool_lifecycle_states());
    }

    #[test]
    fn mismatch_report_has_human_readable_diff() {
        let case = reference_permit_case(CanonicalToolSideEffectClass::Write);
        let report =
            compare_case_to_actual(&case, "unit-test:bad-permit", json!({"decision":"allow"}));

        assert!(!report.passed);
        assert!(report.findings[0].human_diff.contains("expected"));
        assert!(report.findings[0].human_diff.contains("actual"));
        assert!(report.findings[0]
            .reason_codes
            .contains(&"reference-production-mismatch".into()));
    }

    #[test]
    fn safe_coding_exposure_case_interprets_expected() {
        let case = reference_safe_coding_exposure_case();
        let actual = interpret_case(&case).unwrap();

        assert_eq!(actual, case.expected);
        assert_eq!(
            actual["exposed_tool_ids"],
            json!([
                "aidens:file-stat:1",
                "aidens:patch-propose:1",
                "aidens:repo-list:1",
                "aidens:repo-read:1",
                "aidens:repo-search:1"
            ])
        );
        assert!(actual["blocked_tool_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("aidens:patch-apply:1")));
    }

    #[test]
    fn temporal_query_reference_case_interprets_as_of_semantics() {
        let case = reference_temporal_query_case();
        let actual = interpret_case(&case).unwrap();

        assert_eq!(actual, case.expected);
        assert_eq!(actual["deferred"], json!(false));
        assert_eq!(actual["temporal_mode"], json!("exact"));
        assert_eq!(
            actual["visible_claim_version_ids"],
            json!(["claim-version-phase09-temporal-new"])
        );
        assert_eq!(
            actual["hidden_claim_version_ids"],
            json!(["claim-version-phase09-temporal-old"])
        );
    }

    #[test]
    fn p28_bitemporal_reference_fixture_matrix_interprets_required_cases() {
        let cases = reference_bitemporal_fixture_cases();

        assert_eq!(cases.len(), 5);
        for case in &cases {
            assert_eq!(interpret_case(case).unwrap(), case.expected);
        }

        let valid_only = interpret_case(&cases[0]).unwrap();
        assert_eq!(
            valid_only["visible_claim_version_ids"],
            json!(["claim-version-phase09-valid-old"])
        );
        assert!(valid_only["recorded_as_of"].is_null());

        let recorded_only = interpret_case(&cases[1]).unwrap();
        assert_eq!(
            recorded_only["visible_claim_version_ids"],
            json!(["claim-version-phase09-recorded-early"])
        );
        assert!(recorded_only["valid_as_of"].is_null());

        let retro = interpret_case(&cases[2]).unwrap();
        assert_eq!(
            retro["visible_claim_version_ids"],
            json!(["claim-version-phase09-retro-correction"])
        );
        assert_eq!(
            retro["hidden_claim_version_ids"],
            json!(["claim-version-phase09-retro-original"])
        );
        assert!(retro["reason_codes"]
            .as_array()
            .unwrap()
            .contains(&json!("supersession-applied")));

        let stale = interpret_case(&cases[3]).unwrap();
        assert_eq!(stale["temporal_mode"], json!("degraded"));
        assert_eq!(stale["stale_projection"], json!(true));
        assert_eq!(stale["view_disclosure_required"], json!(true));

        let degraded = interpret_case(&cases[4]).unwrap();
        assert_eq!(degraded["temporal_mode"], json!("degraded"));
        assert_eq!(degraded["degradation_record_required"], json!(true));
        assert!(degraded["reason_codes"]
            .as_array()
            .unwrap()
            .contains(&json!("temporal-index-degraded-disclosed")));
    }

    #[test]
    fn p09_proof_semantic_and_view_reference_cases_are_semantic_not_marker_checks() {
        let proof = interpret_case(&reference_proof_waiver_debt_case()).unwrap();
        assert_eq!(proof["blocks_promotion"], json!(true));
        assert_eq!(proof["waiver_is_not_proof"], json!(true));
        assert_eq!(
            proof["escalated_debt_ids"],
            json!(["proof-debt:proof-obligation:phase09-waiver"])
        );
        assert_eq!(proof["requested_use_allowed"], json!(false));

        let semantic = interpret_case(&reference_semantic_state_contamination_case()).unwrap();
        assert_eq!(semantic["can_answer_as_exact"], json!(false));
        assert_eq!(semantic["effective_exactness"], json!("refuted"));
        assert_eq!(semantic["contradiction_visible"], json!(true));
        assert_eq!(semantic["execution_contamination_visible"], json!(true));

        let view = interpret_case(&reference_view_widening_disclosure_case()).unwrap();
        assert_eq!(view["has_visible_widening_or_degradation"], json!(true));
        assert_eq!(view["discloses_policy_events"], json!(true));
        assert_eq!(view["result_exactness"], json!("degraded"));
    }
}
