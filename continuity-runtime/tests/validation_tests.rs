use continuity_runtime::{
    ContinuityValidationError, EscalationClockPolicyV1, IncidentSeverityV1, IncidentStatusV1,
    IncidentTaxonomyV1, RecoveryReplaySliceV1, RecoveryReplayStateV1, ResilienceExerciseKindV1,
    ResilienceExerciseV1, ServiceLevelProfileV1, SeverityPostmortemClockV1,
    SeverityResponseClockV1,
};

#[test]
fn service_level_profile_requires_owner_ref() {
    let err = ServiceLevelProfileV1::new(
        "slp_001",
        "alpha-service",
        "99.9%",
        "p95<250ms",
        "",
        "ebl_001",
    )
    .expect_err("owner ref is required");
    assert_eq!(err, ContinuityValidationError::MissingField("owner_ref"));
}

#[test]
fn replay_slice_requires_gaps_when_partial() {
    let err = RecoveryReplaySliceV1::new(
        "rrs_001",
        "inc_001",
        vec!["fxr_001".to_string()],
        "2026-03-15T12:30:00Z/2026-03-15T13:00:00Z",
        RecoveryReplayStateV1::Partial,
        Vec::new(),
    )
    .expect_err("partial replay requires gaps");
    assert_eq!(err, ContinuityValidationError::InvalidState("gaps"));
}

#[test]
fn exercise_requires_follow_up_actions() {
    let err = ResilienceExerciseV1::new(
        "rex_001",
        "slp_001",
        ResilienceExerciseKindV1::Tabletop,
        "deploy regression with missing approver",
        "2026-03-10T15:00:00Z",
        Vec::new(),
    )
    .expect_err("exercise follow up actions are required");
    assert_eq!(
        err,
        ContinuityValidationError::MissingField("follow_up_actions")
    );
}

#[test]
fn escalation_policy_rejects_non_positive_clock() {
    let err = EscalationClockPolicyV1::new(
        "ecp_001",
        "sm_001",
        vec![SeverityResponseClockV1 {
            severity: IncidentSeverityV1::Sev1,
            minutes: 0,
        }],
        vec![SeverityPostmortemClockV1 {
            severity: IncidentSeverityV1::Sev1,
            hours: 72,
        }],
        vec!["pause_only_during_declared_external_outage".to_string()],
        "incident-governance-review",
    )
    .expect_err("response clock must be positive");
    assert_eq!(err, ContinuityValidationError::InvalidState("minutes"));
}

#[test]
fn taxonomy_and_status_validate_when_well_formed() {
    let taxonomy = IncidentTaxonomyV1::new(
        "itx_001",
        vec![continuity_runtime::IncidentClassRuleV1 {
            incident_class: "availability_degradation".to_string(),
            default_severity: IncidentSeverityV1::Sev2,
        }],
        vec![continuity_runtime::IncidentRouteRuleV1 {
            incident_class: "availability_degradation".to_string(),
            route: "reliability_primary".to_string(),
        }],
        vec!["IncidentCaseV1".to_string()],
    )
    .expect("taxonomy");
    taxonomy.validate().expect("taxonomy valid");

    let incident = continuity_runtime::IncidentCaseV1::new(
        "inc_001",
        "slp_001",
        IncidentSeverityV1::Sev2,
        "2026-03-15T12:30:00Z",
        "elevated latency after change window",
        "person:oncall-1",
        IncidentStatusV1::Active,
    )
    .expect("incident");
    incident.validate().expect("incident valid");
}
