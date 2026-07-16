use aidens_contracts::{
    ArtifactId, CanonicalKernelStopReason, CompiledRegionGraphV1, ConvergenceReportV1,
    DisplayDigestV1, InvariantBudgetV1, KernelResidualReportV1, KernelRunDisplayReportV1,
    KernelStopRuleReportV1, KernelSyndromeReportV1, OracleAgreementV1, OracleSliceRequestV1,
    RegionBoundaryMessageV1, RegionBoundaryReceiptDispositionV1, RegionBoundaryReceiptV1,
    RegionContractV1, RegionGraphKindV1, RegionNodeKindV1, RegionNodeV1, RemovalFrontierV1,
    SupportCoreV1,
};
use aidens_repair_kit::{
    canonical_stack as repair_stack, BoundaryRepairAdmissionDisposition, CanonicalRepairAdapter,
};

#[test]
fn minimal_v11b_region_seed_blocks_wrong_graphs_and_discloses_boundary_outcomes() {
    for graph_kind in [RegionGraphKindV1::Retrieval, RegionGraphKindV1::Control] {
        let node = RegionNodeV1::new(RegionNodeKindV1::Claim, format!("{graph_kind}:node"), None);
        let graph_id = ArtifactId::new(format!("graph:phase10-{graph_kind}"));
        let region = RegionContractV1::new(
            graph_id.clone(),
            graph_kind,
            ArtifactId::new(format!("region:phase10-{graph_kind}")),
            vec![node.node_id.clone()],
            Vec::new(),
            Vec::new(),
            4,
        );
        let graph = CompiledRegionGraphV1::new(
            graph_id,
            graph_kind,
            Some(RegionGraphKindV1::Storage),
            4,
            vec![node],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![region],
            Vec::new(),
            Vec::new(),
        );

        assert!(!graph_kind.can_execute_kernel());
        assert!(!graph.right_graph_law_satisfied);
        assert!(graph
            .reason_codes
            .contains(&"right-graph-law-blocked".into()));
    }

    let message = RegionBoundaryMessageV1::seed(
        ArtifactId::new("region:phase10-source"),
        ArtifactId::new("region:phase10-destination"),
        "residual",
        ArtifactId::new("residual:phase10"),
        DisplayDigestV1::for_json_value(&serde_json::json!({"residual": 0.2})),
    );
    let accepted = RegionBoundaryReceiptV1::seed(&message, true);
    let rejected = RegionBoundaryReceiptV1::seed(&message, false);
    let quarantined = RegionBoundaryReceiptV1::quarantined(&message, "missing replay witness");

    assert_eq!(
        accepted.disposition,
        RegionBoundaryReceiptDispositionV1::Accepted
    );
    assert_eq!(
        rejected.disposition,
        RegionBoundaryReceiptDispositionV1::Rejected
    );
    assert_eq!(
        quarantined.disposition,
        RegionBoundaryReceiptDispositionV1::Quarantined
    );
    assert!(!accepted.can_admit_runtime_payload());
    assert!(!rejected.can_admit_runtime_payload());
    assert!(!quarantined.can_admit_runtime_payload());

    let repair = CanonicalRepairAdapter.boundary_repair_record(
        repair_stack::BoundaryArtifactKind::ControlReceipt,
        "control_receipt_v1",
        "field_normalization",
        "$.actor",
        Some(serde_json::json!("")),
        serde_json::json!("operator"),
        "local repair before global rebuild seed",
    );
    let repair_receipt = CanonicalRepairAdapter.admit_boundary_repair_record(&repair);
    assert_eq!(
        repair_receipt.disposition,
        BoundaryRepairAdmissionDisposition::Accepted
    );
}

#[test]
fn minimal_v11b_region_failure_slice_is_degraded_and_non_promotable() {
    let graph_id = ArtifactId::new("graph:phase10-minimal");
    let region_id = ArtifactId::new("region:phase10-minimal");
    let accepted_claim = ArtifactId::new("claim:phase10-accepted");
    let node = RegionNodeV1::new(
        RegionNodeKindV1::Claim,
        "accepted claim",
        Some(accepted_claim.clone()),
    );
    let region = RegionContractV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        region_id.clone(),
        vec![node.node_id.clone()],
        Vec::new(),
        Vec::new(),
        4,
    );
    let graph = CompiledRegionGraphV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        Some(RegionGraphKindV1::Storage),
        4,
        vec![node],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![region.clone()],
        vec![accepted_claim.clone()],
        Vec::new(),
    );
    let stop = KernelStopRuleReportV1::new(
        CanonicalKernelStopReason::BudgetExhausted,
        1,
        1,
        0.1,
        0.5,
        0.2,
    );
    let residual = KernelResidualReportV1::new(
        graph_id.clone(),
        region_id.clone(),
        1,
        0.3,
        0.2,
        0.1,
        stop.clone(),
    );
    let convergence = ConvergenceReportV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::BudgetExhausted,
        1,
        1,
        0.1,
        0.5,
        residual.current_value,
        vec![residual.residual_id.clone()],
        stop,
        false,
    );
    let syndrome = KernelSyndromeReportV1::contradiction(
        graph_id.clone(),
        region_id.clone(),
        ArtifactId::new("witness:phase10"),
        vec![accepted_claim.clone()],
    );
    let oracle = OracleSliceRequestV1::new(
        graph_id,
        region_id,
        region.node_ids,
        4,
        DisplayDigestV1::for_json_value(&serde_json::json!({"approx": true})),
        DisplayDigestV1::for_json_value(&serde_json::json!({"exact": true})),
        Some(0.25),
    );
    let run = KernelRunDisplayReportV1::new(
        &graph,
        &convergence,
        std::slice::from_ref(&syndrome),
        std::slice::from_ref(&oracle),
    );
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        vec![run.receipt_id.clone()],
        Vec::new(),
        Vec::new(),
    );
    let frontier = RemovalFrontierV1::new(
        &support,
        vec![accepted_claim],
        Vec::new(),
        vec![run.receipt_id.clone()],
        &InvariantBudgetV1::full_history(),
    );

    assert!(graph.is_bounded_region_graph());
    assert!(!graph.can_claim_active_v11b_runtime());
    assert!(residual.blocks_promotion_as_exact());
    assert!(syndrome.blocks_promotion_as_exact());
    assert!(convergence.blocks_promotion_as_exact());
    assert!(run.degraded);
    assert!(!run.can_promote_as_exact_seed_result());
    assert_eq!(oracle.agreement, OracleAgreementV1::BoundedDisagreement);
    assert!(oracle.has_bounded_semantic_diff());
    assert!(frontier.is_blocked());
}
