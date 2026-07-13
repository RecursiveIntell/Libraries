use aidens_governance_kit::{canonical_stack as governance_stack, CanonicalGovernanceAdapter};
use aidens_kernel_kit::{
    CanonicalKernelAdapter, CompilerPolicy, ExecutionBudget, ExecutionStopReason, OracleMode,
};
use aidens_memory_kit::{
    canonical_stack as memory_stack, memory_config_for_root, runtime_config_for_namespace,
    CanonicalMemoryAdapter,
};
use aidens_repair_kit::{canonical_stack as repair_stack, CanonicalRepairAdapter};
use forge_memory_bridge::ProjectionImportBatchV3;
use recursive_kernel_core::ArtifactAuthorityClass;
use semantic_memory::{ProjectionQuery, SearchSource};
use semantic_memory_forge::{
    ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV3,
    ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
    ProjectionVisibilityClass, RetractionRecordV1, EXPORT_ENVELOPE_V3_SCHEMA,
    RETRACTION_RECORD_V1_SCHEMA,
};
use stack_ids::{
    AssertionGroupId, AttemptId, ClaimFamilyId, ClaimId, ClaimVersionId, EntityId, EnvelopeId,
    RetractionRecordId, Scope, ScopeKey, TraceCtx,
};
use std::any::TypeId;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "aidens-phase07";
const TIMESTAMP: &str = "2026-04-29T12:00:00Z";

#[tokio::test]
async fn memory_adapter_delegates_forge_bridge_memory_runtime(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        TypeId::of::<aidens_memory_kit::ProjectionImportBatchV3>(),
        TypeId::of::<forge_memory_bridge::ProjectionImportBatchV3>()
    );
    assert_eq!(
        TypeId::of::<aidens_memory_kit::ProjectionImportResult>(),
        TypeId::of::<semantic_memory::ProjectionImportResult>()
    );
    assert_eq!(
        TypeId::of::<aidens_memory_kit::CanonicalMemoryConfig>(),
        TypeId::of::<semantic_memory::MemoryConfig>()
    );

    let root = temp_root("phase-07-memory-adapter");
    let memory_root = root.join("memory");
    let trace_ctx = TraceCtx::from_trace_id("trace-phase07-memory");
    let envelope = forge_envelope(
        "env-phase07-memory",
        &trace_ctx,
        vec![claim_export(
            "claim-phase07-memory",
            "claim-version-phase07-memory",
            "phase seven canonical adapter proof reaches knowledge runtime",
            "phase07_delegates_to",
            "forge bridge memory runtime",
        )],
    );
    envelope.validate()?;

    let batch = memory_stack::transform_forge_export(&envelope)?;
    assert_eq!(
        batch.schema_version,
        memory_stack::PROJECTION_IMPORT_BATCH_V3_SCHEMA
    );
    assert_eq!(batch.source_envelope_id, envelope.envelope_id);
    assert_eq!(
        batch.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );

    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(NAMESPACE),
    )?;
    let import = memory_stack::import_projection_batch(adapter.store(), &batch).await?;
    assert_eq!(import.status, "complete");
    assert_eq!(import.record_count, 1);

    let mut projection_query = ProjectionQuery::new(ScopeKey::namespace_only(NAMESPACE));
    projection_query.claim_id = Some(ClaimId::new("claim-phase07-memory"));
    let rows = adapter
        .store()
        .query_claim_versions(projection_query)
        .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].claim_version_id,
        ClaimVersionId::new("claim-version-phase07-memory")
    );
    assert_eq!(rows[0].trace_id.as_deref(), Some("trace-phase07-memory"));

    let scope = Scope::new(NAMESPACE);
    let (results, trace) = adapter
        .query("phase seven canonical adapter proof", Some(&scope))
        .await?;
    assert_eq!(trace.scope, ScopeKey::namespace_only(NAMESPACE));
    assert!(results.iter().any(|result| {
        result
            .content
            .contains("phase seven canonical adapter proof")
            && matches!(
                &result.source,
                SearchSource::Projection {
                    projection_kind,
                    source_envelope_id,
                    ..
                } if projection_kind == "claim_version" && source_envelope_id == "env-phase07-memory"
            )
    }));

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn kernel_adapter_delegates_compiler_execution_oracle_conformance() {
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::CompileOutput>(),
        TypeId::of::<constraint_compiler::CompileOutput>()
    );
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::ExecutionReport>(),
        TypeId::of::<kernel_execution::ExecutionReport>()
    );
    assert_eq!(
        TypeId::of::<aidens_kernel_kit::OracleAssessment>(),
        TypeId::of::<kernel_oracles::OracleAssessment>()
    );

    let adapter = CanonicalKernelAdapter;
    let operators = adapter.canonical_operator_metadata();
    assert!(operators.iter().any(|operator| {
        operator.operator_id.as_str() == recursive_kernel_core::CONSTRAINT_COMPILER_OPERATOR_ID
    }));
    assert!(operators.iter().any(|operator| {
        operator.operator_id.as_str()
            == recursive_kernel_core::RECURSIVE_MESSAGE_PASSING_OPERATOR_ID
    }));

    let batch = rich_kernel_batch("phase07-kernel", &["a", "b"]);
    let compiled = adapter.compile_projection_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "phase07".into(),
            include_hyperedges: true,
        },
    );
    assert!(compiled.degradations.is_empty());
    assert!(!compiled.constraints.is_empty());
    assert!(!compiled.oracle_candidates.is_empty());

    let execution = adapter.execute_acyclic(&compiled);
    assert_eq!(
        execution.stop_reason,
        ExecutionStopReason::AcyclicCompletion
    );
    assert_eq!(
        execution.authority_class(),
        ArtifactAuthorityClass::NonAuthoritativeDerived
    );

    let exact = adapter
        .evaluate_exact_bounded(&compiled)
        .expect("small canonical slice has exact bounded oracle");
    assert_eq!(exact.mode, OracleMode::ExactBounded);
    assert!(exact.supported);
    assert!(!exact.selected_region_ids.is_empty());

    let scheduled = adapter.schedule_execution(
        &compiled,
        &ExecutionBudget {
            max_iterations: 1,
            max_messages: 8,
            max_nodes: 1,
            allow_repair: false,
        },
    );
    assert_eq!(
        scheduled.execution.stop_reason,
        ExecutionStopReason::BudgetExhausted
    );
    assert_eq!(
        scheduled.degraded_reason.as_deref(),
        Some("budget_exhausted")
    );
    assert!(adapter.conformance_gate_ids().contains(&"CF-O1"));
}

#[test]
fn verification_adapter_delegates_control_policy_calibration_adjudication(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        TypeId::of::<aidens_governance_kit::CheckPlan>(),
        TypeId::of::<verification_control::CheckPlan>()
    );
    assert_eq!(
        TypeId::of::<governance_stack::PolicySnapshot>(),
        TypeId::of::<verification_policy::PolicySnapshot>()
    );
    assert_eq!(
        TypeId::of::<governance_stack::CalibrationSnapshot>(),
        TypeId::of::<verification_calibration::CalibrationSnapshot>()
    );
    assert_eq!(
        TypeId::of::<governance_stack::AdjudicationResult>(),
        TypeId::of::<verification_adjudication::AdjudicationResult>()
    );

    let adapter = CanonicalGovernanceAdapter;
    let case = adapter.claim_version_case(
        NAMESPACE,
        ClaimVersionId::new("claim-version-phase07-verification"),
        TraceCtx::from_trace_id("trace-phase07-verification"),
        AttemptId::new("attempt-phase07-verification"),
        TIMESTAMP,
    );
    let plan = adapter.check_plan(
        &case,
        governance_stack::CheckMethod::ExactBoundedOracle,
        vec!["exact-bounded-oracle".into()],
        governance_stack::PromotionClass::P2,
        governance_stack::ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "Phase 07 canonical verification adapter proof",
        serde_json::json!({"phase": "07"}),
    );
    let attempt = adapter.completed_attempt(
        &case,
        &plan,
        governance_stack::VerificationAttemptState::Succeeded,
        TIMESTAMP,
        "2026-04-29T12:00:01Z",
        Some("exact-bounded-oracle-succeeded".into()),
    );
    let control = adapter.control_receipt_for_attempt(
        &case,
        &plan,
        &attempt,
        true,
        serde_json::json!({"target_key": case.region.target_key}),
    );
    assert_eq!(
        control.schema_version,
        verification_control::CONTROL_RECEIPT_V1_SCHEMA
    );
    assert!(control.validate().is_ok());

    let policy = governance_stack::PolicySnapshot::permissive("phase07-policy", TIMESTAMP);
    let policy_decision = adapter.evaluate_policy(&policy, &case, &plan, &[], false, false);
    assert!(policy_decision.validate().is_ok());
    assert!(policy_decision.promotion_allowed);

    let calibration =
        adapter.calibration_snapshot(&case, TIMESTAMP, true, true, 500_000, 100_000, Vec::new());
    assert!(!calibration.forces_advisory_only);
    let adjudication = adapter.adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control,
        &policy_decision,
        &calibration,
        false,
        false,
        false,
    );
    assert_eq!(
        adjudication.disposition,
        governance_stack::VerificationDisposition::EligibleForPromotion
    );
    assert!(adjudication.promotion_decision.promotable);
    adjudication.promotion_decision.validate()?;
    Ok(())
}

#[test]
fn repair_adapter_delegates_boundary_repair_and_forge_retraction(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        TypeId::of::<aidens_repair_kit::CanonicalBoundaryRepairRecord>(),
        TypeId::of::<verification_control::BoundaryRepairRecord>()
    );
    assert_eq!(
        TypeId::of::<aidens_repair_kit::CanonicalRetractionRecordV1>(),
        TypeId::of::<semantic_memory_forge::RetractionRecordV1>()
    );

    let repair = CanonicalRepairAdapter.boundary_repair_record(
        repair_stack::BoundaryArtifactKind::ControlReceipt,
        verification_control::CONTROL_RECEIPT_V1_SCHEMA,
        "phase07-backpointer-preservation",
        "$.details.phase07",
        Some(serde_json::json!(null)),
        serde_json::json!({
            "canonical_owner": "verification-control",
            "proof": "phase07"
        }),
        "Phase 07 proves repair records are minted by verification-control",
    );
    assert_eq!(
        repair.schema_version,
        repair_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA
    );
    assert_eq!(repair.repair_kind, "phase07-backpointer-preservation");

    let retraction = RetractionRecordV1 {
        schema_version: RETRACTION_RECORD_V1_SCHEMA.into(),
        retraction_record_id: RetractionRecordId::new("retraction-phase07-proof"),
        claim_id: ClaimId::new("claim-phase07-repair"),
        retracted_claim_version_id: ClaimVersionId::new("claim-version-phase07-repair"),
        superseded_by_claim_version_id: None,
        effective_recorded_at: TIMESTAMP.into(),
        reason: format!("canonical repair record {}", repair.repair_record_id),
        cascade_required: true,
        delta_summary: Some(
            serde_json::json!({
                "boundary_repair_record_id": repair.repair_record_id.to_string(),
                "canonical_owner": "semantic-memory-forge"
            })
            .to_string(),
        ),
    };
    CanonicalRepairAdapter.validate_retraction(&retraction)?;
    assert_eq!(retraction.schema_version, RETRACTION_RECORD_V1_SCHEMA);
    Ok(())
}

fn rich_kernel_batch(namespace: &str, claim_suffixes: &[&str]) -> ProjectionImportBatchV3 {
    let scope = ScopeKey::namespace_only(namespace);
    let records = claim_suffixes
        .iter()
        .enumerate()
        .map(|(index, suffix)| ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: Some(ClaimId::new(format!("claim-phase07-{suffix}"))),
                claim_version_id: Some(ClaimVersionId::new(format!(
                    "claim-version-phase07-{suffix}"
                ))),
                subject_entity_id: EntityId::new(format!("entity-phase07-{index}")),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("phase07-kernel-result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: format!("phase07 claim {suffix} supports kernel-result"),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new(format!("family-{namespace}"))),
                assertion_group_id: Some(AssertionGroupId::new(format!("group-{namespace}"))),
                relation_group_id: None,
                joint_evidence_group_id: None,
                constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                treatment_hint: None,
                outcome_hint: None,
                confounder_hint: None,
                instrument_hint: None,
                effect_modifier_hint: None,
                contradiction_candidate_group_id: None,
                mutual_exclusion_group_id: None,
                comparability_snapshot_version: None,
                nuisance_snapshot: None,
                projection_visibility_class: ProjectionVisibilityClass::Standard,
                export_confidence_class: ExportConfidenceClass::Verified,
                derivation_seed_ids: vec![],
                review_priority_hint: None,
            }),
        })
        .collect::<Vec<_>>();
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some(format!("run-{namespace}")),
        direct_write: false,
        comparability_snapshot_version: None,
        exported_at: TIMESTAMP.into(),
    };
    let digest = ExportEnvelopeV3::compute_digest(
        "semantic-memory-forge",
        &scope,
        &records,
        Some(&export_meta),
        None,
    )
    .expect("canonical forge digest");
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(format!("env-{namespace}")),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "semantic-memory-forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: TIMESTAMP.into(),
        export_meta: Some(export_meta),
        evidence_bundle: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records,
    };

    forge_memory_bridge::transform_envelope_v3(&envelope).expect("canonical bridge transform")
}

fn claim_export(
    claim_id: &str,
    claim_version_id: &str,
    content: &str,
    predicate: &str,
    object_anchor: &str,
) -> ExportRecordV3 {
    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(claim_id)),
            claim_version_id: Some(ClaimVersionId::new(claim_version_id)),
            subject_entity_id: EntityId::new("entity-phase07-memory"),
            predicate: predicate.into(),
            object_anchor: serde_json::json!(object_anchor),
            valid_from: Some("2026-04-29T00:00:00Z".into()),
            valid_to: None,
            confidence: 0.99,
            content: content.into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                "canonical_owner": "semantic-memory-forge",
                "phase": "07"
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
        comparability_snapshot_version: Some("cmp-phase07-adapter-proof".into()),
        exported_at: TIMESTAMP.into(),
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
        exported_at: TIMESTAMP.into(),
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
