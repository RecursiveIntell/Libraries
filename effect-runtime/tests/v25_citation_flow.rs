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
    ProfileExceptionBundleId, ProfileSetId,
};

#[test]
fn v25_effect_artifacts_roundtrip_with_citation_payload() {
    let citation = V25ConstitutionCitation {
        applicability_context_id: ApplicabilityContextId::new("ac-v25"),
        profile_set_id: ProfileSetId::new("ps-v25"),
        composition_receipt_id: CompositionReceiptId::new("cr-v25"),
        effective_constitution_id: stack_ids::EffectiveConstitutionId::new("ec-v25"),
        compiled_obligation_set_id: CompiledObligationSetId::new("cos-v25"),
        composition_conflict_set_id: Some(CompositionConflictSetId::new("ccs-v25")),
        profile_exception_bundle_ids: vec![ProfileExceptionBundleId::new("peb-v25")],
    };

    let obligation_refs = V25ObligationRefs {
        required_obligation_refs: vec!["obligation:required:v25".into()],
        blocking_obligation_refs: vec!["obligation:blocking:v25".into()],
        monitoring_obligation_refs: vec!["obligation:monitoring:v25".into()],
        decision_basis_obligation_refs: vec!["obligation:decision:v25".into()],
    };

    let intent = EffectIntentV1::builder(
        EffectIntentId::new("fxi-v25"),
        EffectClassV1::ExternalWrite,
        "service/checkout",
        "apply safe state transition",
        vec!["artifact:runbook".to_string()],
        "cap-change",
        "project-alpha",
        BlastRadiusCeilingV1::SingleTenant,
        ReversibilityClassV1::Compensatable,
        "idem-v25",
        EffectWindowId::new("fxw-v25"),
        vec!["obs:metric-latency".to_string()],
        vec!["approval:policy".to_string()],
        RunModeV1::Live,
        PublicationStatusV1::Required,
    )
    .build()
    .expect("build intent");

    let preflight = EffectPreflightReportV1::builder(
        EffectPreflightReportId::new("fxp-v25"),
        EffectIntentId::new("fxi-v25"),
        citation.clone(),
        obligation_refs.clone(),
        CheckResultV1::Pass,
        CheckResultV1::Pass,
        CheckResultV1::Pass,
        BudgetSufficiencyResultV1::Sufficient,
        EffectPreflightDispositionV1::CommitEligible,
        "2026-03-18T00:00:00Z",
        vec!["compositional lane attached".to_string()],
    )
    .build()
    .expect("build preflight");

    let commit = EffectCommitDecisionV1::builder(
        EffectCommitDecisionId::new("fxc-v25"),
        EffectIntentId::new("fxi-v25"),
        EffectPreflightReportId::new("fxp-v25"),
        citation.clone(),
        obligation_refs.clone(),
        vec!["approval:review".to_string()],
        EffectCommitDispositionV1::Authorized,
        vec!["person:policy_officer".to_string()],
        Some(ExecutionPermitId::new("permit-v25")),
        Some(ApprovalGrantId::new("grant-v25")),
        "",
        "2026-03-18T00:00:30Z",
    )
    .build()
    .expect("build commit");

    let execution = EffectExecutionReceiptV1::builder(
        EffectExecutionReceiptId::new("fxe-v25"),
        EffectCommitDecisionId::new("fxc-v25"),
        citation.clone(),
        vec!["runtime-orchestrator".to_string()],
        ExecutionStateV1::Completed,
        false,
        "",
        false,
        vec!["effect:change:v25".to_string()],
        "external-handle-v25",
        "2026-03-18T00:01:00Z",
    )
    .build()
    .expect("build execution");

    let observation = EffectObservationBundleV1::builder(
        EffectObservationBundleId::new("fxo-v25"),
        EffectExecutionReceiptId::new("fxe-v25"),
        citation.clone(),
        "2026-03-18T00:01:00Z/2026-03-18T00:02:00Z",
        ObservationStateV1::Complete,
        "change applied",
        "no drift",
        vec!["evidence:metric-latency".to_string()],
        ClosureRecommendationV1::CloseSuccessfully,
        "2026-03-18T00:02:00Z",
    )
    .build()
    .expect("build observation");

    let window = EffectWindowV1::builder(
        EffectWindowId::new("fxw-v25"),
        "2026-03-18T00:00:00Z",
        "2026-03-18T00:00:05Z",
        "2026-03-18T00:05:00Z",
        CommitAtomicityV1::ExactlyOnce,
        RetryOwnerV1::Verification,
        effect_runtime::CloseMidflightBehaviorV1::FailClosed,
    )
    .build()
    .expect("build window");

    let compensation_plan = CompensationPlanV1::builder(
        stack_ids::CompensationPlanId::new("cpl-v25"),
        EffectIntentId::new("fxi-v25"),
        citation.clone(),
        false,
        CompensationClassV1::LocalRollback,
        vec!["no action needed".to_string()],
        vec!["verified outcome".to_string()],
        "team:runtime",
        "2026-03-18T00:02:00Z",
        false,
    )
    .build()
    .expect("build compensation plan");

    let compensation_receipt = CompensationExecutionReceiptV1::builder(
        stack_ids::CompensationExecutionReceiptId::new("cex-v25"),
        stack_ids::CompensationPlanId::new("cpl-v25"),
        citation.clone(),
        ExecutionStateV1::Completed,
        vec!["restored".to_string()],
        Vec::new(),
        "none",
        "2026-03-18T00:03:00Z",
    )
    .build()
    .expect("build compensation receipt");

    let ledger = ExternalEffectLedgerEntryV1::builder(
        stack_ids::ExternalEffectLedgerEntryId::new("xel-v25"),
        EffectExecutionReceiptId::new("fxe-v25"),
        citation.clone(),
        "core",
        "ext-handle-v25",
        ExternalObservationStateV1::Committed,
        "2026-03-18T00:01:30Z",
        true,
    )
    .build()
    .expect("build ledger");

    let value = serde_json::to_string(&intent).unwrap();
    let intent_rt: EffectIntentV1 = serde_json::from_str(&value).unwrap();
    intent_rt.validate().unwrap();
    let value = serde_json::to_string(&preflight).unwrap();
    let preflight_rt: EffectPreflightReportV1 = serde_json::from_str(&value).unwrap();
    preflight_rt.validate().unwrap();
    let value = serde_json::to_string(&commit).unwrap();
    let commit_rt: EffectCommitDecisionV1 = serde_json::from_str(&value).unwrap();
    commit_rt.validate().unwrap();
    let value = serde_json::to_string(&execution).unwrap();
    let execution_rt: EffectExecutionReceiptV1 = serde_json::from_str(&value).unwrap();
    execution_rt.validate().unwrap();
    let value = serde_json::to_string(&observation).unwrap();
    let observation_rt: EffectObservationBundleV1 = serde_json::from_str(&value).unwrap();
    observation_rt.validate().unwrap();
    let value = serde_json::to_string(&compensation_plan).unwrap();
    let compensation_plan_rt: CompensationPlanV1 = serde_json::from_str(&value).unwrap();
    compensation_plan_rt.validate().unwrap();
    let value = serde_json::to_string(&compensation_receipt).unwrap();
    let compensation_receipt_rt: CompensationExecutionReceiptV1 =
        serde_json::from_str(&value).unwrap();
    compensation_receipt_rt.validate().unwrap();
    let value = serde_json::to_string(&window).unwrap();
    let window_rt: EffectWindowV1 = serde_json::from_str(&value).unwrap();
    window_rt.validate().unwrap();
    let value = serde_json::to_string(&ledger).unwrap();
    let ledger_rt: ExternalEffectLedgerEntryV1 = serde_json::from_str(&value).unwrap();
    ledger_rt.validate().unwrap();

    assert_eq!(
        execution.citation.composition_receipt_id,
        citation.composition_receipt_id
    );
    assert_eq!(
        commit.obligation_refs.required_obligation_refs,
        obligation_refs.required_obligation_refs
    );
}
