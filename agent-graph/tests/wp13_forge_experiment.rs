use agent_graph::forge_experiment::*;
use std::cell::RefCell;
use std::collections::BTreeSet;

fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn spec() -> ExperimentSpec {
    ExperimentSpec {
        experiment_id: "exp-13".into(),
        base_revision: "base-a".into(),
        test_sets: TestSetSnapshot {
            public_digest: "public-v1".into(),
            evaluator_digest: "evaluator-v1".into(),
            withheld_digest: "withheld-v1".into(),
        },
        allowed_patch_scope: set(&["agent-graph/src"]),
        max_patch_delta_lines: 20,
    }
}

fn candidate(id: &str, workspace: &str) -> CandidateWorkspace {
    CandidateWorkspace {
        candidate_id: id.into(),
        workspace_id: workspace.into(),
        base_revision: "base-a".into(),
        provenance_ref: format!("provenance-{id}"),
        staging_status: StagingStatus::Passed,
        containment_evidence_ref: Some(format!("containment-{id}")),
        patch: PatchProposal {
            suggested_delta_lines: 5,
            actual_delta_lines: 5,
            touched_scopes: set(&["agent-graph/src"]),
        },
        parent_candidate_ids: Vec::new(),
        interaction_checks: InteractionChecks::NotRequired,
    }
}

#[derive(Default)]
struct TestOwner {
    containment: BTreeSet<String>,
    authorizations: BTreeSet<String>,
    publications: RefCell<Vec<String>>,
    unavailable: bool,
}

impl TestOwner {
    fn allowing(ids: &[&str]) -> Self {
        Self {
            containment: ids.iter().map(|id| format!("containment-{id}")).collect(),
            authorizations: ids.iter().map(|id| (*id).to_owned()).collect(),
            publications: RefCell::new(Vec::new()),
            unavailable: false,
        }
    }
}

impl ForgeNativeOwner for TestOwner {
    fn verify_containment(&self, query: &ContainmentQuery) -> OwnerDecision {
        if self.unavailable {
            OwnerDecision::Unavailable
        } else if self.containment.contains(&query.evidence_ref) {
            OwnerDecision::Authorized
        } else {
            OwnerDecision::Rejected
        }
    }

    fn authorize_operation(&self, query: &OperationAuthorizationQuery) -> OwnerAuthorization {
        if self.unavailable {
            OwnerAuthorization::Unavailable
        } else if self.authorizations.contains(&query.candidate_id) {
            OwnerAuthorization::Granted(NativeOperationGrant {
                operation_identity_ref: format!("operation-{}", query.candidate_id),
                permit_ref: format!("permit-{}", query.candidate_id),
                effect_accounting_ref: format!("effects-{}", query.candidate_id),
            })
        } else {
            OwnerAuthorization::Rejected
        }
    }

    fn publish(&self, request: &PublicationRequest) -> Result<PublicationReceipt, OwnerDecision> {
        if self.unavailable || !self.authorizations.contains(&request.candidate_id) {
            return Err(if self.unavailable {
                OwnerDecision::Unavailable
            } else {
                OwnerDecision::Rejected
            });
        }
        self.publications
            .borrow_mut()
            .push(request.candidate_id.clone());
        Ok(PublicationReceipt {
            publication_ref: format!("publication-{}", request.candidate_id),
            operation_identity_ref: request.authorization.operation_identity_ref.clone(),
            permit_ref: request.authorization.permit_ref.clone(),
            effect_accounting_ref: request.authorization.effect_accounting_ref.clone(),
        })
    }
}

#[test]
fn spec_01_isolated_workspaces_have_distinct_bases_and_provenance() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .register_candidate(candidate("a", "workspace-a"))
        .unwrap();
    let mut second = candidate("b", "workspace-b");
    second.base_revision = "base-b".into();
    experiment.register_candidate(second).unwrap();

    let first = experiment.candidate("a").unwrap();
    let second = experiment.candidate("b").unwrap();
    assert_ne!(first.workspace_id, second.workspace_id);
    assert_ne!(first.base_revision, second.base_revision);
    assert_ne!(first.provenance_ref, second.provenance_ref);

    let duplicate_workspace = candidate("c", "workspace-a");
    assert_eq!(
        experiment.register_candidate(duplicate_workspace),
        Err(ForgeError::WorkspaceNotIsolated("workspace-a".into()))
    );
}

#[test]
fn spec_02_staging_status_cannot_substitute_for_native_containment_evidence() {
    let mut experiment = ForgeExperiment::new(spec());
    let mut staged = candidate("a", "workspace-a");
    staged.containment_evidence_ref = None;
    experiment.register_candidate(staged).unwrap();

    let assessment = experiment.assess_candidate("a", "base-a", &TestOwner::allowing(&["a"]));
    assert_eq!(assessment.disposition, CandidateDisposition::Blocked);
    assert!(assessment
        .reasons
        .contains(&GovernanceReason::ContainmentEvidenceMissing));
    assert!(!assessment.native_containment_verified);
}

#[test]
fn spec_03_stale_base_requires_isolated_rebase_and_retest() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .register_candidate(candidate("a", "workspace-a"))
        .unwrap();

    let stale = experiment.assess_candidate("a", "base-b", &TestOwner::allowing(&["a"]));
    assert_eq!(stale.disposition, CandidateDisposition::StaleBase);

    let rebased = experiment
        .rebase_candidate(
            "a",
            "a-rebased",
            "workspace-a-rebased",
            "base-b",
            "provenance-a-rebase",
        )
        .unwrap();
    assert_eq!(rebased.base_revision, "base-b");
    assert_ne!(rebased.workspace_id, "workspace-a");
    assert_eq!(rebased.interaction_checks, InteractionChecks::Required);
    assert_eq!(
        experiment.candidate("a").unwrap().lifecycle,
        CandidateLifecycle::Historical
    );
    assert_eq!(
        experiment
            .assess_candidate("a-rebased", "base-b", &TestOwner::allowing(&["a-rebased"]))
            .disposition,
        CandidateDisposition::RetestRequired
    );
}

#[test]
fn spec_04_speculative_memory_cannot_promote_canonical_supported_memory() {
    let mut experiment = ForgeExperiment::new(spec());
    let record = experiment.record_candidate_memory("a", "hypothesis: parser race");
    assert_eq!(record.tier, MemoryTier::CandidateQuarantine);
    assert_eq!(
        experiment.promote_candidate_memory(&record.record_id),
        Err(ForgeError::CanonicalMemoryOwnerRequired)
    );
    assert!(experiment.candidate_memory().iter().all(|entry| {
        entry.tier == MemoryTier::CandidateQuarantine && !entry.canonical_supported
    }));
}

#[test]
fn spec_05_exactly_one_selected_candidate_publishes_and_others_remain_historical() {
    let mut experiment = ForgeExperiment::new(spec());
    for (id, workspace) in [("a", "workspace-a"), ("b", "workspace-b")] {
        experiment
            .register_candidate(candidate(id, workspace))
            .unwrap();
        experiment
            .record_interaction_checks(id, "interaction-ok")
            .unwrap();
    }
    experiment.select_candidate("a").unwrap();
    let owner = TestOwner::allowing(&["a", "b"]);

    let receipt = experiment.publish_selected(&owner).unwrap();
    assert_eq!(receipt.publication_ref, "publication-a");
    assert_eq!(owner.publications.borrow().as_slice(), &["a"]);
    assert_eq!(
        experiment.candidate("a").unwrap().lifecycle,
        CandidateLifecycle::Published
    );
    assert_eq!(
        experiment.candidate("b").unwrap().lifecycle,
        CandidateLifecycle::Historical
    );
    assert_eq!(
        experiment.publish_selected(&owner),
        Err(ForgeError::PublicationAlreadyCompleted)
    );
    assert_eq!(owner.publications.borrow().as_slice(), &["a"]);
}

#[test]
fn spec_06_changed_independent_tests_are_flagged_for_review_not_silently_accepted() {
    let experiment = ForgeExperiment::new(spec());
    let changed = TestSetSnapshot {
        public_digest: "public-v1".into(),
        evaluator_digest: "evaluator-v2".into(),
        withheld_digest: "withheld-v1".into(),
    };
    let assessment = experiment.assess_test_sets(&changed);
    assert!(!assessment.accepted);
    assert!(assessment.independent_test_change_review_required);
    assert_eq!(assessment.changed_sets, vec![TestSetKind::Evaluator]);
    assert_eq!(experiment.test_sets().evaluator_digest, "evaluator-v1");
}

#[test]
fn exp_01_cea_proposal_is_suspect_and_hypothesis_until_paired_intervention() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment.record_cea_proposal(CeaProposal {
        proposal_id: "cea-1".into(),
        suspect: "scheduler ordering".into(),
        hypothesis: "ordering causes duplicate effects".into(),
    });
    assert_eq!(
        experiment.causal_state("cea-1"),
        Some(CausalState::HypothesisOnly)
    );
    assert_eq!(experiment.diagnosis("cea-1"), None);

    experiment
        .record_paired_intervention("cea-1", "intervention-1", true, true)
        .unwrap();
    assert_eq!(
        experiment.causal_state("cea-1"),
        Some(CausalState::SupportedByPairedIntervention)
    );
}

#[test]
fn exp_02_invalid_or_noncomparable_ablation_is_not_refutation() {
    let invalid = ForgeExperiment::evaluate_ablation(AblationEvidence {
        valid: false,
        comparable: true,
        control_environment: "env-a".into(),
        treatment_environment: "env-a".into(),
        effect_observed: false,
    });
    let noncomparable = ForgeExperiment::evaluate_ablation(AblationEvidence {
        valid: true,
        comparable: false,
        control_environment: "env-a".into(),
        treatment_environment: "env-a".into(),
        effect_observed: false,
    });
    assert_eq!(invalid, CausalComparison::InconclusiveInvalidAblation);
    assert_eq!(noncomparable, CausalComparison::InconclusiveNonComparable);
    assert_ne!(invalid, CausalComparison::Refuted);
    assert_ne!(noncomparable, CausalComparison::Refuted);
}

#[test]
fn exp_03_environment_confound_blocks_causal_comparison() {
    let comparison = ForgeExperiment::evaluate_ablation(AblationEvidence {
        valid: true,
        comparable: true,
        control_environment: "kernel-a".into(),
        treatment_environment: "kernel-b".into(),
        effect_observed: true,
    });
    assert_eq!(comparison, CausalComparison::BlockedEnvironmentConfound);
}

#[test]
fn exp_04_managed_experiment_blocks_without_native_identity_permit_and_effect_owner() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .register_candidate(candidate("a", "workspace-a"))
        .unwrap();
    let owner = TestOwner {
        containment: set(&["containment-a"]),
        unavailable: true,
        ..TestOwner::default()
    };

    let assessment = experiment.assess_candidate("a", "base-a", &owner);
    assert_eq!(assessment.disposition, CandidateDisposition::Blocked);
    assert!(assessment
        .reasons
        .contains(&GovernanceReason::NativeOwnerUnavailable));
    assert!(assessment.authorization.is_none());
}

#[test]
fn exp_05_suggested_clamp_cannot_hide_actual_delta_or_owner_scope_excess() {
    let mut experiment = ForgeExperiment::new(spec());
    let mut oversized = candidate("a", "workspace-a");
    oversized.patch.suggested_delta_lines = 5;
    oversized.patch.actual_delta_lines = 21;
    oversized.patch.touched_scopes = set(&["agent-graph/src", "semantic-memory/src"]);
    experiment.register_candidate(oversized).unwrap();

    let assessment = experiment.assess_candidate("a", "base-a", &TestOwner::allowing(&["a"]));
    assert_eq!(assessment.disposition, CandidateDisposition::Blocked);
    assert!(assessment
        .reasons
        .contains(&GovernanceReason::ActualDeltaExceedsClamp));
    assert!(assessment
        .reasons
        .contains(&GovernanceReason::PatchScopeExceeded));
    assert!(assessment.authorization.is_none());
}

#[test]
fn exp_06_combined_patches_get_new_identity_and_fresh_interaction_checks() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .register_candidate(candidate("a", "workspace-a"))
        .unwrap();
    experiment
        .register_candidate(candidate("b", "workspace-b"))
        .unwrap();

    let combined = experiment
        .combine_candidates(
            &["a", "b"],
            "combined-ab",
            "workspace-combined",
            "provenance-combined",
        )
        .unwrap();
    assert_eq!(combined.candidate_id, "combined-ab");
    assert_eq!(combined.parent_candidate_ids, vec!["a", "b"]);
    assert_eq!(combined.interaction_checks, InteractionChecks::Required);
    assert_ne!(combined.provenance_ref, "provenance-a");
    assert_eq!(
        experiment
            .assess_candidate(
                "combined-ab",
                "base-a",
                &TestOwner::allowing(&["combined-ab"])
            )
            .disposition,
        CandidateDisposition::RetestRequired
    );
}

#[test]
fn exp_07_all_scheduled_trials_retain_failures_and_costs() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .schedule_trial(TrialPlan::public("trial-pass", "a"))
        .unwrap();
    experiment
        .schedule_trial(TrialPlan::public("trial-fail", "b"))
        .unwrap();
    experiment
        .complete_trial(
            "trial-pass",
            TrialCompletion {
                outcome: TrialOutcome::Passed,
                cost: TrialCost {
                    wall_millis: 11,
                    compute_units: 2,
                },
                environment_ref: "env-a".into(),
            },
        )
        .unwrap();
    experiment
        .complete_trial(
            "trial-fail",
            TrialCompletion {
                outcome: TrialOutcome::Failed("assertion failed".into()),
                cost: TrialCost {
                    wall_millis: 17,
                    compute_units: 3,
                },
                environment_ref: "env-a".into(),
            },
        )
        .unwrap();

    let trials = experiment.trials();
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0].trial_id, "trial-pass");
    assert_eq!(trials[1].trial_id, "trial-fail");
    assert_eq!(trials[0].cost.as_ref().unwrap().wall_millis, 11);
    assert_eq!(trials[1].cost.as_ref().unwrap().compute_units, 3);
    assert!(matches!(trials[1].outcome, Some(TrialOutcome::Failed(_))));
}

#[test]
fn exp_08_withheld_access_denies_and_contaminates_trial_but_public_tests_stay_valid() {
    let mut experiment = ForgeExperiment::new(spec());
    experiment
        .schedule_trial(TrialPlan {
            trial_id: "trial-withheld".into(),
            candidate_id: "a".into(),
            requested_test_access: TestAccess::WithheldDiscriminator,
        })
        .unwrap();
    experiment
        .complete_trial(
            "trial-withheld",
            TrialCompletion {
                outcome: TrialOutcome::Passed,
                cost: TrialCost {
                    wall_millis: 7,
                    compute_units: 1,
                },
                environment_ref: "env-a".into(),
            },
        )
        .unwrap();

    let trial = &experiment.trials()[0];
    assert_eq!(trial.outcome, Some(TrialOutcome::WithheldAccessDenied));
    assert!(trial.contaminated);
    assert!(!trial.usable);
    assert!(trial.public_tests_remain_valid);
    assert_eq!(experiment.test_sets().public_digest, "public-v1");
}
