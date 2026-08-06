//! Recursive kernel pipeline demo — end-to-end using correct V3 envelope API.
//! Mirrors the kernel-oracles test's `rich_batch()` pattern exactly.

use constraint_compiler::{compile_batch, CompilerPolicy};
use forge_memory_bridge::transform_envelope_v3;
use kernel_execution::{execute_residual_correction, schedule_execution, ExecutionBudget};
use kernel_oracles::{evaluate_conservative, evaluate_exact_bounded, minimal_perturbation_witness};
use semantic_memory_forge::{
    ConstraintSeedKind, ExportClaim, ExportConfidenceClass, ExportEnvelopeV3, ExportRecord,
    ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta, ProjectionVisibilityClass,
    EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{AssertionGroupId, ClaimFamilyId, EnvelopeId, ScopeKey};

fn main() {
    let batch = build_demo_batch();

    let policy = CompilerPolicy {
        policy_version: "demo-v1".into(),
        include_hyperedges: true,
    };
    let compiled = compile_batch(&batch, &policy);
    println!(
        "COMPILED: {} nodes {} hyperedges {} constraints {} regions",
        compiled.nodes.len(),
        compiled.hyperedges.len(),
        compiled.constraints.len(),
        compiled.regions.len()
    );
    for deg in &compiled.degradations {
        println!("  degradation: {:?}", deg);
    }
    for oc in &compiled.oracle_candidates {
        println!(
            "  oracle: {} ({} nodes)",
            oc.oracle_slice_id.as_str(),
            oc.node_ids.len()
        );
    }

    let budget = ExecutionBudget {
        max_iterations: 5,
        max_messages: 256,
        max_nodes: 64,
        allow_repair: true,
    };
    let sched = schedule_execution(&compiled, &budget);
    let exec = &sched.execution;
    println!(
        "EXECUTED: {} iter {:?} synd={} wit={} cert={}",
        exec.iteration_count,
        exec.stop_reason,
        exec.syndromes.len(),
        exec.witnesses.len(),
        exec.certificates.len()
    );
    for b in &exec.node_beliefs {
        println!("  belief {}={}μ", b.node_id, b.belief_micros);
    }
    for s in &exec.syndromes {
        println!(
            "  syndrome {} blocked={}",
            s.syndrome_id.as_str(),
            s.blocked_by_degradation
        );
    }
    for w in &exec.witnesses {
        println!(
            "  witness {} belief={}μ constraints={}",
            w.node_id,
            w.belief_micros,
            w.supporting_constraint_ids.len()
        );
    }
    if let Some(ref cal) = exec.calibration_report {
        println!(
            "  calib: degraded={} nuisance={}",
            cal.degraded,
            cal.nuisance_node_ids.len()
        );
    }

    let corr = execute_residual_correction(&compiled, 3);
    println!(
        "CORRECTED: {} iter {:?} converged={}",
        corr.iteration_count, corr.stop_reason, corr.convergence_report.converged
    );

    if let Some(exact) = evaluate_exact_bounded(&compiled) {
        println!(
            "ORACLE exact: supported={} constraints={} regions={}",
            exact.supported,
            exact.satisfied_constraint_count,
            exact.selected_region_ids.len()
        );
    }
    let cons = evaluate_conservative(&compiled);
    println!(
        "ORACLE conservative: supported={} regions={}",
        cons.supported,
        cons.selected_region_ids.len()
    );

    if let Some(first) = compiled.nodes.first() {
        let refut = minimal_perturbation_witness(&compiled, &first.node_id, 4);
        println!(
            "REFUTE {}: mode={:?} supported={} budget={}",
            first.node_id, refut.mode, refut.baseline_supported, refut.searched_budget
        );
        match refut.outcome {
            kernel_oracles::OracleRefutationOutcome::FlipWitness { removed_node_ids } => {
                println!("  FLIP: removed {:?}", removed_node_ids)
            }
            kernel_oracles::OracleRefutationOutcome::NoFlipFound { searched_budget } => {
                println!("  NO FLIP: searched {}", searched_budget)
            }
            kernel_oracles::OracleRefutationOutcome::NotApplicable { reason } => {
                println!("  N/A: {}", reason)
            }
        }
    }

    println!("advisory_only={} — PIPELINE COMPLETE", exec.advisory_only);
}

fn build_demo_batch() -> forge_memory_bridge::ProjectionImportBatchV3 {
    let scope = ScopeKey::namespace_only("kernel-demo");
    let records = vec![
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-demo-1".into()),
                subject_entity_id: stack_ids::EntityId::new("entity-alpha"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("convergence"),
                valid_from: None,
                valid_to: None,
                confidence: 0.95,
                content: "entity-alpha supports convergence".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(semantics()),
        },
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-demo-2".into()),
                subject_entity_id: stack_ids::EntityId::new("entity-beta"),
                predicate: "contradicts".into(),
                object_anchor: serde_json::json!("drift"),
                valid_from: None,
                valid_to: None,
                confidence: 0.72,
                content: "entity-beta reports drift".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(semantics()),
        },
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-demo-3".into()),
                subject_entity_id: stack_ids::EntityId::new("entity-gamma"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("repair"),
                valid_from: None,
                valid_to: None,
                confidence: 0.88,
                content: "entity-gamma supports repair".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(semantics()),
        },
    ];
    let export_meta = ForgeExportMeta {
        authority: semantic_memory_forge::ExportAuthority::Forge,
        run_id: Some("run-demo-1".into()),
        direct_write: false,
        comparability_snapshot_version: None,
        exported_at: "2026-08-03T00:00:00Z".into(),
    };
    let digest =
        ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .unwrap();
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new("envelope-demo-001"),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-08-03T00:00:00Z".into(),
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
    transform_envelope_v3(&envelope).unwrap()
}

fn semantics() -> ExportRecordSemanticsV3 {
    ExportRecordSemanticsV3 {
        claim_family_id: Some(ClaimFamilyId::new("forge_verification")),
        assertion_group_id: Some(AssertionGroupId::new("group-supports-demo")),
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
    }
}
