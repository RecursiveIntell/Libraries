//! Approval admission must compare instants and reject malformed/future authority.
use stack_ids::{AttemptId, ScopeKey, TraceCtx};
use verification_control::{
    CaseRegion, CheckMethod, CheckPlan, PromotionClass, ReversibilityClass, VerificationCase,
    VerificationCaseClass,
};
use verification_policy::{approval_matches, ApprovalRecord, ApprovalScope};

fn fixture() -> (VerificationCase, CheckPlan, ApprovalRecord) {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: "isolated".into(),
            scope_key: Some(ScopeKey::namespace_only("isolated")),
            target_key: "candidate".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("approval-time-test"),
        "2026-11-12T08:30:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::PairedPatch,
        vec!["test".into()],
        PromotionClass::P2,
        ReversibilityClass::RequiresSupersession,
        true,
        false,
        false,
        "time admission",
        serde_json::json!({"patch": "fixed"}),
    );
    let approval = ApprovalRecord::new(
        "explicit-policy",
        Some(case.case_id.clone()),
        Some(plan.plan_id.clone()),
        ApprovalScope {
            namespace: "isolated".into(),
            target_key: Some("candidate".into()),
            method: Some(plan.method),
            promotion_class: Some(plan.promotion_class),
            expires_at: Some("2026-11-12T09:00:00Z".into()),
            reusable: false,
        },
        "operator",
        "2026-11-12T08:00:00Z",
        vec!["review:fixed".into()],
        "scoped approval",
    );
    (case, plan, approval)
}

#[test]
fn approval_window_uses_instants_and_a_half_open_interval() {
    let (case, plan, mut approval) = fixture();
    approval.scope.expires_at = Some("2026-11-12T03:00:00-06:00".into());
    assert!(approval_matches(
        &approval,
        &case,
        &plan,
        "2026-11-12T08:30:00Z"
    ));
    assert!(!approval_matches(
        &approval,
        &case,
        &plan,
        "2026-11-12T09:00:00Z"
    ));
    approval.approved_at = "2026-11-12T10:00:00+02:00".into();
    assert!(approval_matches(
        &approval,
        &case,
        &plan,
        "2026-11-12T08:30:00Z"
    ));
}

#[test]
fn future_malformed_and_reversed_approval_windows_refuse() {
    let (case, plan, original) = fixture();
    for (start, end, now) in [
        (
            "2026-11-12T08:31:00Z",
            Some("2026-11-12T09:00:00Z"),
            "2026-11-12T08:30:00Z",
        ),
        ("not-a-time", None, "2026-11-12T08:30:00Z"),
        (
            "2026-11-12T08:00:00Z",
            Some("zz-invalid"),
            "2026-11-12T08:30:00Z",
        ),
        ("2026-11-12T08:00:00Z", None, "invalid-query"),
        (
            "2026-11-12T09:00:00Z",
            Some("2026-11-12T08:00:00Z"),
            "2026-11-12T08:30:00Z",
        ),
    ] {
        let mut approval = original.clone();
        approval.approved_at = start.into();
        approval.scope.expires_at = end.map(str::to_owned);
        assert!(
            !approval_matches(&approval, &case, &plan, now),
            "accepted {start} {end:?} {now}"
        );
    }
}

#[test]
fn invalid_approval_metadata_cannot_authorize_a_matching_scope() {
    let (case, plan, mut approval) = fixture();
    approval.schema_version = "not-an-approval-schema".into();
    assert!(!approval_matches(
        &approval,
        &case,
        &plan,
        "2026-11-12T08:30:00Z"
    ));
}
