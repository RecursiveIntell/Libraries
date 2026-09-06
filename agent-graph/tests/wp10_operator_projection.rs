use agent_graph::operator_projection::{
    aggregate_recommendations, project_action_nudges, project_observations,
    project_regression_export, project_sensitive_features, project_source_invalidation,
    project_terminal, route_operator_request, ActionIdentityDecision, ActionIdentityOwnerPort,
    ActionNudge, ApprovalDecision, ApprovalOwnerPort, AuthorizedCommandPort, CoverageProjection,
    DurableRunState, EvidenceReference, FeatureReference, InteractionPolicyLimits, Observation,
    OperatorRequest, ProjectionState, PurposeDecision, PurposeRestrictionOwnerPort, Recommendation,
    RecommendationStance, RegressionExportInput, SensitiveFeatureInput, SourceInvalidationDecision,
    SourceInvalidationOwnerPort, TerminalInput,
};
use std::collections::BTreeMap;

#[derive(Default)]
struct TestOwners {
    approval: ApprovalDecision,
    action_identities: BTreeMap<String, ActionIdentityDecision>,
    purpose_decisions: BTreeMap<(String, String), PurposeDecision>,
    invalidation: Option<SourceInvalidationDecision>,
}

impl ApprovalOwnerPort for TestOwners {
    fn approval_for(&self, _action_identity: &str) -> ApprovalDecision {
        self.approval.clone()
    }
}

impl ActionIdentityOwnerPort for TestOwners {
    fn classify_action(&self, nudge: &ActionNudge) -> ActionIdentityDecision {
        self.action_identities
            .get(&nudge.text)
            .cloned()
            .unwrap_or(ActionIdentityDecision::Unknown)
    }
}

impl PurposeRestrictionOwnerPort for TestOwners {
    fn decide_purpose(&self, feature_ref: &str, purpose: &str) -> PurposeDecision {
        self.purpose_decisions
            .get(&(feature_ref.to_owned(), purpose.to_owned()))
            .cloned()
            .unwrap_or(PurposeDecision::Unknown)
    }
}

impl SourceInvalidationOwnerPort for TestOwners {
    fn source_invalidation(&self, _source_ref: &str) -> Option<SourceInvalidationDecision> {
        self.invalidation.clone()
    }
}

#[derive(Default)]
struct RecordingCommandPort {
    writes: Vec<String>,
}

impl AuthorizedCommandPort for RecordingCommandPort {
    fn execute_authorized(&mut self, command_ref: &str, _approval_ref: &str) {
        self.writes.push(command_ref.to_owned());
    }
}

#[test]
fn obs_01_observation_sink_sequence_gap_is_exact_and_durable_state_is_unchanged() {
    let canonical = DurableRunState {
        run_ref: "run-1".into(),
        durable_sequence: 5,
        terminal: false,
        state_digest: "sha256:canonical".into(),
    };
    let before = canonical.clone();
    let projection = project_observations(
        &canonical,
        &[
            Observation::new(1, "event-1"),
            Observation::new(2, "event-2"),
            Observation::new(5, "event-5"),
        ],
    );

    assert_eq!(projection.state, ProjectionState::Observed);
    assert_eq!(projection.gaps.len(), 1);
    assert_eq!(projection.gaps[0].missing_from, 3);
    assert_eq!(projection.gaps[0].missing_through, 4);
    assert_eq!(projection.gaps[0].next_observed, 5);
    assert_eq!(projection.canonical_run_state, before);
    assert_eq!(canonical, before);
}

#[test]
fn obs_02_terminal_execution_and_semantic_closure_are_separate_when_evidence_is_missing() {
    let projection = project_terminal(TerminalInput {
        run_ref: "run-2".into(),
        terminal_receipt_ref: "terminal-receipt-2".into(),
        execution_completed: true,
        required_evidence_refs: vec!["evidence-a".into(), "evidence-b".into()],
        supported_evidence_refs: vec!["evidence-a".into()],
    });

    assert_eq!(
        projection.execution_state,
        ProjectionState::CompletedExecution
    );
    assert_eq!(projection.semantic_state, ProjectionState::Unsupported);
    assert_eq!(projection.missing_evidence_refs, vec!["evidence-b"]);
    assert_ne!(projection.semantic_state, ProjectionState::Supported);
}

#[test]
fn obs_03_source_change_projection_exposes_coverage_chain_and_affected_refs_without_transcript() {
    let owners = TestOwners {
        invalidation: Some(SourceInvalidationDecision {
            change_ref: "change:source-v2".into(),
            coverage: CoverageProjection {
                covered_scope_refs: vec!["src/a.rs".into(), "src/b.rs".into()],
                uncovered_scope_refs: vec!["src/c.rs".into()],
            },
            dependency_chain_refs: vec!["source:v1".into(), "analysis:7".into(), "result:9".into()],
            affected_result_refs: vec!["result:9".into()],
        }),
        ..TestOwners::default()
    };

    let projection = project_source_invalidation("source:v1", &owners);
    assert_eq!(projection.state, ProjectionState::Observed);
    assert_eq!(projection.change_ref.as_deref(), Some("change:source-v2"));
    assert_eq!(projection.coverage.uncovered_scope_refs, vec!["src/c.rs"]);
    assert_eq!(
        projection.dependency_chain_refs,
        vec!["source:v1", "analysis:7", "result:9"]
    );
    assert_eq!(projection.affected_result_refs, vec!["result:9"]);

    let encoded = serde_json::to_string(&projection).unwrap();
    assert!(!encoded.contains("transcript"));
}

#[test]
fn obs_04_sensitive_failure_export_is_clean_room_redacted_and_limitation_explicit() {
    let secret = "Bearer super-sensitive-token";
    let export = project_regression_export(RegressionExportInput {
        failure_ref: "failure:44".into(),
        replay_scope_refs: vec!["fixture:shape-only".into()],
        evidence: vec![EvidenceReference::sensitive(
            "evidence:private-44",
            secret.as_bytes().to_vec(),
        )],
        missing_data_refs: vec!["private-input:omitted".into()],
    });

    assert_eq!(export.state, ProjectionState::Withheld);
    assert_eq!(
        export.clean_room_replay_scope_refs,
        vec!["fixture:shape-only"]
    );
    assert_eq!(export.evidence_refs[0].evidence_ref, "evidence:private-44");
    assert!(export.evidence_refs[0].redacted);
    assert_eq!(
        export.limitations,
        vec!["missing-data:private-input:omitted"]
    );

    let encoded = serde_json::to_string(&export).unwrap();
    assert!(!encoded.contains(secret));
    assert!(!encoded.contains("super-sensitive-token"));
}

#[test]
fn obs_05_observer_mutation_and_direct_permit_are_denied_with_zero_unauthorized_writes() {
    let denied_owners = TestOwners {
        approval: ApprovalDecision::Denied {
            reason_ref: "policy:no-observer-authority".into(),
        },
        ..TestOwners::default()
    };
    let mut commands = RecordingCommandPort::default();

    for request in [
        OperatorRequest::ObserverMutation {
            command_ref: "mutate:run".into(),
        },
        OperatorRequest::DirectPermit {
            action_identity: "action:deploy".into(),
        },
    ] {
        let result = route_operator_request(request, &denied_owners, &mut commands);
        assert_eq!(result.state, ProjectionState::Blocked);
        assert!(result.command_receipt_ref.is_none());
    }
    assert!(commands.writes.is_empty());

    let authorized_owners = TestOwners {
        approval: ApprovalDecision::Granted {
            approval_ref: "approval:owner-7".into(),
        },
        ..TestOwners::default()
    };
    let result = route_operator_request(
        OperatorRequest::AuthorizedCommand {
            command_ref: "command:approved".into(),
            action_identity: "action:approved".into(),
        },
        &authorized_owners,
        &mut commands,
    );
    assert_eq!(result.state, ProjectionState::Observed);
    assert_eq!(commands.writes, vec!["command:approved"]);
}

#[test]
fn agy_01_agreement_collapses_shared_dependence_without_creating_approval() {
    let mut recommendations: Vec<_> = (0..10)
        .map(|index| Recommendation {
            recommendation_ref: format!("recommendation:{index}"),
            action_identity: "action:ship".into(),
            provenance_group_ref: "provenance:shared".into(),
            dependence_group_ref: "dependence:shared".into(),
            stance: RecommendationStance::Agree,
        })
        .collect();
    recommendations.push(Recommendation {
        recommendation_ref: "recommendation:dissent".into(),
        action_identity: "action:ship".into(),
        provenance_group_ref: "provenance:dissent".into(),
        dependence_group_ref: "dependence:dissent".into(),
        stance: RecommendationStance::Dissent,
    });
    let owners = TestOwners {
        approval: ApprovalDecision::Required {
            requirement_ref: "approval-required:ship".into(),
        },
        ..TestOwners::default()
    };

    let projection = aggregate_recommendations("action:ship", &recommendations, &owners);
    assert_eq!(projection.agreeing_recommendation_count, 10);
    assert_eq!(projection.independent_agreement_count, 1);
    assert_eq!(projection.provenance_group_refs, vec!["provenance:shared"]);
    assert_eq!(projection.dependence_group_refs, vec!["dependence:shared"]);
    assert_eq!(projection.dissent_refs, vec!["recommendation:dissent"]);
    assert!(matches!(
        projection.approval,
        ApprovalDecision::Required { .. }
    ));
    assert_eq!(projection.state, ProjectionState::Blocked);
}

#[test]
fn agy_02_paraphrased_nudges_share_canonical_action_and_bounded_policy_count() {
    let nudges = vec![
        ActionNudge::new(1, "Please deploy the release"),
        ActionNudge::new(2, "Could you ship it now?"),
        ActionNudge::new(3, "Go ahead with deployment"),
    ];
    let owners = TestOwners {
        action_identities: nudges
            .iter()
            .map(|nudge| {
                (
                    nudge.text.clone(),
                    ActionIdentityDecision::Known {
                        action_identity: "action:deploy-release".into(),
                        classification_ref: "classifier:owner-v1".into(),
                    },
                )
            })
            .collect(),
        ..TestOwners::default()
    };

    let projection = project_action_nudges(
        &nudges,
        &owners,
        InteractionPolicyLimits {
            max_policy_entries: 2,
        },
    );
    assert_eq!(projection.state, ProjectionState::Observed);
    assert_eq!(projection.actions.len(), 1);
    assert_eq!(
        projection.actions[0].action_identity.as_deref(),
        Some("action:deploy-release")
    );
    assert_eq!(projection.actions[0].rounds, vec![1, 2, 3]);
    assert_eq!(projection.interaction_policy_count, 1);
    assert!(projection.interaction_policy_count <= 2);
}

#[test]
fn agy_03_disallowed_persuasion_withholds_sensitive_personalization_without_receipt_leak() {
    let secret_receipt_text = "medical-condition=private persuasion-profile=coercive";
    let feature = SensitiveFeatureInput {
        feature: FeatureReference {
            feature_ref: "feature:personalization-7".into(),
            sensitive_receipt_ref: "receipt:sensitive-7".into(),
        },
        sensitive_receipt_text: secret_receipt_text.into(),
    };
    let owners = TestOwners {
        purpose_decisions: [(
            ("feature:personalization-7".into(), "persuasion".into()),
            PurposeDecision::Withheld {
                disclosure_ref: "disclosure:purpose-block-7".into(),
            },
        )]
        .into_iter()
        .collect(),
        ..TestOwners::default()
    };

    let projection = project_sensitive_features("persuasion", &[feature], &owners);
    assert_eq!(projection[0].state, ProjectionState::Withheld);
    assert_eq!(
        projection[0].disclosure_ref.as_deref(),
        Some("disclosure:purpose-block-7")
    );
    assert_eq!(projection[0].feature_ref, "feature:personalization-7");

    let encoded = serde_json::to_string(&projection).unwrap();
    assert!(!encoded.contains(secret_receipt_text));
    assert!(!encoded.contains("medical-condition"));
    assert!(!encoded.contains("coercive"));
}
