use aidens_governance_kit::{canonical_stack as governance_stack, CanonicalGovernanceAdapter};
use aidens_repair_kit::{canonical_stack as repair_stack, CanonicalRepairAdapter};
use aidens_tool_kit::{ToolDispatcher, ToolInvocationError, ToolRegistryV1};
use semantic_memory_forge::{RetractionRecordV1, RETRACTION_RECORD_V1_SCHEMA};
use stack_ids::{AttemptId, ClaimId, ClaimVersionId, RetractionRecordId, TraceCtx};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "aidens";
const TIMESTAMP: &str = "2026-04-28T12:00:00Z";

#[test]
fn promotion_denies_without_verification() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = CanonicalGovernanceAdapter;
    let case = adapter.claim_version_case(
        NAMESPACE,
        ClaimVersionId::new("claim-version-phase06-model-only"),
        TraceCtx::from_trace_id("trace-phase06-model-only"),
        AttemptId::new("attempt-phase06-model-only"),
        TIMESTAMP,
    );
    let plan = adapter.check_plan(
        &case,
        governance_stack::CheckMethod::ExactBoundedOracle,
        vec!["exact-bounded-oracle".into()],
        governance_stack::PromotionClass::P2,
        governance_stack::ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "promotion requires completed canonical verification",
        serde_json::json!({"requested_by": "phase-06-test"}),
    );
    let failed_attempt = adapter.completed_attempt(
        &case,
        &plan,
        governance_stack::VerificationAttemptState::Failed,
        TIMESTAMP,
        "2026-04-28T12:00:01Z",
        Some("model-only-assertion-without-verification".into()),
    );
    let control_receipt = adapter.control_receipt_for_attempt(
        &case,
        &plan,
        &failed_attempt,
        true,
        serde_json::json!({
            "model_claimed_promotable": true,
            "target_key": case.region.target_key,
        }),
    );
    assert_eq!(
        control_receipt.schema_version,
        verification_control::CONTROL_RECEIPT_V1_SCHEMA
    );
    assert!(!control_receipt.promotable);
    assert!(control_receipt.validate().is_ok());

    let policy = governance_stack::PolicySnapshot::permissive("phase06-policy", TIMESTAMP);
    let policy_decision = adapter.evaluate_policy(&policy, &case, &plan, &[], false, false);
    assert!(policy_decision.promotion_allowed);
    assert!(policy_decision.validate().is_ok());

    let calibration =
        adapter.calibration_snapshot(&case, TIMESTAMP, true, true, 500_000, 100_000, Vec::new());
    assert!(!calibration.forces_advisory_only);
    let adjudication = adapter.adjudicate_case(
        &case,
        &plan,
        &failed_attempt,
        &control_receipt,
        &policy_decision,
        &calibration,
        false,
        false,
        false,
    );

    assert_ne!(
        adjudication.disposition,
        governance_stack::VerificationDisposition::EligibleForPromotion
    );
    assert!(!adjudication.promotion_decision.promotable);
    assert_eq!(
        adjudication.promotion_decision.control_receipt_id,
        Some(control_receipt.receipt_id)
    );
    adjudication.promotion_decision.validate()?;
    Ok(())
}

#[tokio::test]
async fn approval_required_for_side_effect() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = CanonicalGovernanceAdapter;
    let case = adapter.verification_case(
        governance_stack::VerificationCaseClass::QueryTurn,
        NAMESPACE,
        "phase06-side-effect",
        TraceCtx::from_trace_id("trace-phase06-side-effect"),
        AttemptId::new("attempt-phase06-side-effect"),
        TIMESTAMP,
        false,
        false,
    );
    let plan = adapter.check_plan(
        &case,
        governance_stack::CheckMethod::PairedPatch,
        vec!["paired-patch".into()],
        governance_stack::PromotionClass::P2,
        governance_stack::ReversibilityClass::RequiresSupersession,
        true,
        false,
        false,
        "side-effect promotion requires human approval",
        serde_json::json!({"side_effect": "patch_apply"}),
    );
    let mut policy =
        governance_stack::PolicySnapshot::permissive("phase06-approval-policy", TIMESTAMP);
    policy.method_rules = vec![governance_stack::MethodPolicy {
        method: governance_stack::CheckMethod::PairedPatch,
        allowed: true,
        max_autonomy: governance_stack::AutonomyCeiling::PromotionEligible,
        approval_requirement: governance_stack::ApprovalRequirement::HumanReview,
    }];

    let no_approval = adapter.evaluate_policy(&policy, &case, &plan, &[], false, false);
    assert!(no_approval.approval_required);
    assert!(!no_approval.approval_satisfied);
    assert!(no_approval
        .reasons
        .contains(&"human approval is required before execution".into()));

    let succeeded_attempt = adapter.completed_attempt(
        &case,
        &plan,
        governance_stack::VerificationAttemptState::Succeeded,
        TIMESTAMP,
        "2026-04-28T12:00:01Z",
        Some("paired-patch-succeeded".into()),
    );
    let control_receipt = adapter.control_receipt_for_attempt(
        &case,
        &plan,
        &succeeded_attempt,
        true,
        serde_json::json!({"target_key": case.region.target_key}),
    );
    let calibration =
        adapter.calibration_snapshot(&case, TIMESTAMP, true, true, 500_000, 100_000, Vec::new());
    let blocked = adapter.adjudicate_case(
        &case,
        &plan,
        &succeeded_attempt,
        &control_receipt,
        &no_approval,
        &calibration,
        false,
        false,
        false,
    );
    assert_eq!(
        blocked.disposition,
        governance_stack::VerificationDisposition::PendingApproval
    );
    assert!(!blocked.promotion_decision.promotable);

    let approval = adapter.approval_record(
        policy.policy_version.clone(),
        Some(&case),
        Some(&plan),
        governance_stack::ApprovalScope {
            namespace: NAMESPACE.into(),
            target_key: Some(case.region.target_key.clone()),
            method: Some(plan.method),
            promotion_class: Some(plan.promotion_class),
            expires_at: Some("2026-04-29T12:00:00Z".into()),
            reusable: false,
        },
        "phase06-human-reviewer",
        TIMESTAMP,
        vec![control_receipt.receipt_id.to_string()],
        "reviewed paired patch side effect",
    );
    approval.validate()?;
    let approved = adapter.evaluate_policy(&policy, &case, &plan, &[approval], false, false);
    assert!(approved.approval_required);
    assert!(approved.approval_satisfied);

    let root = temp_root("phase-06-side-effect-approval");
    let repo = root.join("repo");
    std::fs::create_dir_all(&repo)?;
    std::fs::write(repo.join("README.md"), "before\n")?;
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-before\n+after\n";
    let denied = ToolDispatcher::new(ToolRegistryV1::safe_coding_with_dispatchers(&repo)?)
        .invoke("aidens:patch-apply:1", serde_json::json!({ "diff": diff }))
        .await
        .expect_err("side-effect tool must require explicit permit/approval");
    let denied = denied
        .downcast_ref::<ToolInvocationError>()
        .expect("typed AiDENs invocation error");
    assert!(denied.approval_request().is_some());
    assert!(denied.receipt().approval_request_id.is_some());
    assert_eq!(std::fs::read_to_string(repo.join("README.md"))?, "before\n");

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn repair_record_backpointer() -> Result<(), Box<dyn std::error::Error>> {
    let governance = CanonicalGovernanceAdapter;
    let case = governance.claim_version_case(
        NAMESPACE,
        ClaimVersionId::new("claim-version-phase06-repair"),
        TraceCtx::from_trace_id("trace-phase06-repair"),
        AttemptId::new("attempt-phase06-repair"),
        TIMESTAMP,
    );
    let plan = governance.check_plan(
        &case,
        governance_stack::CheckMethod::CausalRefuter,
        vec!["causal-refuter".into()],
        governance_stack::PromotionClass::P1,
        governance_stack::ReversibilityClass::RequiresSupersession,
        true,
        false,
        false,
        "repair backpointer test uses canonical control receipt",
        serde_json::json!({"phase": "06"}),
    );
    let attempt = governance.completed_attempt(
        &case,
        &plan,
        governance_stack::VerificationAttemptState::Succeeded,
        TIMESTAMP,
        "2026-04-28T12:00:01Z",
        Some("repair-record-backpointer".into()),
    );
    let receipt = governance.control_receipt_for_attempt(
        &case,
        &plan,
        &attempt,
        false,
        serde_json::json!({
            "target_key": case.region.target_key,
            "source": "phase-06-repair-record-backpointer",
        }),
    );

    let repair = CanonicalRepairAdapter.boundary_repair_record(
        repair_stack::BoundaryArtifactKind::ControlReceipt,
        verification_control::CONTROL_RECEIPT_V1_SCHEMA,
        "backpointer-preservation",
        "$.details.repair_backpointer",
        None,
        serde_json::json!({
            "source_control_receipt_id": receipt.receipt_id.to_string(),
            "source_case_id": case.case_id.to_string(),
            "source_plan_id": plan.plan_id.to_string(),
            "source_attempt_id": attempt.attempt_id.to_string(),
            "replay_ref": format!("control-receipt://{}", receipt.receipt_id),
        }),
        format!("preserve replay lineage for {}", receipt.receipt_id),
    );
    assert_eq!(
        repair.schema_version,
        repair_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA
    );
    assert_eq!(
        repair.repaired_value["source_control_receipt_id"],
        receipt.receipt_id.to_string()
    );
    assert!(repair.repaired_value["replay_ref"]
        .as_str()
        .unwrap_or_default()
        .contains(receipt.receipt_id.as_str()));

    let retraction = RetractionRecordV1 {
        schema_version: RETRACTION_RECORD_V1_SCHEMA.into(),
        retraction_record_id: RetractionRecordId::new("retraction-phase06-repair"),
        claim_id: ClaimId::new("claim-phase06-repair"),
        retracted_claim_version_id: ClaimVersionId::new("claim-version-phase06-repair"),
        superseded_by_claim_version_id: None,
        effective_recorded_at: TIMESTAMP.into(),
        reason: format!(
            "boundary_repair_record={} control_receipt={}",
            repair.repair_record_id, receipt.receipt_id
        ),
        cascade_required: true,
        delta_summary: Some(
            serde_json::json!({
                "boundary_repair_record_id": repair.repair_record_id.to_string(),
                "control_receipt_id": receipt.receipt_id.to_string(),
                "replay_ref": format!("control-receipt://{}", receipt.receipt_id),
            })
            .to_string(),
        ),
    };
    CanonicalRepairAdapter.validate_retraction(&retraction)?;
    assert!(retraction
        .delta_summary
        .as_deref()
        .unwrap_or_default()
        .contains(receipt.receipt_id.as_str()));

    Ok(())
}

fn temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
