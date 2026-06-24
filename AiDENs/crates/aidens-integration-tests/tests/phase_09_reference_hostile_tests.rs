use aidens_agency_kit::{
    AgencyPolicyClassifierKindV1, AgencyPolicyEngineV1, AgencyPolicyInputV1, NudgeLedgerV1,
    AGENCY_POLICY_CLASSIFIER_V1,
};
use aidens_boundary_kit::compile_json_boundary;
use aidens_contracts::{
    ArtifactId, BoundaryCompileRequestV1, DegradationEventV1, ExecutionContaminationRecordV1,
    ProjectionDigestV1, ProofDebtLedgerV1, ProofObligationV1, ProofWaiverReceiptV1,
    ProviderBackendStatusV1, ProviderRouteKindV1, QueryWideningReportV1, RetrievalPolicyV1,
    RuntimeViewModeV1, RuntimeViewRequestV1, SemanticContradictionRecordV1, SemanticStateV1,
    ViewDisclosureReportV1,
};
use aidens_memory_kit::{
    canonical_stack, memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
use aidens_provider_kit::{
    provider_backend_matrix, provider_readiness_for_spec, route_receipt_v2_for_spec, ProviderSpecV1,
};
use aidens_repair_kit::{canonical_stack as repair_stack, CanonicalRepairAdapter};
use aidens_testkit::{
    all_provider_kinds, compare_case_to_actual, interpret_case, reference_proof_waiver_debt_case,
    reference_provider_case, reference_semantic_state_contamination_case,
    reference_temporal_query_case, reference_view_widening_disclosure_case,
};
use chrono::{DateTime, Utc};
use forge_memory_bridge::ImportProjectionRecord;
use knowledge_runtime::QueryWarning;
use semantic_memory::{ProjectionQuery, SearchResult, SearchSource};
use semantic_memory_forge::{
    ExportAuthority, ExportClaim, ExportEnvelopeV3, ExportRecord, ExportRecordV3, ForgeExportMeta,
    RetractionRecordV1, EXPORT_ENVELOPE_V3_SCHEMA, RETRACTION_RECORD_V1_SCHEMA,
};
use serde_json::{json, Value};
use stack_ids::{
    ClaimId, ClaimVersionId, EntityId, EnvelopeId, RetractionRecordId, Scope, ScopeKey, TraceCtx,
};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "aidens-phase09";
const RECORDED_FUTURE: &str = "9999-01-01T00:00:00Z";
const RECORDED_BEFORE_IMPORT: &str = "1970-01-01T00:00:00Z";
const EXPORTED_AT: &str = "2026-04-28T12:00:00Z";

#[test]
fn temporal_reference_interpreter_is_executable() -> Result<(), Box<dyn std::error::Error>> {
    let case = reference_temporal_query_case();
    let actual = interpret_case(&case)?;

    assert_eq!(actual, case.expected);
    assert_eq!(actual["accepted"], json!(true));
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
    Ok(())
}

#[test]
fn proof_semantic_and_view_reference_cases_match_contract_runtime_dtos(
) -> Result<(), Box<dyn std::error::Error>> {
    let proof_expected = interpret_case(&reference_proof_waiver_debt_case())?;
    let now = DateTime::parse_from_rfc3339("2026-05-07T00:00:00Z")?.with_timezone(&Utc);
    let expires_at = DateTime::parse_from_rfc3339("2026-05-01T00:00:00Z")?.with_timezone(&Utc);
    let mut obligation = ProofObligationV1::new("phase 09 proof", "hostile-fixture");
    let waiver =
        ProofWaiverReceiptV1::new(obligation.obligation_id.clone(), "operator", "temporary");
    obligation.waived_by.push(waiver.receipt_id);
    let profile = aidens_contracts::LocalProofProfileV1::local_exact(vec![obligation]);
    let mut debt =
        ProofDebtLedgerV1::from_profile(ArtifactId("artifact:p09-proof".into()), &profile);
    debt.items[0] = debt.items[0].clone().with_expiry(expires_at);
    assert_eq!(
        proof_expected["blocks_promotion"],
        json!(debt.blocks_promotion())
    );
    assert_eq!(
        proof_expected["waiver_without_proof"],
        json!(profile.has_waiver_without_proof())
    );
    assert_eq!(
        proof_expected["requested_use_allowed"],
        json!(debt.allows_use_at("local-advisory-display", now))
    );
    assert_eq!(
        proof_expected["expired_debt_ids"].as_array().unwrap().len(),
        debt.expired_items_at(now).len()
    );
    assert_eq!(
        proof_expected["escalated_debt_ids"]
            .as_array()
            .unwrap()
            .len(),
        debt.escalated_items_at(now).len()
    );

    let semantic_expected = interpret_case(&reference_semantic_state_contamination_case())?;
    let artifact_ref = ArtifactId("artifact:p09-semantic".into());
    let contradiction = SemanticContradictionRecordV1::new(
        artifact_ref.clone(),
        vec![ArtifactId("witness:p09".into())],
        "refuted-by-hostile-fixture",
    );
    let contamination = ExecutionContaminationRecordV1::new(
        artifact_ref.clone(),
        ArtifactId("execution-context:p09".into()),
        "execution-output-used-as-domain-truth",
    );
    let state = SemanticStateV1::exact_supported(artifact_ref, "proof-profile:p09")
        .with_execution_contamination(&contamination)
        .with_contradiction(&contradiction);
    assert_eq!(
        semantic_expected["can_answer_as_exact"],
        json!(state.can_answer_as_exact())
    );
    assert_eq!(
        semantic_expected["blocks_promotion"],
        json!(state.blocks_promotion())
    );
    assert_eq!(
        semantic_expected["contradiction_visible"],
        json!(!state.contradiction_record_ids.is_empty())
    );
    assert_eq!(
        semantic_expected["execution_contamination_visible"],
        json!(!state.execution_contamination_ids.is_empty())
    );

    let view_expected = interpret_case(&reference_view_widening_disclosure_case())?;
    let valid_at = DateTime::parse_from_rfc3339("2026-04-01T00:00:00Z")?.with_timezone(&Utc);
    let recorded_at = DateTime::parse_from_rfc3339("2026-05-01T00:00:00Z")?.with_timezone(&Utc);
    let policy = RetrievalPolicyV1::time_scoped(RuntimeViewModeV1::Temporal, valid_at, recorded_at)
        .with_alias_expansion();
    let request = RuntimeViewRequestV1::new("phase nine", policy.clone()).with_alias("phase 9");
    let widening = QueryWideningReportV1::alias_expansion(
        &request,
        vec!["phase nine".into()],
        vec!["phase nine".into(), "phase 9".into()],
    );
    let degradation = DegradationEventV1::new(
        &request,
        "stale-projection-disclosed",
        false,
        false,
        Vec::new(),
    );
    let projection = ProjectionDigestV1::new(
        RuntimeViewModeV1::Temporal,
        policy.policy_id.clone(),
        Vec::new(),
        vec![ArtifactId("claim:p09".into())],
        Vec::new(),
        json!({"claim_ids":["claim:p09"]}),
    );
    let disclosure = ViewDisclosureReportV1::new(
        &request,
        projection,
        vec![ArtifactId("claim:p09".into())],
        vec![widening.receipt_id],
        vec![degradation.event_id],
    );
    assert_eq!(
        view_expected["has_visible_widening_or_degradation"],
        json!(disclosure.has_visible_widening_or_degradation())
    );
    assert_eq!(
        view_expected["discloses_policy_events"],
        json!(disclosure.discloses_policy_events())
    );
    Ok(())
}

#[tokio::test]
async fn temporal_asof_reference_matches_canonical_memory_runtime(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = reference_temporal_query_case();
    let expected = interpret_case(&case)?;
    let root = temp_root("phase-09-temporal-reference");
    let memory_root = root.join("memory");
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;
    let trace_ctx = TraceCtx::from_trace_id("trace-phase09-temporal");
    let envelope = forge_envelope(
        "env-phase09-temporal",
        &trace_ctx,
        vec![
            claim_export(
                "claim-phase09-temporal",
                "claim-version-phase09-temporal-old",
                "phase nine temporal old value",
                "2026-01-01T00:00:00Z",
                Some("2026-03-01T00:00:00Z"),
            ),
            claim_export(
                "claim-phase09-temporal",
                "claim-version-phase09-temporal-new",
                "phase nine temporal new value",
                "2026-03-01T00:00:00Z",
                None,
            ),
        ],
    );

    envelope.validate()?;
    adapter.import_forge_export(&envelope).await?;

    let mut query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    query.claim_id = Some(ClaimId::new("claim-phase09-temporal"));
    query.valid_at = Some(expected_string(&expected, "valid_as_of"));
    query.recorded_at_or_before = Some(expected_string(&expected, "recorded_as_of"));
    query.limit = 10;
    let rows = adapter.store().query_claim_versions(query).await?;
    assert_eq!(
        json!(claim_version_ids(&rows)),
        expected["visible_claim_version_ids"]
    );

    let mut pre_import_query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    pre_import_query.claim_id = Some(ClaimId::new("claim-phase09-temporal"));
    pre_import_query.valid_at = Some(expected_string(&expected, "valid_as_of"));
    pre_import_query.recorded_at_or_before = Some(RECORDED_BEFORE_IMPORT.into());
    let pre_import_rows = adapter
        .store()
        .query_claim_versions(pre_import_query)
        .await?;
    assert!(
        pre_import_rows.is_empty(),
        "recorded_at_or_before must exclude rows imported after the transaction-time cutoff"
    );

    let scope = Scope::new(NAMESPACE);
    let (results, trace) = adapter
        .query_temporal(
            "phase nine temporal",
            Some(&scope),
            &expected_string(&expected, "valid_as_of"),
            &expected_string(&expected, "recorded_as_of"),
        )
        .await?;
    assert!(projection_result_ids(&results).contains("claim-version-phase09-temporal-new"));
    assert!(!contains_content(&results, "phase nine temporal old value"));
    assert_eq!(trace.temporal_mode.as_deref(), Some("exact"));
    assert_eq!(
        trace.valid_as_of.as_deref(),
        Some(expected_string(&expected, "valid_as_of").as_str())
    );
    assert_eq!(
        trace.recorded_as_of.as_deref(),
        Some(expected_string(&expected, "recorded_as_of").as_str())
    );
    assert!(!trace.has_temporal_downgrade());

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn bridge_digest_backpointer_atomicity_has_failure_receipt(
) -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-09-bridge-atomicity");
    let memory_root = root.join("memory");
    let trace_ctx = TraceCtx::from_trace_id("trace-phase09-bridge");
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;

    let baseline = forge_envelope(
        "env-phase09-atomic-baseline",
        &trace_ctx,
        vec![claim_export(
            "claim-phase09-atomic-baseline",
            "claim-version-phase09-atomic-baseline",
            "phase nine atomic baseline",
            "2026-01-01T00:00:00Z",
            None,
        )],
    );
    let baseline_batch = canonical_stack::transform_forge_export(&baseline)?;
    assert_eq!(baseline_batch.source_envelope_id, baseline.envelope_id);
    assert_eq!(baseline_batch.content_digest, baseline.content_digest);
    assert_eq!(
        baseline_batch.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );
    assert!(baseline_batch
        .records
        .iter()
        .all(|record| match &record.record {
            ImportProjectionRecord::ClaimVersion(claim) =>
                claim.source_envelope_id.as_str() == baseline.envelope_id.as_str(),
            _ => true,
        }));
    canonical_stack::import_projection_batch(adapter.store(), &baseline_batch).await?;

    let logs = adapter
        .store()
        .query_projection_imports(Some(NAMESPACE), 10)
        .await?;
    let baseline_log = logs
        .iter()
        .find(|log| log.source_envelope_id == "env-phase09-atomic-baseline")
        .expect("baseline import log");
    assert_eq!(
        baseline_log.content_digest,
        baseline_batch.content_digest.hex()
    );
    assert_eq!(baseline_log.status, "complete");

    let failing = forge_envelope(
        "env-phase09-atomic-failing",
        &trace_ctx,
        vec![
            claim_export(
                "claim-phase09-atomic-extra",
                "claim-version-phase09-atomic-extra",
                "phase nine atomic extra must roll back",
                "2026-01-01T00:00:00Z",
                None,
            ),
            claim_export(
                "claim-phase09-atomic-baseline",
                "claim-version-phase09-atomic-conflict",
                "phase nine atomic conflict must fail",
                "2026-02-01T00:00:00Z",
                None,
            ),
        ],
    );
    let failing_batch = canonical_stack::transform_forge_export(&failing)?;
    let error = canonical_stack::import_projection_batch(adapter.store(), &failing_batch)
        .await
        .expect_err("overlapping preferred-open claim interval must fail");
    assert!(error
        .to_string()
        .contains("preferred-open claim interval conflict"));

    let extra_rows = query_claim(adapter.store(), "claim-phase09-atomic-extra").await?;
    assert!(
        extra_rows.is_empty(),
        "failed bridge import must not partially commit non-conflicting rows"
    );
    let failures = adapter
        .store()
        .query_projection_import_failures(Some(NAMESPACE), 10)
        .await?;
    let failure = failures
        .iter()
        .find(|failure| failure.source_envelope_id == "env-phase09-atomic-failing")
        .expect("failed import receipt");
    assert_eq!(failure.content_digest, failing_batch.content_digest.hex());
    assert_eq!(
        failure.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );
    assert!(failure
        .error_message
        .contains("preferred-open claim interval conflict"));

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn provider_capability_truth_rejects_fallback_as_support() -> Result<(), Box<dyn std::error::Error>>
{
    for provider_kind in all_provider_kinds() {
        let case = reference_provider_case(&provider_kind);
        let spec = provider_spec_from_reference_case(&provider_kind, &case.input);
        let readiness = provider_readiness_for_spec(&spec);
        let actual = json!({
            "configured": readiness.configured,
            "executable": readiness.executable,
            "route_label": readiness.route_label,
            "native_tool_loop": readiness.native_tool_loop_executable,
            "reason_codes": readiness.reason_codes
        });
        let report = compare_case_to_actual(
            &case,
            "aidens-provider-kit::provider_readiness_for_spec",
            actual,
        );
        assert!(report.passed, "{:?}", report.findings);

        let route = route_receipt_v2_for_spec(&spec);
        if provider_kind != "ollama" {
            assert!(!route.native_tool_loop, "{provider_kind}");
        }
        if matches!(
            provider_kind.as_str(),
            "openai" | "openrouter" | "anthropic"
        ) {
            assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
            assert!(!route.chat_completion_executable);
        }
    }

    let matrix = provider_backend_matrix();
    // Ollama now supports native tool loop; mock and others don't
    assert!(matrix
        .entries
        .iter()
        .filter(|e| e.provider_kind != "ollama")
        .all(|entry| !entry.native_tool_loop_ready()));
    assert_eq!(
        matrix.entry_for("mock").expect("mock provider").status,
        ProviderBackendStatusV1::Executable
    );
    assert!(
        !matrix
            .entry_for("mock")
            .expect("mock provider")
            .native_tool_loop_executable
    );
    let ollama = matrix.entry_for("ollama").expect("ollama provider");
    assert_eq!(ollama.status, ProviderBackendStatusV1::Executable);
    assert!(ollama.native_tool_loop_executable);
    assert!(ollama
        .reason_codes
        .contains(&"ollama-native-tool-loop-via-function-calling".into()));
    Ok(())
}

#[test]
fn agency_eval_cases_match_decision_semantics_and_required_receipts(
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = AgencyPolicyEngineV1::default();
    for line in include_str!("../../../evals/p20_agency_eval_cases.jsonl").lines() {
        if line.trim().is_empty() {
            continue;
        }
        let case: Value = serde_json::from_str(line)?;
        let id = required_str(&case, "id");
        let risk_surface = required_str(&case, "risk_surface");
        let expected_policy = required_str(&case, "expected_policy");
        let input = AgencyPolicyInputV1::from_eval_case(id, risk_surface, &case["input"])?;
        let mut ledger = NudgeLedgerV1::default();
        let report = engine.evaluate(&input, &mut ledger);

        assert_eq!(report.outcome.as_policy_label(), expected_policy, "{id}");
        assert_eq!(report.classifier_id, AGENCY_POLICY_CLASSIFIER_V1, "{id}");
        assert_eq!(
            report.classifier_kind,
            AgencyPolicyClassifierKindV1::HeuristicBoundaryClassifier,
            "{id}"
        );
        let blocked = report
            .blocked_behavior
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for forbidden in required_string_array(&case, "forbidden_behavior") {
            assert!(blocked.contains(&forbidden), "{id}: {forbidden}");
        }
        let receipt_schemas = report.receipt_schema_names();
        for receipt in required_string_array(&case, "required_receipts") {
            assert!(receipt_schemas.contains(&receipt), "{id}: {receipt}");
        }
        assert!(!report.receipt_ids().is_empty(), "{id}");
        assert!(!report.reason_codes().is_empty(), "{id}");
    }
    Ok(())
}

#[test]
fn boundary_repair_hard_fails_unverifiable_treatment_change() {
    let mut request = BoundaryCompileRequestV1::new(
        "assistant preface {\"dosage\":\"increase\",\"ok\":true} trailing",
    )
    .with_treatment_critical_fields(vec!["dosage".into()])
    .with_hard_fail_on_treatment_change(true);
    request.allow_json_substring_extract = true;

    let outcome = compile_json_boundary(request);

    assert!(!outcome.accepted);
    assert!(outcome.degraded);
    assert!(outcome
        .reason_codes
        .contains(&"treatment-integrity-hard-fail".into()));
    let repair = outcome.repair_receipt.expect("repair receipt");
    assert_eq!(repair.repair_kind, "json-substring-extracted");
    assert!(repair.changed);
    assert!(repair.hard_failed);
    assert!(repair
        .reason_codes
        .contains(&"treatment-integrity-unverifiable".into()));
    assert!(repair.treatment_integrity_warnings.iter().any(|warning| {
        warning == "treatment-integrity-unverifiable:json-substring-extracted:dosage"
    }));
    assert!(repair.canonical_backpointers.iter().any(|pointer| {
        pointer.owner_crate == "verification-control"
            && pointer.owner_type == "BoundaryRepairRecord"
    }));
}

#[tokio::test]
async fn runtime_widening_disclosure_is_explicit_on_degraded_temporal_query(
) -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-09-runtime-widening");
    let memory_root = root.join("memory");
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;
    let scope = Scope::new(NAMESPACE);

    let (_results, trace) = adapter
        .query_temporal(
            "phase nine empty temporal query",
            Some(&scope),
            "2026-04-01T00:00:00Z",
            RECORDED_FUTURE,
        )
        .await?;

    assert!(trace.is_degraded());
    assert!(trace.has_temporal_downgrade());
    assert!(trace.warnings.iter().any(|warning| matches!(
        warning,
        QueryWarning::TemporalDowngradedToHybrid { temporal_expr }
            if temporal_expr == "explicit bitemporal filters"
    )));
    assert!(trace.widenings.iter().any(|widening| {
        widening
            .reason
            .contains("temporal route degraded to semantic hybrid execution")
    }));
    let provenance = trace.runtime_query_provenance();
    assert_eq!(
        provenance.valid_as_of.as_deref(),
        Some("2026-04-01T00:00:00Z")
    );
    assert_eq!(provenance.recorded_as_of.as_deref(), Some(RECORDED_FUTURE));
    assert!(!provenance.warnings.is_empty());
    assert!(!provenance.widenings.is_empty());

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn repair_records_preserve_canonical_invariants_and_reject_empty_reason(
) -> Result<(), Box<dyn std::error::Error>> {
    let repair = CanonicalRepairAdapter.boundary_repair_record(
        repair_stack::BoundaryArtifactKind::ControlReceipt,
        repair_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA,
        "phase09-hostile-repair-record",
        "$.details.repair_backpointer",
        Some(json!(null)),
        json!({
            "source_control_receipt_id": "control-receipt:phase09",
            "replay_ref": "phase09-replay-ref",
            "canonical_owner": "verification-control"
        }),
        "Phase 09 hostile repair-record invariant proof",
    );
    assert_eq!(
        repair.schema_version,
        repair_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA
    );
    assert_eq!(repair.repair_kind, "phase09-hostile-repair-record");
    assert_eq!(repair.field_path, "$.details.repair_backpointer");
    assert_eq!(
        repair.repaired_value["source_control_receipt_id"],
        "control-receipt:phase09"
    );

    let valid_retraction = RetractionRecordV1 {
        schema_version: RETRACTION_RECORD_V1_SCHEMA.into(),
        retraction_record_id: RetractionRecordId::new("retraction-phase09-hostile"),
        claim_id: ClaimId::new("claim-phase09-repair"),
        retracted_claim_version_id: ClaimVersionId::new("claim-version-phase09-repair"),
        superseded_by_claim_version_id: None,
        effective_recorded_at: EXPORTED_AT.into(),
        reason: format!(
            "canonical boundary repair record {}",
            repair.repair_record_id
        ),
        cascade_required: true,
        delta_summary: Some(
            json!({
                "boundary_repair_record_id": repair.repair_record_id.to_string(),
                "source_control_receipt_id": "control-receipt:phase09",
                "canonical_owner": "semantic-memory-forge"
            })
            .to_string(),
        ),
    };
    CanonicalRepairAdapter.validate_retraction(&valid_retraction)?;
    assert!(valid_retraction.cascade_required);
    assert!(valid_retraction
        .delta_summary
        .as_deref()
        .is_some_and(|summary| summary.contains(repair.repair_record_id.as_str())));

    let mut invalid_retraction = valid_retraction.clone();
    invalid_retraction.reason.clear();
    let error = CanonicalRepairAdapter
        .validate_retraction(&invalid_retraction)
        .expect_err("empty retraction reason must be rejected");
    assert!(error.contains("reason"));
    Ok(())
}

async fn query_claim(
    store: &canonical_stack::CanonicalMemoryStore,
    claim_id: &str,
) -> Result<Vec<semantic_memory::ProjectionClaimVersion>, semantic_memory::MemoryError> {
    let mut query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    query.claim_id = Some(ClaimId::new(claim_id));
    query.limit = 10;
    store.query_claim_versions(query).await
}

fn claim_export(
    claim_id: &str,
    claim_version_id: &str,
    content: &str,
    valid_from: &str,
    valid_to: Option<&str>,
) -> ExportRecordV3 {
    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(claim_id)),
            claim_version_id: Some(ClaimVersionId::new(claim_version_id)),
            subject_entity_id: EntityId::new("entity-phase09"),
            predicate: "phase09_reference_state".into(),
            object_anchor: json!("canonical-reference-proof"),
            valid_from: Some(valid_from.into()),
            valid_to: valid_to.map(str::to_string),
            confidence: 0.99,
            content: content.into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(json!({
                "canonical_owner": "semantic-memory-forge",
                "phase": "09"
            })),
        }),
        semantics: None,
    }
}

fn forge_envelope(
    envelope_id: &str,
    trace_ctx: &TraceCtx,
    records: Vec<ExportRecordV3>,
) -> ExportEnvelopeV3 {
    let scope_key = ScopeKey::namespace_only(NAMESPACE);
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some(envelope_id.into()),
        direct_write: false,
        comparability_snapshot_version: Some("cmp-phase09-reference".into()),
        exported_at: EXPORTED_AT.into(),
    };
    let content_digest = ExportEnvelopeV3::compute_digest(
        "semantic-memory-forge",
        &scope_key,
        &records,
        Some(&export_meta),
        None,
    )
    .expect("canonical v3 digest");

    ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(envelope_id),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest,
        source_authority: "semantic-memory-forge".into(),
        scope_key,
        trace_ctx: Some(trace_ctx.clone()),
        exported_at: EXPORTED_AT.into(),
        export_meta: Some(export_meta),
        evidence_bundle: None,
        support_sets: Vec::new(),
        contradiction_witnesses: Vec::new(),
        retraction_records: Vec::new(),
        claim_states_v13: Vec::new(),
        intervention_bundles_v14: Vec::new(),
        outcome_schemas_v14: Vec::new(),
        cohort_contracts_v14: Vec::new(),
        counterfactual_slices_v14: Vec::new(),
        experiment_cases_v14: Vec::new(),
        comparability_matrices_v14: Vec::new(),
        decision_traces_v14: Vec::new(),
        refuter_suites_v14: Vec::new(),
        refuter_results_v14: Vec::new(),
        experiment_budgets_v14: Vec::new(),
        rollout_decisions_v14: Vec::new(),
        rollback_decisions_v14: Vec::new(),
        attestation_envelopes_v15: Vec::new(),
        trust_root_sets_v15: Vec::new(),
        artifact_admission_policies_v15: Vec::new(),
        transparency_receipts_v15: Vec::new(),
        attestation_revocations_v15: Vec::new(),
        attestation_supersessions_v15: Vec::new(),
        remote_oracle_leases_v15: Vec::new(),
        remote_slice_requests_v15: Vec::new(),
        remote_slice_results_v15: Vec::new(),
        cross_runtime_replay_tickets_v15: Vec::new(),
        dispute_bundles_v15: Vec::new(),
        disclosure_policies_v15: Vec::new(),
        disclosure_budgets_v15: Vec::new(),
        records,
    }
}

fn provider_spec_from_reference_case(provider_kind: &str, input: &Value) -> ProviderSpecV1 {
    let mut spec = ProviderSpecV1::new(provider_kind);
    if input
        .get("api_key_configured")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        spec.api_key = Some("configured".into());
    }
    if input
        .get("model_configured")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        spec.model = Some("model".into());
    }
    if input
        .get("mock_response_configured")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        spec.mock_response = Some("ok".into());
    }
    spec
}

fn expected_string(value: &Value, field: &str) -> String {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("expected {field} string"))
        .to_string()
}

fn claim_version_ids(rows: &[semantic_memory::ProjectionClaimVersion]) -> Vec<String> {
    let mut ids = rows
        .iter()
        .map(|row| row.claim_version_id.to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn projection_result_ids(results: &[SearchResult]) -> BTreeSet<String> {
    results
        .iter()
        .filter_map(|result| match &result.source {
            SearchSource::Projection {
                projection_kind,
                projection_id,
                ..
            } if projection_kind == "claim_version" => Some(projection_id.clone()),
            _ => None,
        })
        .collect()
}

fn contains_content(results: &[SearchResult], needle: &str) -> bool {
    results.iter().any(|result| result.content.contains(needle))
}

fn required_str<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("expected {field} string"))
}

fn required_string_array(value: &Value, field: &str) -> Vec<String> {
    value[field]
        .as_array()
        .unwrap_or_else(|| panic!("expected {field} array"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("expected {field} string item"))
                .to_string()
        })
        .collect()
}

fn temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
