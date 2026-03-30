use federated_settlement::{
    LocalDissentRecordV1, ReplayRequirementV1, SettlementCaseV1, SharedDispositionV1,
    SharedViewDowngradeV1, SurfaceStatus, TreatyBundleV1, V25ConstitutionCitation,
};
use kernel_conformance::reference_interpreters::{
    interpret_divergence_or_suspension, interpret_generated_artifact_integrity,
    interpret_generated_companion_bundles, interpret_generated_surface_governance,
    interpret_settlement_admissibility, interpret_shared_replay,
};
use spec_execution::{
    HumanVetoBundleV1, MetaChallengeBundleV1, NormativeASTV1, NormativeAstNodeV1,
    ProofObligationSetV1, ProofObligationV1, SpecBundleV1,
};
use stack_ids::{
    CrossRuntimeEquivalenceBundleId, LocalDissentId, NormativeAstId, ProofObligationSetId,
    SettlementCaseId, SharedDispositionId, SharedViewDowngradeId, SpecBundleId, TreatyBundleId,
};

#[test]
fn settlement_reference_interpreter_preserves_local_dissent_on_downgrade() {
    let _treaty = TreatyBundleV1 {
        schema_version: "treaty_bundle_v1".into(),
        treaty_bundle_id: TreatyBundleId::new("treaty-1"),
        treaty_name: "bounded-treaty".into(),
        parties: vec![],
        canonical_owner: "federated-settlement".into(),
        publication_status: SurfaceStatus::AdvisoryOnly,
        advisory_only: true,
        non_authoritative: true,
    };
    let case = SettlementCaseV1 {
        schema_version: "settlement_case_v1".into(),
        settlement_case_id: SettlementCaseId::new("case-1"),
        treaty_bundle_id: TreatyBundleId::new("treaty-1"),
        equivalence_bundle_id: CrossRuntimeEquivalenceBundleId::new("equiv-1"),
        citation: V25ConstitutionCitation::default(),
        shared_disposition: SharedDispositionV1 {
            schema_version: "shared_disposition_v1".into(),
            shared_disposition_id: SharedDispositionId::new("disp-1"),
            disposition_label: "share".into(),
            treatment: "bounded".into(),
            advisory_only: false,
        },
        replay_requirements: vec![ReplayRequirementV1 {
            runtime_name: "runtime-b".into(),
            required: true,
            replay_available: false,
        }],
        local_dissent: vec![LocalDissentRecordV1 {
            schema_version: "local_dissent_record_v1".into(),
            local_dissent_id: LocalDissentId::new("dissent-1"),
            runtime_name: "runtime-a".into(),
            reason: "local authority keeps a narrower reading".into(),
            blocks_shared_settlement: false,
        }],
        advisory_only: false,
        non_admitted: false,
    };

    let receipt = interpret_settlement_admissibility(&case, "2026-03-14T00:00:00Z");
    assert!(receipt.downgrade.is_some());
    assert_eq!(receipt.local_dissent.len(), 1);

    let _example = SharedViewDowngradeV1 {
        schema_version: "shared_view_downgrade_v1".into(),
        shared_view_downgrade_id: SharedViewDowngradeId::new("downgrade-1"),
        reason: federated_settlement::DowngradeReason::MissingRequiredReplay,
        detail: "missing replay".into(),
        preserves_local_dissent: true,
    };

    let replay = interpret_shared_replay(&case, "2026-03-14T00:00:00Z");
    let (report, suspension) = interpret_divergence_or_suspension(
        &case,
        &replay,
        "bounded shared replay diverged from local reading",
        "2026-03-14T00:00:00Z",
    );
    assert!(report.requires_local_dissent_preservation);
    assert!(suspension.is_some());
}

#[test]
fn generated_artifact_interpreter_keeps_obligation_integrity_visible() {
    let spec = SpecBundleV1 {
        schema_version: "spec_bundle_v1".into(),
        spec_bundle_id: SpecBundleId::new("spec-1"),
        spec_name: "demo".into(),
        canonical_owner: "spec-execution".into(),
        publication_status: spec_execution::SurfaceStatus::AdvisoryOnly,
    };
    let ast = NormativeASTV1 {
        schema_version: "normative_ast_v1".into(),
        normative_ast_id: NormativeAstId::new("ast-1"),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        nodes: vec![NormativeAstNodeV1 {
            node_id: "node-1".into(),
            kind: "rule".into(),
            normative_text: "must preserve backpointers".into(),
        }],
        publication_status: spec_execution::SurfaceStatus::AdvisoryOnly,
    };
    let obligations = ProofObligationSetV1 {
        schema_version: "proof_obligation_set_v1".into(),
        proof_obligation_set_id: ProofObligationSetId::new("proofs-1"),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        obligations: vec![ProofObligationV1 {
            obligation_id: "obl-1".into(),
            description: "backpointer integrity".into(),
            satisfied: false,
            blocking: true,
        }],
    };

    let (bundle, receipt) = interpret_generated_artifact_integrity(
        &spec,
        &ast,
        &obligations,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );

    assert_eq!(bundle.spec_bundle_id, spec.spec_bundle_id);
    assert_eq!(receipt.unsatisfied_blocking_obligations, vec!["obl-1"]);
}

#[test]
fn generated_companion_interpreter_keeps_veto_and_challenge_baseline_visible() {
    let spec = SpecBundleV1 {
        schema_version: "spec_bundle_v1".into(),
        spec_bundle_id: SpecBundleId::new("spec-2"),
        spec_name: "demo-governance".into(),
        canonical_owner: "spec-execution".into(),
        publication_status: spec_execution::SurfaceStatus::AdvisoryOnly,
    };
    let ast = NormativeASTV1 {
        schema_version: "normative_ast_v1".into(),
        normative_ast_id: NormativeAstId::new("ast-2"),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        nodes: vec![NormativeAstNodeV1 {
            node_id: "node-2".into(),
            kind: "rule".into(),
            normative_text: "human veto must remain first-class".into(),
        }],
        publication_status: spec_execution::SurfaceStatus::AdvisoryOnly,
    };
    let obligations = ProofObligationSetV1 {
        schema_version: "proof_obligation_set_v1".into(),
        proof_obligation_set_id: ProofObligationSetId::new("proofs-2"),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        obligations: vec![ProofObligationV1 {
            obligation_id: "obl-2".into(),
            description: "generated surfaces preserve shared proof roots".into(),
            satisfied: true,
            blocking: true,
        }],
    };

    let bundles = interpret_generated_companion_bundles(
        &spec,
        &ast,
        &obligations,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );
    let veto = HumanVetoBundleV1 {
        schema_version: "human_veto_bundle_v1".into(),
        human_veto_bundle_id: stack_ids::HumanVetoBundleId::new("veto-1"),
        generated_schema_bundle_id: bundles.schema_bundle.generated_schema_bundle_id.clone(),
        reason: "human review rejects silent self-hosting promotion".into(),
    };
    let challenge = MetaChallengeBundleV1 {
        schema_version: "meta_challenge_bundle_v1".into(),
        meta_challenge_bundle_id: stack_ids::MetaChallengeBundleId::new("challenge-1"),
        generated_schema_bundle_id: bundles.schema_bundle.generated_schema_bundle_id.clone(),
        challenge_summary: "challenge requests explicit rollback visibility".into(),
        horizon_only: true,
    };

    let receipt = interpret_generated_surface_governance(
        &spec,
        &bundles,
        Some(&veto),
        Some(&challenge),
        "2026-03-14T00:00:00Z",
    );

    assert!(receipt.rollback_required);
    assert!(receipt.human_veto_bundle_id.is_some());
    assert!(receipt.meta_challenge_bundle_id.is_some());
}
