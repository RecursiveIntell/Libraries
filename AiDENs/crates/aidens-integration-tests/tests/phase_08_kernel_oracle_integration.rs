use aidens_kernel_kit::{
    CanonicalKernelAdapter, CompilerPolicy, ExecutionBudget, ExecutionStopReason, OracleMode,
};
use forge_memory_bridge::{transform_envelope_v3, ProjectionImportBatchV3};
use recursive_kernel_core::ArtifactAuthorityClass;
use semantic_memory_forge::{
    ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV3,
    ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
    ProjectionVisibilityClass, EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{AssertionGroupId, ClaimFamilyId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey};
use std::any::TypeId;

fn rich_kernel_batch(namespace: &str, claim_suffixes: &[&str]) -> ProjectionImportBatchV3 {
    let scope = ScopeKey::namespace_only(namespace);
    let records = claim_suffixes
        .iter()
        .enumerate()
        .map(|(index, suffix)| ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some(ClaimVersionId::new(format!("claim-version-{suffix}"))),
                subject_entity_id: EntityId::new(format!("entity-{index}")),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("kernel-result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: format!("claim {suffix} supports kernel-result"),
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
        exported_at: "2026-04-29T00:00:00Z".into(),
    };
    let digest =
        ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .expect("canonical forge digest");
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(format!("env-{namespace}")),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-04-29T00:00:00Z".into(),
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

    transform_envelope_v3(&envelope).expect("canonical bridge transform")
}

#[test]
fn kernel_exact_small_slice() {
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
    let batch = rich_kernel_batch("phase-08-exact", &["a", "b"]);
    let compiled = adapter.compile_projection_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "phase-08".into(),
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
        .expect("small verified slice should have bounded exact oracle");
    let conservative = adapter.evaluate_conservative(&compiled);

    assert_eq!(exact.mode, OracleMode::ExactBounded);
    assert!(exact.supported);
    assert_eq!(
        exact.satisfied_constraint_count,
        conservative.satisfied_constraint_count
    );
    assert!(!exact.selected_region_ids.is_empty());
    assert!(
        adapter.conformance_gate_ids().contains(&"CF-O1"),
        "adapter must expose canonical conformance gates"
    );
}

#[test]
fn loopy_nonconvergence_degrades() {
    let adapter = CanonicalKernelAdapter;
    let batch = rich_kernel_batch("phase-08-loop", &["a", "b"]);
    let compiled = adapter.compile_projection_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "phase-08".into(),
            include_hyperedges: true,
        },
    );

    assert!(!compiled.hyperedges.is_empty());

    let report = adapter.execute_message_passing(&compiled, 1);
    assert_eq!(report.stop_reason, ExecutionStopReason::MaxIterations);
    assert!(!report.convergence_report.converged);
    assert!(report.convergence_report.escalated);
    assert_eq!(
        report.convergence_report.governance.escalation_rule,
        "emit_failure_artifact_on_nonconvergence"
    );
    assert_eq!(
        report.authority_class(),
        ArtifactAuthorityClass::NonAuthoritativeDerived
    );

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
}
