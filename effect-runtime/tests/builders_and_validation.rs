#![allow(clippy::expect_used)]

use effect_runtime::{
    BlastRadiusCeilingV1, BudgetSufficiencyResultV1, CheckResultV1, CommitAtomicityV1,
    CompensationClassV1, CompensationPlanV1, EffectClassV1, EffectCommitDecisionV1,
    EffectCommitDispositionV1, EffectExecutionReceiptV1, EffectIntentV1,
    EffectPreflightDispositionV1, EffectPreflightReportV1, EffectRuntimeValidationError,
    EffectWindowV1, ExecutionStateV1, PublicationStatusV1, RetryOwnerV1, ReversibilityClassV1,
    RunModeV1, V25ConstitutionCitation, V25ObligationRefs,
};
use stack_ids::{
    ApplicabilityContextId, ApprovalGrantId, CompiledObligationSetId, CompositionReceiptId,
    EffectCommitDecisionId, EffectExecutionReceiptId, EffectIntentId, EffectPreflightReportId,
    EffectWindowId, ExecutionPermitId, ProfileSetId,
};

fn sample_citation() -> V25ConstitutionCitation {
    V25ConstitutionCitation {
        applicability_context_id: ApplicabilityContextId::new("ac_test"),
        profile_set_id: ProfileSetId::new("ps_test"),
        composition_receipt_id: CompositionReceiptId::new("cr_test"),
        effective_constitution_id: stack_ids::EffectiveConstitutionId::new("ec_test"),
        compiled_obligation_set_id: CompiledObligationSetId::new("cos_test"),
        composition_conflict_set_id: None,
        profile_exception_bundle_ids: Vec::new(),
    }
}

fn sample_obligation_refs() -> V25ObligationRefs {
    V25ObligationRefs {
        required_obligation_refs: vec!["obligation:required".into()],
        blocking_obligation_refs: Vec::new(),
        monitoring_obligation_refs: Vec::new(),
        decision_basis_obligation_refs: vec!["obligation:basis".into()],
    }
}

#[test]
fn builders_emit_stable_snake_case_wire_values() {
    let window = EffectWindowV1::builder(
        EffectWindowId::new("fxw_test"),
        "2026-03-20T00:00:00Z",
        "2026-03-20T00:01:00Z",
        "2026-03-20T00:05:00Z",
        CommitAtomicityV1::BoundedMultiStep,
        RetryOwnerV1::EffectRuntime,
        effect_runtime::CloseMidflightBehaviorV1::AbortAndEmitReceipt,
    )
    .build()
    .expect("window");
    let intent = EffectIntentV1::builder(
        EffectIntentId::new("fxi_test"),
        EffectClassV1::ExternalWrite,
        "service/api",
        "apply bounded change",
        vec!["episode:test".to_string()],
        "cap:test",
        "project:test",
        BlastRadiusCeilingV1::SingleTenant,
        ReversibilityClassV1::Compensatable,
        "idem_test",
        window.effect_window_id.clone(),
        vec!["obs:test".to_string()],
        vec!["approval:test".to_string()],
        RunModeV1::Simulated,
        PublicationStatusV1::AdvisoryOnly,
    )
    .build()
    .expect("intent");
    let json = serde_json::to_value(intent).expect("serialize");
    assert_eq!(json["effect_class"], "external_write");
    assert_eq!(json["blast_radius_ceiling"], "single_tenant");
    assert_eq!(json["reversibility_class"], "compensatable");
    assert_eq!(json["run_mode"], "simulated");
    assert_eq!(json["publication_status"], "advisory_only");
}

#[test]
fn validation_rejects_authorized_commit_without_permit() {
    let error = EffectCommitDecisionV1::builder(
        EffectCommitDecisionId::new("fxc_test"),
        EffectIntentId::new("fxi_test"),
        EffectPreflightReportId::new("fxp_test"),
        sample_citation(),
        sample_obligation_refs(),
        vec!["approval:required".to_string()],
        EffectCommitDispositionV1::Authorized,
        vec!["person:alice".to_string()],
        None,
        Some(ApprovalGrantId::new("grant_test")),
        "",
        "2026-03-20T00:02:00Z",
    )
    .build()
    .expect_err("authorized commit without permit must fail");
    assert!(matches!(
        error,
        EffectRuntimeValidationError::InvalidState(message)
            if message.contains("execution_permit_id")
    ));
}

#[test]
fn validation_rejects_commit_eligible_preflight_with_failed_checks() {
    let error = EffectPreflightReportV1::builder(
        EffectPreflightReportId::new("fxp_test"),
        EffectIntentId::new("fxi_test"),
        sample_citation(),
        sample_obligation_refs(),
        CheckResultV1::Pass,
        CheckResultV1::Fail,
        CheckResultV1::Pass,
        BudgetSufficiencyResultV1::Sufficient,
        EffectPreflightDispositionV1::CommitEligible,
        "2026-03-20T00:01:00Z",
        vec!["bad".to_string()],
    )
    .build()
    .expect_err("commit-eligible preflight with failed checks must fail");
    assert!(matches!(
        error,
        EffectRuntimeValidationError::InvalidState(message)
            if message.contains("commit-eligible")
    ));
}

#[test]
fn validation_rejects_completed_execution_with_cancellation_reason() {
    let error = EffectExecutionReceiptV1::builder(
        EffectExecutionReceiptId::new("fxe_test"),
        EffectCommitDecisionId::new("fxc_test"),
        sample_citation(),
        vec!["runtime".to_string()],
        ExecutionStateV1::Completed,
        false,
        "operator_cancelled",
        false,
        vec!["effect:test".to_string()],
        "external:test",
        "2026-03-20T00:03:00Z",
    )
    .build()
    .expect_err("completed execution with cancellation reason must fail");
    assert!(matches!(
        error,
        EffectRuntimeValidationError::InvalidState(message)
            if message.contains("cancellation_reason")
    ));
}

#[test]
fn validation_rejects_required_compensation_without_steps() {
    let error = CompensationPlanV1::builder(
        stack_ids::CompensationPlanId::new("cpl_test"),
        EffectIntentId::new("fxi_test"),
        sample_citation(),
        true,
        CompensationClassV1::StateRestore,
        Vec::new(),
        vec!["state restored".to_string()],
        "team:ops",
        "2026-03-20T00:04:00Z",
        false,
    )
    .build()
    .expect_err("required compensation without steps must fail");
    assert!(matches!(
        error,
        EffectRuntimeValidationError::MissingField("compensation_steps")
    ));
}

#[test]
fn validation_accepts_authorized_commit_with_permit_and_grant() {
    let commit = EffectCommitDecisionV1::builder(
        EffectCommitDecisionId::new("fxc_ok"),
        EffectIntentId::new("fxi_ok"),
        EffectPreflightReportId::new("fxp_ok"),
        sample_citation(),
        sample_obligation_refs(),
        vec!["approval:required".to_string()],
        EffectCommitDispositionV1::Authorized,
        vec!["person:alice".to_string()],
        Some(ExecutionPermitId::new("permit_ok")),
        Some(ApprovalGrantId::new("grant_ok")),
        "",
        "2026-03-20T00:02:00Z",
    )
    .build()
    .expect("authorized commit should validate");
    assert_eq!(
        commit.execution_permit_id.as_ref().unwrap().as_str(),
        "permit_ok"
    );
}
