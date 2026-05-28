#![allow(clippy::expect_used)]

use assurance_runtime::{
    AssuranceCaseV1, AssuranceValidationError, CertificationBundleV1, CertificationStatusV1,
    ControlCoverageStateV1, ControlMappingV1, RecertificationTriggerKindV1,
    RecertificationTriggerV1, ReleaseDecisionStateV1, ReleaseReadinessDecisionV1,
};

#[test]
fn control_mapping_rejects_covered_state_with_gaps() {
    let err = ControlMappingV1::new(
        "ctl_001",
        "hzr_001",
        vec!["rollback".to_string()],
        ControlCoverageStateV1::Covered,
        vec!["gap:manual-approval".to_string()],
    )
    .expect_err("covered mapping with gaps must fail");
    assert_eq!(
        err,
        AssuranceValidationError::InvalidState("covered mapping must not retain gaps")
    );
}

#[test]
fn blocked_release_requires_blocking_gaps() {
    let err = ReleaseReadinessDecisionV1::new(
        "rrd_001",
        "dpf_001",
        ReleaseDecisionStateV1::Blocked,
        Vec::new(),
        Vec::new(),
        true,
        "2026-03-15T12:03:00Z",
    )
    .expect_err("blocked release without gaps must fail");
    assert_eq!(err, AssuranceValidationError::MissingField("blocking_gaps"));
}

#[test]
fn assurance_case_requires_evidence_refs() {
    let err = AssuranceCaseV1::new(
        "asc_001",
        "deployment meets bar",
        Vec::new(),
        "hazard -> control -> evidence",
        vec!["monitor post-deploy".to_string()],
        true,
    )
    .expect_err("assurance case without evidence must fail");
    assert_eq!(err, AssuranceValidationError::MissingField("evidence_refs"));
}

#[test]
fn certification_and_recertification_paths_validate_when_well_formed() {
    let cert = CertificationBundleV1::new(
        "cbn_001",
        "dpf_001",
        "asc_001",
        "rrd_001",
        vec!["team:reliability".to_string()],
        "2026-03-15T12:04:00Z",
        CertificationStatusV1::AdvisoryUntilAdmitted,
    )
    .expect("certification bundle");
    cert.validate().expect("certification bundle valid");

    let trigger = RecertificationTriggerV1::new(
        "rct_001",
        "dpf_001",
        RecertificationTriggerKindV1::SloBreach,
        "2026-03-16T02:00:00Z",
        "reopen assurance case",
    )
    .expect("trigger");
    trigger.validate().expect("trigger valid");
}
