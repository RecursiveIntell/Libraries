use aidens_cli::{view_command, ViewCommand};
use aidens_memory_kit::{
    canonical_stack, memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
use knowledge_runtime::QueryWarning;
use semantic_memory::{ProjectionQuery, SearchResult};
use semantic_memory_forge::{
    ExportAuthority, ExportClaim, ExportEnvelopeV3, ExportRecord, ExportRecordV3, ForgeExportMeta,
    EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{ClaimId, ClaimVersionId, EntityId, EnvelopeId, Scope, ScopeKey, TraceCtx};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "aidens";
const RECORDED_FUTURE: &str = "9999-01-01T00:00:00Z";
const RECORDED_BEFORE_IMPORT: &str = "1970-01-01T00:00:00Z";

#[test]
fn bitemporal_asof_query() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-05-bitemporal-asof");
    let memory_root = root.join("memory");
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let trace_ctx = TraceCtx::generate();
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&memory_root),
            runtime_config_for_namespace(NAMESPACE),
        )?;
        let envelope = forge_envelope(
            "env-phase05-bitemporal-001",
            &trace_ctx,
            vec![
                claim_export(
                    "claim-phase05-memory-runtime",
                    "claim-version-phase05-memory-runtime-old",
                    "phase five memory status old local authority removed before hardening",
                    "2026-01-01T00:00:00Z",
                    Some("2026-03-01T00:00:00Z"),
                ),
                claim_export(
                    "claim-phase05-memory-runtime",
                    "claim-version-phase05-memory-runtime-new",
                    "phase five memory status new canonical semantic-memory knowledge-runtime hardening",
                    "2026-03-01T00:00:00Z",
                    None,
                ),
            ],
        );

        envelope.validate()?;
        let imported = adapter.import_forge_export(&envelope).await?;
        assert_eq!(imported.status, "complete");

        let scope = Scope::new(NAMESPACE);
        let (old_results, old_trace) = adapter
            .query_temporal(
                "phase five memory status",
                Some(&scope),
                "2026-02-01T00:00:00Z",
                RECORDED_FUTURE,
            )
            .await?;
        assert!(contains_content(
            &old_results,
            "old local authority removed"
        ));
        assert!(!contains_content(
            &old_results,
            "new canonical semantic-memory"
        ));
        assert_eq!(
            old_trace.valid_as_of.as_deref(),
            Some("2026-02-01T00:00:00Z")
        );
        assert_eq!(old_trace.recorded_as_of.as_deref(), Some(RECORDED_FUTURE));
        assert_eq!(old_trace.temporal_mode.as_deref(), Some("exact"));
        assert!(!old_trace.has_temporal_downgrade());

        let (new_results, new_trace) = adapter
            .query_temporal(
                "phase five memory status",
                Some(&scope),
                "2026-04-01T00:00:00Z",
                RECORDED_FUTURE,
            )
            .await?;
        assert!(contains_content(
            &new_results,
            "new canonical semantic-memory"
        ));
        assert!(!contains_content(
            &new_results,
            "old local authority removed"
        ));
        assert_eq!(
            new_trace.valid_as_of.as_deref(),
            Some("2026-04-01T00:00:00Z")
        );
        assert_eq!(new_trace.recorded_as_of.as_deref(), Some(RECORDED_FUTURE));

        let (pre_import_results, pre_import_trace) = adapter
            .query_temporal(
                "phase five memory status",
                Some(&scope),
                "2026-04-01T00:00:00Z",
                RECORDED_BEFORE_IMPORT,
            )
            .await?;
        assert!(pre_import_results.is_empty());
        assert_eq!(
            pre_import_trace.recorded_as_of.as_deref(),
            Some(RECORDED_BEFORE_IMPORT)
        );

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    let cli_output = view_command(ViewCommand::Query {
        memory_store: memory_root.to_string_lossy().into_owned(),
        view_mode: "temporal".into(),
        query: "phase five memory status".into(),
        subject: None,
        predicate: None,
        valid_at: Some("2026-04-01T00:00:00Z".parse()?),
        recorded_at: Some(RECORDED_FUTURE.parse()?),
        aliases: Vec::new(),
        allow_alias_expansion: false,
        allow_timeless_fallback: false,
    })?;
    let cli_json: serde_json::Value = serde_json::from_str(&cli_output)?;
    assert_eq!(cli_json["kind"], "canonical-runtime-view");
    assert_eq!(cli_json["canonical_owner"], "knowledge-runtime");
    assert_eq!(cli_json["memory_owner"], "semantic-memory");
    assert_eq!(cli_json["trace"]["valid_as_of"], "2026-04-01T00:00:00Z");
    assert_eq!(cli_json["trace"]["recorded_as_of"], RECORDED_FUTURE);
    assert!(cli_output.contains("new canonical semantic-memory"));
    assert!(!cli_output.contains("old local authority removed"));

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn import_atomicity() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-05-import-atomicity");
    let memory_root = root.join("memory");
    let trace_ctx = TraceCtx::generate();
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;

    let baseline = forge_envelope(
        "env-phase05-atomic-baseline",
        &trace_ctx,
        vec![claim_export(
            "claim-phase05-atomic-baseline",
            "claim-version-phase05-atomic-baseline",
            "phase five atomic baseline canonical semantic-memory row",
            "2026-01-01T00:00:00Z",
            None,
        )],
    );
    let baseline_batch = canonical_stack::transform_forge_export(&baseline)?;
    canonical_stack::import_projection_batch(adapter.store(), &baseline_batch).await?;

    let failing = forge_envelope(
        "env-phase05-atomic-failing",
        &trace_ctx,
        vec![
            claim_export(
                "claim-phase05-atomic-extra",
                "claim-version-phase05-atomic-extra",
                "phase five atomic extra must not partially commit",
                "2026-01-01T00:00:00Z",
                None,
            ),
            claim_export(
                "claim-phase05-atomic-baseline",
                "claim-version-phase05-atomic-conflict",
                "phase five atomic conflict must fail through semantic-memory",
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

    let baseline_rows = query_claim(adapter.store(), "claim-phase05-atomic-baseline").await?;
    assert_eq!(baseline_rows.len(), 1);
    assert_eq!(
        baseline_rows[0].claim_version_id,
        ClaimVersionId::new("claim-version-phase05-atomic-baseline")
    );
    let extra_rows = query_claim(adapter.store(), "claim-phase05-atomic-extra").await?;
    assert!(
        extra_rows.is_empty(),
        "semantic-memory import must not commit non-conflicting rows from a failed batch"
    );

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn query_widening_disclosure() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-05-query-widening");
    let memory_root = root.join("memory");
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;
    let scope = Scope::new(NAMESPACE);

    let (_results, trace) = adapter
        .query_temporal(
            "phase five memory status",
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

async fn query_claim(
    store: &canonical_stack::CanonicalMemoryStore,
    claim_id: &str,
) -> Result<Vec<semantic_memory::ProjectionClaimVersion>, semantic_memory::MemoryError> {
    let mut query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    query.claim_id = Some(ClaimId::new(claim_id));
    store.query_claim_versions(query).await
}

fn contains_content(results: &[SearchResult], needle: &str) -> bool {
    results.iter().any(|result| result.content.contains(needle))
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
            subject_entity_id: EntityId::new("entity-phase05-memory-runtime"),
            predicate: "phase05_memory_runtime_state".into(),
            object_anchor: serde_json::json!("canonical-library-owned"),
            valid_from: Some(valid_from.into()),
            valid_to: valid_to.map(str::to_string),
            confidence: 0.99,
            content: content.into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                "canonical_owner": "semantic-memory-forge",
                "phase": "05"
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
        comparability_snapshot_version: Some("cmp-phase05-memory-runtime".into()),
        exported_at: "2026-04-28T12:00:00Z".into(),
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
        exported_at: "2026-04-28T12:00:00Z".into(),
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

fn temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
