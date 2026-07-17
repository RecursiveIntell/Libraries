use aidens_runner::learning_experiment::{
    evaluate, Decision, EvaluationPlan, OwnerTrialEvidence, Side,
};

fn plan() -> EvaluationPlan {
    EvaluationPlan::new(
        "task",
        "family",
        "split",
        "verifier",
        "environment",
        "policy",
        "candidate",
        "baseline",
        7,
    )
}

fn trial(id: &str, family: &str, side: Side) -> OwnerTrialEvidence {
    OwnerTrialEvidence::new(id, family, "train", side, 1)
}

#[test]
fn missing_adapter_is_red_but_contract_is_explicit() {
    let result = evaluate(&plan(), &[trial("a", "f1", Side::Baseline)]);
    assert_eq!(result.decision, Decision::Quarantine);
    assert!(result
        .reasons
        .iter()
        .any(|r| r == "paired-trials-below-minimum"));
}

#[test]
fn qualifying_owner_evidence_is_advice_only() {
    let mut evidence = Vec::new();
    for family in ["f1", "f2", "f3", "f4", "f5"] {
        for n in 0..4 {
            evidence.push(trial(&format!("{family}-b-{n}"), family, Side::Baseline));
            evidence.push(trial(&format!("{family}-c-{n}"), family, Side::Candidate));
        }
    }
    let result = evaluate(&plan(), &evidence);
    assert_eq!(result.decision, Decision::Quarantine);
    assert!(result
        .reasons
        .contains(&"qualifying-advisory-evidence".into()));
}

#[test]
fn owner_invariants_fail_closed() {
    let mut e = vec![
        trial("same", "f1", Side::Baseline),
        trial("same", "f1", Side::Candidate),
    ];
    assert!(evaluate(&plan(), &e)
        .reasons
        .contains(&"duplicate-trial".into()));
    e[1] = trial("other", "f1", Side::Candidate).with_verifier("changed");
    assert!(evaluate(&plan(), &e)
        .reasons
        .contains(&"changed-verifier-or-budget".into()));
}
