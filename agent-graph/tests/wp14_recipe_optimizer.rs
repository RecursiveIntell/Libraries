use agent_graph::recipe_optimizer::{
    AcceptanceCriteria, AccessRequest, ActivationDecision, BaselineRecipe, CandidateRecipe,
    CandidateState, ControlMutation, EvaluationDecision, EvaluationEnvelope,
    EvaluatorIntegrityOwnerPort, OfflineRecipeOptimizer, OptimizationDisposition,
    PublisherAuthorityOwnerPort, QualificationDecision, QualificationOwnerPort,
    ReadOnlyTrialReceipt, ReadOnlyWork, ReadOnlyWorkOwnerPort, RecipeCapability, TrialDisposition,
    TrialRequest,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct TestQualificationOwner {
    decisions: BTreeMap<String, QualificationDecision>,
}

impl QualificationOwnerPort for TestQualificationOwner {
    fn qualification_for(&self, work: &ReadOnlyWork) -> QualificationDecision {
        self.decisions
            .get(&work.work_ref)
            .cloned()
            .unwrap_or(QualificationDecision::Unknown)
    }
}

#[derive(Default)]
struct TestEvaluator {
    evaluated_receipts: std::cell::RefCell<Vec<String>>,
}

impl EvaluatorIntegrityOwnerPort for TestEvaluator {
    fn assess_integrity(&self, envelope: &EvaluationEnvelope<'_>) -> EvaluationDecision {
        if !envelope.proposed_control_mutations.is_empty() {
            return EvaluationDecision::RejectedControlMutation;
        }
        if envelope
            .access_requests
            .iter()
            .any(AccessRequest::is_held_out)
        {
            return EvaluationDecision::Contaminated;
        }
        match envelope.trial_receipt {
            Some(receipt) => {
                self.evaluated_receipts
                    .borrow_mut()
                    .push(receipt.receipt_ref.clone());
                receipt.evaluation.clone()
            }
            None => EvaluationDecision::Admissible,
        }
    }
}

#[derive(Default)]
struct TestPublisher {
    decision: ActivationDecision,
    requests: std::cell::RefCell<Vec<String>>,
    native_activation_log: std::cell::RefCell<Vec<String>>,
}

impl PublisherAuthorityOwnerPort for TestPublisher {
    fn authorize_activation(&self, candidate_ref: &str) -> ActivationDecision {
        self.requests.borrow_mut().push(candidate_ref.to_owned());
        self.decision.clone()
    }
}

#[derive(Default)]
struct RecordingReadOnlyOwner {
    executed: Vec<String>,
    patch_engine_calls: usize,
    effect_calls: usize,
    outcomes: BTreeMap<String, EvaluationDecision>,
}

impl ReadOnlyWorkOwnerPort for RecordingReadOnlyOwner {
    fn execute_read_only(
        &mut self,
        work: &ReadOnlyWork,
        _candidate: &CandidateRecipe,
        qualification_receipt_ref: &str,
    ) -> ReadOnlyTrialReceipt {
        self.executed.push(work.work_ref.clone());
        ReadOnlyTrialReceipt {
            receipt_ref: format!("trial:{}", work.work_ref),
            work_ref: work.work_ref.clone(),
            qualification_receipt_ref: qualification_receipt_ref.to_owned(),
            evaluation: self
                .outcomes
                .get(&work.work_ref)
                .cloned()
                .unwrap_or(EvaluationDecision::Inconclusive),
        }
    }
}

fn baseline() -> BaselineRecipe {
    BaselineRecipe::new("recipe:baseline", "sha256:baseline", 0.80)
}

fn criteria() -> AcceptanceCriteria {
    AcceptanceCriteria::new(
        "sha256:criteria-v1",
        ["check:safety", "check:quality"],
        0.05,
    )
}

fn candidate(id: &str) -> CandidateRecipe {
    CandidateRecipe::new(id, "recipe:baseline", [("context_window", "8192")])
}

fn public_work(id: &str) -> ReadOnlyWork {
    ReadOnlyWork::new(id, [AccessRequest::PublicInput("fixture:public".into())])
}

fn qualified_owner(work_refs: &[&str]) -> TestQualificationOwner {
    TestQualificationOwner {
        decisions: work_refs
            .iter()
            .map(|work_ref| {
                (
                    (*work_ref).to_owned(),
                    QualificationDecision::Qualified {
                        receipt_ref: format!("qualification:{work_ref}"),
                    },
                )
            })
            .collect(),
    }
}

#[test]
fn auth_05_publisher_attempts_self_promotion_without_independent_authorization() {
    let optimizer = OfflineRecipeOptimizer::new(baseline(), criteria());
    let publisher = TestPublisher {
        decision: ActivationDecision::Denied {
            reason_ref: "authorization:independent-owner-required".into(),
        },
        ..TestPublisher::default()
    };
    let candidate = candidate("candidate:self-promote");

    let request = optimizer.request_activation(&candidate, &publisher);

    assert_eq!(
        request.decision,
        ActivationDecision::Denied {
            reason_ref: "authorization:independent-owner-required".into()
        }
    );
    assert_eq!(
        publisher.requests.borrow().as_slice(),
        ["candidate:self-promote"]
    );
    assert!(publisher.native_activation_log.borrow().is_empty());
    assert!(!request.activated);
    assert_eq!(request.candidate.state, CandidateState::NonAuthorizing);
}

#[test]
fn eval_02_hidden_test_access_contaminates_and_excludes_validation() {
    let optimizer = OfflineRecipeOptimizer::new(baseline(), criteria());
    let candidate = candidate("candidate:hidden-reader");
    let work = ReadOnlyWork::new(
        "work:hidden",
        [
            AccessRequest::HeldOutRead("acceptance:hidden-7".into()),
            AccessRequest::HeldOutWrite("acceptance:hidden-7".into()),
        ],
    );
    let qualifier = qualified_owner(&["work:hidden"]);
    let evaluator = TestEvaluator::default();
    let mut worker = RecordingReadOnlyOwner::default();

    let report = optimizer.optimize(
        [TrialRequest::new(candidate, work)],
        &qualifier,
        &evaluator,
        &mut worker,
    );

    assert_eq!(
        report.trials[0].disposition,
        TrialDisposition::ExcludedContaminated
    );
    assert_eq!(
        report.trials[0].evaluation,
        EvaluationDecision::Contaminated
    );
    assert!(report.trials[0].validation_score.is_none());
    assert!(worker.executed.is_empty());
    assert!(evaluator.evaluated_receipts.borrow().is_empty());
    assert_eq!(
        report.disposition,
        OptimizationDisposition::NoQualifiedImprovement
    );
}

#[test]
fn opt_01_only_already_qualified_read_only_work_executes_without_effect_capability() {
    let optimizer = OfflineRecipeOptimizer::new(baseline(), criteria());
    let qualifier = qualified_owner(&["work:qualified"]);
    let evaluator = TestEvaluator::default();
    let mut worker = RecordingReadOnlyOwner {
        outcomes: [(
            "work:qualified".into(),
            EvaluationDecision::Qualified { score: 0.90 },
        )]
        .into_iter()
        .collect(),
        ..RecordingReadOnlyOwner::default()
    };

    let report = optimizer.optimize(
        [
            TrialRequest::new(
                candidate("candidate:qualified"),
                public_work("work:qualified"),
            ),
            TrialRequest::new(
                candidate("candidate:unqualified"),
                public_work("work:unqualified"),
            ),
        ],
        &qualifier,
        &evaluator,
        &mut worker,
    );

    assert_eq!(worker.executed, vec!["work:qualified"]);
    assert_eq!(
        report.trials[0].disposition,
        TrialDisposition::QualifiedImprovement
    );
    assert_eq!(
        report.trials[1].disposition,
        TrialDisposition::ExcludedUnqualified
    );
    assert_eq!(worker.patch_engine_calls, 0);
    assert_eq!(worker.effect_calls, 0);
    assert_eq!(
        report.capability_manifest.capabilities,
        BTreeSet::from([
            RecipeCapability::ReadQualifiedWork,
            RecipeCapability::RankCandidates
        ])
    );
    assert!(!report
        .capability_manifest
        .allows(RecipeCapability::PatchEngine));
    assert!(!report
        .capability_manifest
        .allows(RecipeCapability::ExternalEffect));
    assert!(!report.granted_new_effect_permission);
}

#[test]
fn opt_02_evaluator_or_acceptance_gate_mutation_is_rejected_and_candidate_stays_separate() {
    let baseline = baseline();
    let criteria = criteria();
    let optimizer = OfflineRecipeOptimizer::new(baseline.clone(), criteria.clone());
    let qualifier = qualified_owner(&["work:mutator"]);
    let evaluator = TestEvaluator::default();
    let mut worker = RecordingReadOnlyOwner::default();
    let request = TrialRequest::new(candidate("candidate:mutator"), public_work("work:mutator"))
        .with_control_mutations([
            ControlMutation::RemoveMandatoryCheck("check:safety".into()),
            ControlMutation::SetHeldOutPassMargin(0.0),
            ControlMutation::ReplaceEvaluator("evaluator:self".into()),
        ]);

    let report = optimizer.optimize([request], &qualifier, &evaluator, &mut worker);

    assert_eq!(
        report.trials[0].disposition,
        TrialDisposition::RejectedControlMutation
    );
    assert_eq!(
        report.trials[0].evaluation,
        EvaluationDecision::RejectedControlMutation
    );
    assert!(worker.executed.is_empty());
    assert_eq!(report.baseline, baseline);
    assert_eq!(report.acceptance_criteria, criteria);
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(report.candidates[0].recipe.recipe_ref, "candidate:mutator");
    assert_eq!(report.candidates[0].state, CandidateState::NonAuthorizing);
    assert_ne!(
        report.candidates[0].recipe.recipe_ref,
        report.baseline.recipe_ref
    );
    assert!(report.candidates[0].activation_ref.is_none());
}

#[test]
fn opt_03_no_qualified_improvement_preserves_baseline_and_criteria() {
    let baseline = baseline();
    let criteria = criteria();
    let optimizer = OfflineRecipeOptimizer::new(baseline.clone(), criteria.clone());
    let qualifier = qualified_owner(&["work:regression", "work:inconclusive"]);
    let evaluator = TestEvaluator::default();
    let mut worker = RecordingReadOnlyOwner {
        outcomes: [
            (
                "work:regression".into(),
                EvaluationDecision::Qualified { score: 0.70 },
            ),
            ("work:inconclusive".into(), EvaluationDecision::Inconclusive),
        ]
        .into_iter()
        .collect(),
        ..RecordingReadOnlyOwner::default()
    };

    let report = optimizer.optimize(
        [
            TrialRequest::new(
                candidate("candidate:regression"),
                public_work("work:regression"),
            ),
            TrialRequest::new(
                candidate("candidate:inconclusive"),
                public_work("work:inconclusive"),
            ),
        ],
        &qualifier,
        &evaluator,
        &mut worker,
    );

    assert_eq!(
        report.disposition,
        OptimizationDisposition::NoQualifiedImprovement
    );
    assert_eq!(report.selected_recipe, baseline);
    assert_eq!(report.baseline, baseline);
    assert_eq!(report.acceptance_criteria, criteria);
    assert!(report.promotion_request.is_none());
    assert!(report.candidates.iter().all(|artifact| {
        artifact.state == CandidateState::NonAuthorizing && artifact.activation_ref.is_none()
    }));
    assert_eq!(
        report
            .trials
            .iter()
            .map(|trial| trial.disposition.clone())
            .collect::<Vec<_>>(),
        vec![
            TrialDisposition::QualifiedRegression,
            TrialDisposition::Inconclusive
        ]
    );
}
