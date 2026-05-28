#![allow(clippy::expect_used)]

use authority_delegation::{
    ActingOnBehalfReceiptV1, AuthorityChainV1, AuthorityChainValidityStateV1,
    AuthorityValidationError, BreakGlassGrantV1, DualControlApprovalV1,
};

#[test]
fn authority_chain_requires_approval_refs() {
    let err = AuthorityChainV1::new(
        "auc_001",
        "agent:ops-runner",
        vec!["person:alice".to_string()],
        Vec::new(),
        Vec::new(),
        AuthorityChainValidityStateV1::Valid,
    )
    .expect_err("authority chain without approval refs must fail");
    assert_eq!(err, AuthorityValidationError::MissingField("approval_refs"));
}

#[test]
fn dual_control_requires_two_approvers() {
    let err = DualControlApprovalV1::new(
        "dca_001",
        vec!["person:alice".to_string()],
        "matched change plan",
        true,
        "2026-03-15T12:01:00Z",
    )
    .expect_err("single approver must fail");
    assert_eq!(err, AuthorityValidationError::InvalidState("approver_refs"));
}

#[test]
fn emergency_and_receipt_paths_validate_when_well_formed() {
    let grant = BreakGlassGrantV1::new(
        "bgg_001",
        "aul_001",
        "production incident",
        vec!["person:carol".to_string()],
        "2026-03-15T12:45:00Z",
        true,
    )
    .expect("grant");
    grant.validate().expect("grant valid");

    let receipt = ActingOnBehalfReceiptV1::new(
        "aob_001",
        "auc_001",
        "fxr_001",
        "execute change window",
        true,
        "2026-03-15T12:08:10Z",
    )
    .expect("receipt");
    receipt.validate().expect("receipt valid");
}
