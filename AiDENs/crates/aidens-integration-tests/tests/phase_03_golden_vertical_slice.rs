use aidens_cli::{view_command, ViewCommand};
use aidens_memory_kit::{
    canonical_stack, memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
use aidens_receipts::{
    forge_tool_receipt_from_runtime, CanonicalEventLog, CanonicalEventLogConfig,
};
use llm_tool_runtime::{
    ToolApprovalState, ToolBackendKind, ToolPlannerStage, ToolReceipt, ToolRetryOwner,
};
use semantic_memory::{ProjectionQuery, SearchSource};
use semantic_memory_forge::{
    CausalQuestion, EvidenceBundle, EvidenceBundleId, ExportAuthority, ExportClaim,
    ExportEnvelopeV3, ExportEpisode, ExportEvidenceRef, ExportRecord, ExportRecordV3,
    ForgeExportMeta, OutcomeSpec, TreatmentSpec, EXPORT_ENVELOPE_V3_SCHEMA,
};
use serde_json::Value;
use stack_ids::{
    AttemptId, ClaimId, ClaimVersionId, ContentDigest, EntityId, EnvelopeId, EpisodeId, Scope,
    ScopeKey, TraceCtx, TrialId,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "aidens";
const TIMESTAMP: &str = "2026-04-28T12:00:00Z";
const OPERATOR_INPUT: &str = "prove the golden vertical slice through canonical stack crates";
const QUERY: &str = "golden vertical slice operator receipt";
const RUNTIME_RECEIPT_ID: &str = "receipt-golden-runtime-001";
const CLAIM_ID: &str = "claim-golden-vertical-slice";
const CLAIM_VERSION_ID: &str = "claim-version-golden-vertical-slice-001";
const ENVELOPE_ID: &str = "env-golden-vertical-slice-001";

#[test]
fn golden_vertical_slice() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-03-golden-vertical-slice");
    let receipt_root = root.join("receipts");
    let memory_root = root.join("memory");
    let trace_ctx = TraceCtx::generate();
    let runtime = tokio::runtime::Runtime::new()?;

    let runtime_receipt = build_runtime_tool_receipt(&trace_ctx);
    let forge_receipt = forge_tool_receipt_from_runtime(
        &runtime_receipt,
        serde_json::json!({
            "operator_input": OPERATOR_INPUT,
            "result": "canonical golden vertical slice payload",
        }),
    );
    assert_eq!(forge_receipt.receipt_id, runtime_receipt.receipt_id);
    assert_eq!(forge_receipt.trace_ctx.trace_id, trace_ctx.trace_id);

    let receipt_log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipt_root))?;
    let runtime_record = receipt_log.append_runtime_tool_receipt(&runtime_receipt)?;
    let forge_record = receipt_log.append_forge_tool_receipt(&forge_receipt)?;
    assert_eq!(runtime_record.owner_crate, "llm-tool-runtime");
    assert_eq!(forge_record.owner_crate, "semantic-memory-forge");
    assert_ne!(forge_record.receipt_id, runtime_record.receipt_id);
    assert_eq!(forge_record.body["receipt_id"], forge_receipt.receipt_id);
    assert!(receipt_log.verify_digest(RUNTIME_RECEIPT_ID)?);
    assert!(receipt_log.verify_digest(&forge_record.receipt_id)?);

    let envelope = build_forge_envelope(
        &trace_ctx,
        &runtime_receipt.receipt_id,
        &forge_receipt.receipt_id,
    );
    envelope.validate()?;

    let bridge_batch = canonical_stack::transform_forge_export(&envelope)?;
    assert_eq!(
        bridge_batch.schema_version,
        canonical_stack::PROJECTION_IMPORT_BATCH_V3_SCHEMA
    );
    assert_eq!(
        bridge_batch.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );
    assert_eq!(bridge_batch.source_envelope_id, envelope.envelope_id);
    assert_eq!(bridge_batch.scope_key, envelope.scope_key);
    assert!(bridge_batch.execution_context.is_some());
    assert!(bridge_batch.evidence_bundle.is_some());
    assert_eq!(bridge_batch.records.len(), envelope.records.len());

    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;
    let import_result = runtime.block_on(canonical_stack::import_projection_batch(
        adapter.store(),
        &bridge_batch,
    ))?;
    assert_eq!(import_result.status, "complete");
    assert_eq!(import_result.record_count, bridge_batch.records.len());

    let mut projection_query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    projection_query.claim_id = Some(ClaimId::new(CLAIM_ID));
    let claim_rows = runtime.block_on(adapter.store().query_claim_versions(projection_query))?;
    assert_eq!(claim_rows.len(), 1);
    assert_eq!(
        claim_rows[0].claim_version_id,
        ClaimVersionId::new(CLAIM_VERSION_ID)
    );
    assert_eq!(
        claim_rows[0].trace_id.as_deref(),
        Some(trace_ctx.trace_id.as_str())
    );
    assert!(claim_rows[0].content.contains(RUNTIME_RECEIPT_ID));
    assert!(claim_rows[0]
        .content
        .contains(forge_receipt.receipt_id.as_str()));

    let scope = Scope::new(NAMESPACE);
    let (runtime_results, runtime_trace) = runtime.block_on(adapter.runtime().query_with_trace(
        QUERY,
        Some(&scope),
        Some(trace_ctx.clone()),
    ))?;
    assert_eq!(runtime_trace.trace_ctx.trace_id, trace_ctx.trace_id);
    assert!(runtime_results.iter().any(|result| matches!(
        &result.source,
        SearchSource::Projection {
            projection_kind,
            source_envelope_id,
            ..
        } if projection_kind == "claim_version" && source_envelope_id == ENVELOPE_ID
    )));

    let cli_output = view_command(ViewCommand::Query {
        memory_store: memory_root.to_string_lossy().into_owned(),
        view_mode: "temporal".into(),
        query: QUERY.into(),
        subject: None,
        predicate: None,
        valid_at: None,
        recorded_at: None,
        aliases: Vec::new(),
        allow_alias_expansion: false,
        allow_timeless_fallback: false,
    })?;
    let cli_json: Value = serde_json::from_str(&cli_output)?;
    assert_eq!(cli_json["kind"], "canonical-runtime-view");
    assert_eq!(cli_json["canonical_owner"], "knowledge-runtime");
    assert_eq!(cli_json["memory_owner"], "semantic-memory");
    assert!(cli_json["result_count"].as_u64().unwrap_or_default() > 0);
    assert!(cli_output.contains(RUNTIME_RECEIPT_ID));
    assert!(cli_output.contains(forge_receipt.receipt_id.as_str()));
    assert!(cli_output.contains(ENVELOPE_ID));

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

fn build_runtime_tool_receipt(trace_ctx: &TraceCtx) -> ToolReceipt {
    ToolReceipt {
        receipt_id: RUNTIME_RECEIPT_ID.into(),
        tool_name: "aidens.golden_vertical_slice".into(),
        tool_version: "0.1.0".into(),
        backend_kind: ToolBackendKind::LocalFunction,
        input_digest: ContentDigest::compute_str(OPERATOR_INPUT),
        output_digest_or_refs: serde_json::json!({
            "claim_id": CLAIM_ID,
            "operator_input": OPERATOR_INPUT,
        }),
        policy_hash: ContentDigest::compute_str("phase-03-readonly-policy"),
        approval_state: ToolApprovalState::NotRequired,
        host_identity: "aidens-testkit".into(),
        started_at: TIMESTAMP.into(),
        finished_at: TIMESTAMP.into(),
        trace_ctx: trace_ctx.clone(),
        attempt_id: AttemptId::new("attempt-golden-vertical-slice-001"),
        trial_id: TrialId::new("trial-golden-vertical-slice-001"),
        planner_stage: ToolPlannerStage::Execution,
        deadline: None,
        workload_class: Some("phase-03-golden-slice".into()),
        budget_context: None,
        parent_receipt_id: None,
        family_receipt_id: None,
        replay_parent_receipt_id: None,
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        error_class: None,
        retry_owner: ToolRetryOwner::ForgeOrchestration,
        replay_link: None,
        tool_run_id: "tool-run-golden-vertical-slice-001".into(),
        provider_call_id: None,
    }
}

fn build_forge_envelope(
    trace_ctx: &TraceCtx,
    runtime_receipt_id: &str,
    forge_receipt_id: &str,
) -> ExportEnvelopeV3 {
    let scope_key = ScopeKey::namespace_only(NAMESPACE);
    let records = vec![
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: Some(ClaimId::new(CLAIM_ID)),
                claim_version_id: Some(ClaimVersionId::new(CLAIM_VERSION_ID)),
                subject_entity_id: EntityId::new("entity-aidens-wrapper"),
                predicate: "delegates_to".into(),
                object_anchor: serde_json::json!("canonical Libraries crates"),
                valid_from: Some("2026-04-28T00:00:00Z".into()),
                valid_to: None,
                confidence: 0.99,
                content: format!(
                    "golden vertical slice operator receipt proves {runtime_receipt_id} \
                     became Forge receipt {forge_receipt_id}, bridge batch, semantic-memory \
                     projection, knowledge-runtime result, and CLI output"
                ),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: Some(serde_json::json!({
                    "operator_input": OPERATOR_INPUT,
                    "runtime_tool_receipt_id": runtime_receipt_id,
                    "forge_tool_receipt_id": forge_receipt_id,
                    "canonical_owner": "semantic-memory-forge"
                })),
            }),
            semantics: None,
        },
        ExportRecordV3 {
            record: ExportRecord::EvidenceRef(ExportEvidenceRef {
                claim_id: ClaimId::new(CLAIM_ID),
                claim_version_id: Some(ClaimVersionId::new(CLAIM_VERSION_ID)),
                fetch_handle: format!("receipt-log://{runtime_receipt_id}"),
                source_authority: "semantic-memory-forge".into(),
                metadata: Some(serde_json::json!({
                    "forge_tool_receipt_id": forge_receipt_id,
                })),
            }),
            semantics: None,
        },
        ExportRecordV3 {
            record: ExportRecord::Episode(ExportEpisode {
                episode_id: Some(EpisodeId::new("episode-golden-vertical-slice-001")),
                document_id: "doc-golden-vertical-slice".into(),
                cause_ids: vec![CLAIM_ID.into()],
                effect_type: "canonical_projection_import".into(),
                outcome: "available_to_runtime_and_cli".into(),
                confidence: 0.99,
                experiment_id: Some("phase-03-golden-vertical-slice".into()),
                metadata: Some(serde_json::json!({
                    "source_receipt_id": runtime_receipt_id,
                    "forge_receipt_id": forge_receipt_id,
                })),
            }),
            semantics: None,
        },
    ];
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some("run-golden-vertical-slice-001".into()),
        direct_write: false,
        comparability_snapshot_version: Some("cmp-golden-vertical-slice-001".into()),
        exported_at: TIMESTAMP.into(),
    };
    let evidence_bundle = build_evidence_bundle(runtime_receipt_id, forge_receipt_id);
    let content_digest = ExportEnvelopeV3::compute_digest(
        "semantic-memory-forge",
        &scope_key,
        &records,
        Some(&export_meta),
        Some(&evidence_bundle),
    )
    .expect("canonical v3 digest");

    ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(ENVELOPE_ID),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest,
        source_authority: "semantic-memory-forge".into(),
        scope_key,
        trace_ctx: Some(trace_ctx.clone()),
        exported_at: TIMESTAMP.into(),
        export_meta: Some(export_meta),
        evidence_bundle: Some(evidence_bundle),
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

fn build_evidence_bundle(runtime_receipt_id: &str, forge_receipt_id: &str) -> EvidenceBundle {
    let mut bundle = EvidenceBundle::new(
        CausalQuestion {
            description: "Does the operator request reach the canonical runtime query path?".into(),
            unit_definition: "one phase 03 test run".into(),
        },
        TreatmentSpec {
            description: "route through receipt log, Forge envelope, bridge, memory, runtime, CLI"
                .into(),
            baseline_description: "no local AiDENs truth store".into(),
            paired_trials: false,
        },
        OutcomeSpec {
            description: "CLI returns the imported canonical projection with provenance".into(),
            measurement_method: "aidens-testkit golden_vertical_slice".into(),
            outcome_type: "binary".into(),
        },
        "phase_03_testkit",
        "0.1.0",
        0.99,
    );
    bundle.id = EvidenceBundleId::new("evidence-golden-vertical-slice-001");
    bundle.claim_ids = vec![ClaimId::new(CLAIM_ID)];
    bundle.comparability_snapshot_version = Some("cmp-golden-vertical-slice-001".into());
    bundle.metadata = Some(serde_json::json!({
        "runtime_tool_receipt_id": runtime_receipt_id,
        "forge_tool_receipt_id": forge_receipt_id,
        "canonical_owner": "semantic_memory_forge::EvidenceBundle"
    }));
    bundle
}

fn temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
