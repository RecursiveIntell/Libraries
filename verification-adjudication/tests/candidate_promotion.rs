use verification_adjudication::*;

fn digest(label: &str) -> IdentityDigest {
    IdentityDigest::of(label)
}
fn input() -> CandidatePromotionInput {
    CandidatePromotionInput {
        adjudication_id: "adj-1".into(),
        candidate_id: "candidate-1".into(),
        candidate_digest: digest("candidate"),
        patch_digest: digest("patch"),
        source_tree_digest: digest("source"),
        verifier_digest: digest("verifier"),
        check_policy_digest: digest("policy"),
        environment_digest: digest("env"),
        image_digest: digest("image"),
        experiment_id: "experiment-1".into(),
        evidence_bundle_id: "bundle-1".into(),
        evidence_bundle_digest: digest("bundle"),
        assignment_digest: digest("assignment"),
        paired_denominator: 10,
        admissible_pairs: 10,
        excluded_pairs: 0,
        uncertainty: UncertaintyV1 {
            estimate: 0.9,
            lower_bound: 0.85,
            upper_bound: 0.95,
        },
        family_results: vec![FamilyGateV1 {
            family: "exact".into(),
            score: 0.95,
            passed: true,
            admissible_pairs: 10,
        }],
        holdout_result: HoldoutGateV1 {
            score: 0.9,
            passed: true,
            admissible_pairs: 4,
        },
        thresholds: FrozenPromotionThresholdsV1 {
            minimum_admissible_pairs: 8,
            minimum_family_score: 0.8,
            minimum_holdout_score: 0.8,
            maximum_uncertainty: 0.2,
        },
        source_receipt_refs: vec![ReceiptRef {
            receipt_id: "receipt-1".into(),
            receipt_digest: digest("receipt"),
        }],
        created_at: "2026-07-19T00:00:00Z".into(),
    }
}

#[test]
fn eligible_contract_is_valid_and_digest_is_deterministic() {
    let first = adjudicate_candidate(input());
    let second = adjudicate_candidate(input());
    assert_eq!(
        first.decision,
        AdjudicationDecisionV1::EligibleForLifecycleConsideration
    );
    assert_eq!(first.adjudication_digest, second.adjudication_digest);
    first.validate().unwrap();
}

#[test]
fn failed_family_or_holdout_is_quarantined() {
    let mut value = input();
    value.family_results[0].passed = false;
    assert_eq!(
        adjudicate_candidate(value).decision,
        AdjudicationDecisionV1::Quarantined
    );
    let mut value = input();
    value.holdout_result.passed = false;
    assert_eq!(
        adjudicate_candidate(value).decision,
        AdjudicationDecisionV1::Quarantined
    );
}

#[test]
fn zero_denominator_is_inconclusive_and_invalid_artifacts_are_rejected() {
    let mut value = input();
    value.paired_denominator = 0;
    value.admissible_pairs = 0;
    value.excluded_pairs = 0;
    let artifact = adjudicate_candidate(value);
    assert_eq!(artifact.decision, AdjudicationDecisionV1::Inconclusive);
    assert!(artifact.validate().is_err());
    assert!(IdentityDigest::new("not-a-digest").is_err());
}

#[test]
fn material_mutation_breaks_digest_binding() {
    let mut artifact = adjudicate_candidate(input());
    artifact.candidate_id = "altered".into();
    assert!(artifact.validate().is_err());
}
