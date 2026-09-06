use agent_graph::evidence_join::{
    BranchEvidenceCandidate, ClaimProjection, ClaimScope, EvidenceJoin, EvidenceJoinError,
    EvidenceJoinInput, EvidenceJoinLimits, EvidenceJoinOracle, EvidenceJoinSeed,
    EvidenceRefProjection, EvidenceScope, JoinBudgetProjection, JoinDisposition, JoinReasonCode,
    OracleDecision, RequiredCheckProjection,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Default)]
struct TestOracle {
    checks: BTreeMap<String, OracleDecision>,
    supports: BTreeMap<String, OracleDecision>,
    obligations: BTreeMap<String, OracleDecision>,
    counterexamples: BTreeMap<String, OracleDecision>,
    epoch_compatibility: OracleDecision,
    reject_scope_mismatch: bool,
    reject_zero_execution: bool,
}

impl EvidenceJoinOracle for TestOracle {
    fn verify_check_receipt(
        &self,
        query: &agent_graph::evidence_join::CheckReceiptVerification,
    ) -> OracleDecision {
        if self.reject_zero_execution && query.executed_count == Some(0) {
            return OracleDecision::Rejected;
        }
        self.checks
            .get(&query.receipt_ref)
            .copied()
            .unwrap_or(OracleDecision::Unavailable)
    }

    fn verify_evidence_support(
        &self,
        query: &agent_graph::evidence_join::EvidenceSupportVerification,
    ) -> OracleDecision {
        if self.reject_scope_mismatch
            && matches!(query.claim_scope, ClaimScope::NaturalLanguage)
            && matches!(query.evidence_scope, EvidenceScope::NumericalMethod { .. })
        {
            return OracleDecision::Rejected;
        }
        self.supports
            .get(&query.evidence_ref)
            .copied()
            .unwrap_or(OracleDecision::Unavailable)
    }

    fn verify_source_epoch_compatibility(
        &self,
        _query: &agent_graph::evidence_join::SourceEpochCompatibilityVerification,
    ) -> OracleDecision {
        self.epoch_compatibility
    }

    fn verify_cross_cutting_obligation(
        &self,
        query: &agent_graph::evidence_join::CrossCuttingObligationVerification,
    ) -> OracleDecision {
        self.obligations
            .get(&query.obligation_ref)
            .copied()
            .unwrap_or(OracleDecision::Unavailable)
    }

    fn validate_counterexample(
        &self,
        query: &agent_graph::evidence_join::CounterexampleVerification,
    ) -> OracleDecision {
        self.counterexamples
            .get(&query.counterexample_ref)
            .copied()
            .unwrap_or(OracleDecision::Unavailable)
    }
}

fn branch(index: usize) -> BranchEvidenceCandidate {
    BranchEvidenceCandidate {
        branch_ref: format!("branch-{index}"),
        obligation_refs: vec![format!("obligation-{index}")],
        evidence: vec![EvidenceRefProjection {
            evidence_ref: format!("evidence-{index}"),
            digest: Some(format!("sha256:evidence-{index}")),
            scope: EvidenceScope::NaturalLanguage,
        }],
        required_checks: vec![RequiredCheckProjection {
            check_ref: format!("check-{index}"),
            receipt_ref: Some(format!("receipt-{index}")),
            status: Some("passed".into()),
            executed_count: Some(1),
        }],
        source_group_ref: format!("source-{index}"),
        source_epoch: "epoch-a".into(),
        mandatory_gap_refs: Vec::new(),
        counterexample_refs: Vec::new(),
        abstained: false,
        native_completed: true,
        status: Some("passed".into()),
        support_score: Some(0.99),
        payload: json!({"large_payload": "not authoritative"}),
    }
}

fn seed(branches: Vec<BranchEvidenceCandidate>) -> EvidenceJoinSeed {
    EvidenceJoinSeed {
        target_revision: "revision-a".into(),
        claim: ClaimProjection {
            claim_ref: "claim-a".into(),
            scope: ClaimScope::NaturalLanguage,
        },
        branches,
        cross_cutting_obligation_refs: Vec::new(),
        budget: JoinBudgetProjection {
            exhausted: false,
            required_checks_remaining: 0,
        },
    }
}

fn oracle_for(branches: &[BranchEvidenceCandidate]) -> TestOracle {
    let mut oracle = TestOracle::default();
    for branch in branches {
        for check in &branch.required_checks {
            if let Some(receipt_ref) = &check.receipt_ref {
                oracle
                    .checks
                    .insert(receipt_ref.clone(), OracleDecision::Confirmed);
            }
        }
        for evidence in &branch.evidence {
            oracle
                .supports
                .insert(evidence.evidence_ref.clone(), OracleDecision::Confirmed);
        }
    }
    oracle
}

fn projected(join: &EvidenceJoin, seed: EvidenceJoinSeed) -> EvidenceJoinInput {
    match join.project(seed) {
        Ok(input) => input,
        Err(error) => panic!("projection failed: {error}"),
    }
}

fn resolved(
    join: &EvidenceJoin,
    input: &EvidenceJoinInput,
    oracle: &dyn EvidenceJoinOracle,
) -> agent_graph::evidence_join::EvidenceJoinResult {
    match join.resolve(input, oracle) {
        Ok(result) => result,
        Err(error) => panic!("resolution failed: {error}"),
    }
}

fn has_reason(
    result: &agent_graph::evidence_join::EvidenceJoinResult,
    code: JoinReasonCode,
) -> bool {
    result.reasons.iter().any(|reason| reason.code == code)
}

#[test]
fn ctx_09_large_branch_payloads_become_bounded_projections() {
    let branches: Vec<_> = (0..10)
        .map(|index| {
            let mut candidate = branch(index);
            candidate.payload = json!({"blob": "x".repeat(1_000_000)});
            candidate
        })
        .collect();
    let oracle = oracle_for(&branches);
    let join = EvidenceJoin::new(EvidenceJoinLimits {
        max_branches: 10,
        max_projection_bytes: 32_768,
    });
    let input = projected(&join, seed(branches));
    let encoded = match serde_json::to_vec(&input) {
        Ok(encoded) => encoded,
        Err(error) => panic!("input serialization failed: {error}"),
    };

    assert!(encoded.len() <= join.limits().max_projection_bytes);
    assert!(!String::from_utf8_lossy(&encoded).contains(&"x".repeat(128)));
    assert_eq!(input.branches.len(), 10);
    for (index, projection) in input.branches.iter().enumerate() {
        assert_eq!(projection.branch_ref, format!("branch-{index}"));
        assert_eq!(
            projection.obligation_refs,
            vec![format!("obligation-{index}")]
        );
        assert_eq!(
            projection.evidence[0].evidence_ref,
            format!("evidence-{index}")
        );
    }
    assert_eq!(
        resolved(&join, &input, &oracle).disposition,
        JoinDisposition::Pass
    );

    let too_many = (0..11).map(branch).collect();
    assert!(matches!(
        join.project(seed(too_many)),
        Err(EvidenceJoinError::TooManyBranches {
            actual: 11,
            max: 10
        })
    ));

    let one_branch = vec![branch(0)];
    let measured = serde_json::to_vec(&projected(
        &EvidenceJoin::new(EvidenceJoinLimits {
            max_branches: 10,
            max_projection_bytes: usize::MAX,
        }),
        seed(one_branch.clone()),
    ))
    .map_or(0, |bytes| bytes.len());
    let too_small = EvidenceJoin::new(EvidenceJoinLimits {
        max_branches: 10,
        max_projection_bytes: measured.saturating_sub(1),
    });
    assert!(matches!(
        too_small.project(seed(one_branch)),
        Err(EvidenceJoinError::ProjectionTooLarge { .. })
    ));
}

#[test]
fn ctx_10_failed_parent_cross_cutting_obligation_blocks_join() {
    let branches = vec![branch(0), branch(1)];
    let mut oracle = oracle_for(&branches);
    oracle
        .obligations
        .insert("parent-safety".into(), OracleDecision::Rejected);
    let mut request = seed(branches);
    request.cross_cutting_obligation_refs = vec!["parent-safety".into()];
    let join = EvidenceJoin::default();
    let input = projected(&join, request);
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(
        &result,
        JoinReasonCode::CrossCuttingObligationRejected
    ));
    assert_eq!(result.cross_cutting_obligation_refs, vec!["parent-safety"]);
}

#[test]
fn dep_08_mixed_source_epochs_require_explicit_compatibility() {
    let mut second = branch(1);
    second.source_epoch = "epoch-b".into();
    let branches = vec![branch(0), second];
    let base_oracle = oracle_for(&branches);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));

    for decision in [OracleDecision::Unavailable, OracleDecision::Rejected] {
        let oracle = TestOracle {
            epoch_compatibility: decision,
            ..TestOracle {
                checks: base_oracle.checks.clone(),
                supports: base_oracle.supports.clone(),
                ..TestOracle::default()
            }
        };
        let result = resolved(&join, &input, &oracle);
        assert_eq!(result.disposition, JoinDisposition::Blocked);
    }

    let compatible = TestOracle {
        epoch_compatibility: OracleDecision::Confirmed,
        checks: base_oracle.checks,
        supports: base_oracle.supports,
        ..TestOracle::default()
    };
    let result = resolved(&join, &input, &compatible);
    assert_eq!(result.disposition, JoinDisposition::Pass);
    assert_eq!(result.source_epochs, vec!["epoch-a", "epoch-b"]);
}

#[test]
fn join_01_forged_passed_fields_without_receipt_do_not_verify() {
    let mut forged = branch(0);
    forged.status = Some("passed".into());
    forged.support_score = Some(1.0);
    forged.required_checks[0].status = Some("passed".into());
    forged.required_checks[0].receipt_ref = None;
    let branches = vec![forged];
    let oracle = oracle_for(&branches);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(&result, JoinReasonCode::MissingCheckReceipt));
}

#[test]
fn join_02_receipt_bound_to_another_revision_is_rejected() {
    let branches = vec![branch(0)];
    let mut oracle = oracle_for(&branches);
    oracle
        .checks
        .insert("receipt-0".into(), OracleDecision::Rejected);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(&result, JoinReasonCode::CheckReceiptRejected));
    assert_ne!(result.disposition, JoinDisposition::Pass);
}

#[test]
fn join_03_digest_with_false_support_is_unsupported() {
    let branches = vec![branch(0)];
    let mut oracle = oracle_for(&branches);
    oracle
        .supports
        .insert("evidence-0".into(), OracleDecision::Rejected);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    assert!(input.branches[0].evidence[0].digest.is_some());
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Unsupported);
    assert!(has_reason(&result, JoinReasonCode::EvidenceUnsupported));
}

#[test]
fn join_04_shared_upstream_group_counts_as_one_source() {
    let mut branches: Vec<_> = (0..10).map(branch).collect();
    for branch in &mut branches {
        branch.source_group_ref = "shared-upstream".into();
    }
    let oracle = oracle_for(&branches);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Pass);
    assert_eq!(result.independent_source_count, 1);
    assert_eq!(result.source_group_refs, vec!["shared-upstream"]);
}

#[test]
fn join_05_abstention_and_mandatory_gaps_remain_visible() {
    let mut abstaining = branch(0);
    abstaining.abstained = true;
    abstaining.mandatory_gap_refs = vec!["gap-required-analysis".into()];
    let branches = vec![abstaining, branch(1)];
    let oracle = oracle_for(&branches);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert_eq!(result.abstained_branch_refs, vec!["branch-0"]);
    assert_eq!(result.mandatory_gap_refs, vec!["gap-required-analysis"]);
    assert!(has_reason(&result, JoinReasonCode::BranchAbstained));
    assert!(has_reason(&result, JoinReasonCode::MandatoryGap));
}

#[test]
fn join_06_valid_counterexample_reopens_claim() {
    let mut branches: Vec<_> = (0..10).map(branch).collect();
    branches[9].counterexample_refs = vec!["counterexample-valid".into()];
    let mut oracle = oracle_for(&branches);
    oracle
        .counterexamples
        .insert("counterexample-valid".into(), OracleDecision::Confirmed);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Reopen);
    assert_eq!(
        result.validated_counterexample_refs,
        vec!["counterexample-valid"]
    );
    assert!(has_reason(&result, JoinReasonCode::CounterexampleValidated));
}

#[test]
fn join_07_high_support_score_never_substitutes_for_support() {
    let mut candidate = branch(0);
    candidate.support_score = Some(1.0);
    let branches = vec![candidate];
    let mut oracle = oracle_for(&branches);
    oracle.supports.clear();
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(
        &result,
        JoinReasonCode::EvidenceSupportUnavailable
    ));
}

#[test]
fn join_08_unavailable_required_verifier_is_blocked_unknown() {
    let branches = vec![branch(0)];
    let mut oracle = oracle_for(&branches);
    oracle
        .checks
        .insert("receipt-0".into(), OracleDecision::Unavailable);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(result.unknown_or_unavailable);
    assert!(has_reason(
        &result,
        JoinReasonCode::CheckVerifierUnavailable
    ));
}

#[test]
fn join_09_exhausted_budget_with_checks_remaining_cannot_succeed() {
    let branches = vec![branch(0)];
    let oracle = oracle_for(&branches);
    let mut request = seed(branches);
    request.budget = JoinBudgetProjection {
        exhausted: true,
        required_checks_remaining: 1,
    };
    let join = EvidenceJoin::default();
    let input = projected(&join, request);
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(&result, JoinReasonCode::BudgetExhausted));
}

#[test]
fn str_03_method_scoped_oracle_cannot_certify_natural_language_claim() {
    let mut candidate = branch(0);
    candidate.evidence[0].scope = EvidenceScope::NumericalMethod {
        method_ref: "method-only".into(),
    };
    let branches = vec![candidate];
    let mut oracle = oracle_for(&branches);
    oracle.reject_scope_mismatch = true;
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Unsupported);
    assert!(has_reason(&result, JoinReasonCode::EvidenceUnsupported));
}

#[test]
fn str_04_zero_executed_check_does_not_satisfy_requirement() {
    let mut candidate = branch(0);
    candidate.required_checks[0].status = Some("passed".into());
    candidate.required_checks[0].executed_count = Some(0);
    let branches = vec![candidate];
    let mut oracle = oracle_for(&branches);
    oracle.reject_zero_execution = true;
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.disposition, JoinDisposition::Blocked);
    assert!(has_reason(&result, JoinReasonCode::CheckReceiptRejected));
}

#[test]
fn str_05_native_completion_visible_when_semantic_support_fails() {
    let branches = vec![branch(0)];
    let mut oracle = oracle_for(&branches);
    oracle
        .supports
        .insert("evidence-0".into(), OracleDecision::Rejected);
    let join = EvidenceJoin::default();
    let input = projected(&join, seed(branches));
    let result = resolved(&join, &input, &oracle);

    assert_eq!(result.native_completed_branch_refs, vec!["branch-0"]);
    assert_eq!(result.disposition, JoinDisposition::Unsupported);
    assert_ne!(result.disposition, JoinDisposition::Pass);
}

#[allow(dead_code)]
fn _payload_is_explicitly_non_authoritative(value: Value) -> Value {
    value
}
