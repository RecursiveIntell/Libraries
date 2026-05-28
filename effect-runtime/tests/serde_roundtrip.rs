#![allow(clippy::expect_used)]

use effect_runtime::{
    BlastRadiusCeilingV1, BudgetSufficiencyResultV1, CheckResultV1, ClosureRecommendationV1,
    CommitAtomicityV1, CompensationClassV1, CompensationExecutionReceiptV1, CompensationPlanV1,
    EffectClassV1, EffectCommitDecisionV1, EffectCommitDispositionV1, EffectExecutionReceiptV1,
    EffectIntentV1, EffectObservationBundleV1, EffectPreflightDispositionV1,
    EffectPreflightReportV1, EffectWindowV1, ExecutionStateV1, ExternalEffectLedgerEntryV1,
    ExternalObservationStateV1, ObservationStateV1, PublicationStatusV1, RetryOwnerV1,
    ReversibilityClassV1, RunModeV1, V25ConstitutionCitation, V25ObligationRefs,
};
use stack_ids::{
    ApplicabilityContextId, ApprovalGrantId, CompiledObligationSetId, CompositionConflictSetId,
    CompositionReceiptId, EffectCommitDecisionId, EffectExecutionReceiptId, EffectIntentId,
    EffectObservationBundleId, EffectPreflightReportId, EffectWindowId, ExecutionPermitId,
    ExternalEffectLedgerEntryId, ProfileExceptionBundleId, ProfileSetId,
};

fn sample_citation() -> V25ConstitutionCitation {
    V25ConstitutionCitation {
        applicability_context_id: ApplicabilityContextId::new("ac_001"),
        profile_set_id: ProfileSetId::new("ps_001"),
        composition_receipt_id: CompositionReceiptId::new("cr_001"),
        effective_constitution_id: stack_ids::EffectiveConstitutionId::new("ec_001"),
        compiled_obligation_set_id: CompiledObligationSetId::new("cos_001"),
        composition_conflict_set_id: Some(CompositionConflictSetId::new("ccs_001")),
        profile_exception_bundle_ids: vec![ProfileExceptionBundleId::new("peb_001")],
    }
}

fn sample_obligation_refs() -> V25ObligationRefs {
    V25ObligationRefs {
        required_obligation_refs: vec!["obs:admission-check-ok".into()],
        blocking_obligation_refs: Vec::new(),
        monitoring_obligation_refs: vec!["obs:latency".into()],
        decision_basis_obligation_refs: vec!["obs:policy-compliance".into()],
    }
}

#[test]
fn serde_roundtrip_example() {
    let citation = sample_citation();
    let obligation_refs = sample_obligation_refs();

    let window = EffectWindowV1::builder(
        EffectWindowId::new("fxw_001"),
        "2026-03-15T12:00:00Z",
        "2026-03-15T12:10:00Z",
        "2026-03-15T12:30:00Z",
        CommitAtomicityV1::ExactlyOnce,
        RetryOwnerV1::ForgeOrchestration,
        effect_runtime::CloseMidflightBehaviorV1::FailClosed,
    )
    .build()
    .expect("build window");

    let intent = EffectIntentV1::builder(
        EffectIntentId::new("fxi_001"),
        EffectClassV1::ExternalWrite,
        "ticketing/api",
        "apply approved configuration change",
        vec!["episode:ep_100".to_string(), "plan:pl_200".to_string()],
        "cap_change_window",
        "project:alpha",
        BlastRadiusCeilingV1::SingleTenant,
        ReversibilityClassV1::Compensatable,
        "idem_001",
        window.effect_window_id.clone(),
        vec!["obs:latency".to_string(), "obs:error_budget".to_string()],
        vec!["approval:chg_881".to_string()],
        RunModeV1::Live,
        PublicationStatusV1::Required,
    )
    .build()
    .expect("build intent");

    let preflight = EffectPreflightReportV1::builder(
        EffectPreflightReportId::new("fxp_001"),
        intent.effect_intent_id.clone(),
        citation.clone(),
        obligation_refs.clone(),
        CheckResultV1::Pass,
        CheckResultV1::Pass,
        CheckResultV1::Pass,
        BudgetSufficiencyResultV1::Sufficient,
        EffectPreflightDispositionV1::CommitEligible,
        "2026-03-15T12:00:00Z",
        vec!["all preconditions satisfied".to_string()],
    )
    .build()
    .expect("build preflight");

    let commit = EffectCommitDecisionV1::builder(
        EffectCommitDecisionId::new("fxc_001"),
        intent.effect_intent_id.clone(),
        preflight.effect_preflight_report_id.clone(),
        citation.clone(),
        obligation_refs.clone(),
        vec!["approval:chaperone".to_string()],
        EffectCommitDispositionV1::Authorized,
        vec!["person:alice".to_string(), "person:bob".to_string()],
        Some(ExecutionPermitId::new("permit_001")),
        Some(ApprovalGrantId::new("grant_001")),
        "",
        "2026-03-15T12:02:00Z",
    )
    .build()
    .expect("build commit");

    let execution_receipt = EffectExecutionReceiptV1::builder(
        EffectExecutionReceiptId::new("fxr_001"),
        commit.effect_commit_decision_id.clone(),
        citation.clone(),
        vec![
            "forge-pilot".to_string(),
            "llm-tool-runtime".to_string(),
            "ticketing/api".to_string(),
        ],
        ExecutionStateV1::Completed,
        false,
        "",
        false,
        vec!["ticket:chg-881".to_string()],
        "ext_8821",
        "2026-03-15T12:08:00Z",
    )
    .build()
    .expect("build execution receipt");

    let observation_bundle = EffectObservationBundleV1::builder(
        EffectObservationBundleId::new("fxo_001"),
        execution_receipt.effect_execution_receipt_id.clone(),
        citation.clone(),
        "2026-03-15T12:08:00Z/2026-03-15T13:08:00Z",
        ObservationStateV1::Complete,
        "change applied without rollback",
        "no material drift observed",
        vec!["metric:latency_1h".to_string(), "log:event_991".to_string()],
        ClosureRecommendationV1::CloseSuccessfully,
        "2026-03-15T13:09:00Z",
    )
    .build()
    .expect("build observation bundle");

    let compensation_plan = CompensationPlanV1::builder(
        stack_ids::CompensationPlanId::new("fxcp_001"),
        intent.effect_intent_id.clone(),
        citation.clone(),
        true,
        CompensationClassV1::StateRestore,
        vec![
            "reapply prior config".to_string(),
            "notify operator".to_string(),
        ],
        vec![
            "prior state restored".to_string(),
            "post-check green".to_string(),
        ],
        "team:ops",
        "2026-03-15T12:40:00Z",
        false,
    )
    .build()
    .expect("build compensation plan");

    let compensation_receipt = CompensationExecutionReceiptV1::builder(
        stack_ids::CompensationExecutionReceiptId::new("fxcr_001"),
        compensation_plan.compensation_plan_id.clone(),
        citation.clone(),
        ExecutionStateV1::Completed,
        vec![
            "reapply prior config".to_string(),
            "notify operator".to_string(),
        ],
        Vec::new(),
        "none",
        "2026-03-15T12:28:00Z",
    )
    .build()
    .expect("build compensation receipt");

    let ledger = ExternalEffectLedgerEntryV1::builder(
        ExternalEffectLedgerEntryId::new("fxl_001"),
        execution_receipt.effect_execution_receipt_id.clone(),
        citation,
        "ticketing",
        "ext_8821",
        ExternalObservationStateV1::Committed,
        "2026-03-15T12:08:30Z",
        true,
    )
    .build()
    .expect("build ledger");

    let value = serde_json::to_string(&intent).expect("serialize intent");
    let intent_rt: EffectIntentV1 = serde_json::from_str(&value).expect("deserialize intent");
    intent_rt.validate().expect("validate intent");

    let value = serde_json::to_string(&preflight).expect("serialize preflight");
    let preflight_rt: EffectPreflightReportV1 =
        serde_json::from_str(&value).expect("deserialize preflight");
    preflight_rt.validate().expect("validate preflight");

    let value = serde_json::to_string(&commit).expect("serialize commit");
    let commit_rt: EffectCommitDecisionV1 =
        serde_json::from_str(&value).expect("deserialize commit");
    commit_rt.validate().expect("validate commit");

    let value = serde_json::to_string(&execution_receipt).expect("serialize execution");
    let execution_rt: EffectExecutionReceiptV1 =
        serde_json::from_str(&value).expect("deserialize execution");
    execution_rt.validate().expect("validate execution");

    let value = serde_json::to_string(&observation_bundle).expect("serialize observation");
    let observation_rt: EffectObservationBundleV1 =
        serde_json::from_str(&value).expect("deserialize observation");
    observation_rt.validate().expect("validate observation");

    let value = serde_json::to_string(&compensation_plan).expect("serialize plan");
    let compensation_plan_rt: CompensationPlanV1 =
        serde_json::from_str(&value).expect("deserialize plan");
    compensation_plan_rt.validate().expect("validate plan");

    let value =
        serde_json::to_string(&compensation_receipt).expect("serialize compensation receipt");
    let compensation_receipt_rt: CompensationExecutionReceiptV1 =
        serde_json::from_str(&value).expect("deserialize compensation receipt");
    compensation_receipt_rt
        .validate()
        .expect("validate compensation receipt");

    let value = serde_json::to_string(&window).expect("serialize window");
    let window_rt: EffectWindowV1 = serde_json::from_str(&value).expect("deserialize window");
    window_rt.validate().expect("validate window");

    let value = serde_json::to_string(&ledger).expect("serialize ledger");
    let ledger_rt: ExternalEffectLedgerEntryV1 =
        serde_json::from_str(&value).expect("deserialize ledger");
    ledger_rt.validate().expect("validate ledger");
}
