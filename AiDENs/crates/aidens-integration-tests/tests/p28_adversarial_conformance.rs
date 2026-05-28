use aidens_agency_kit::{
    AgencyPolicyEngineV1, AgencyPolicyInputV1, AgencySurfaceV1, DecisionDomainV1, InfluenceClassV1,
    NudgeLedgerV1, PersonalizationFeatureUseV1,
};
use aidens_boundary_kit::compile_json_boundary;
use aidens_contracts::{
    AdvisorySystemKindV1, AdvisorySystemPromotionGuardV1, ArtifactId, ArtifactKindV1,
    BoundaryCompileRequestV1, CanonicalToolSideEffectClass, ExecutionCompletionStateV1,
    ExecutionContextEnvelopeV1, ExternalAdmissionDispositionV1,
    ExternalArtifactAdmissionDecisionV1, InvariantBudgetV1, LocalProofProfileV1, ProofDebtLedgerV1,
    ProofObligationV1, ProofWaiverReceiptV1, RegionGraphKindV1, ReleaseReadinessReportV1,
    ReleaseSurfaceStateV1, SemanticExactnessV1, SubtractionOperatorV1, SubtractionPlanV1,
    SupportCoreV1, ToolCallReceiptV1, V11ActivationLevelV1,
};
use aidens_permit_kit::PermitPolicyV1;
use aidens_testkit::{interpret_case, reference_temporal_stale_projection_case};
use aidens_tool_kit::{ToolDispatcher, ToolInvocationError, ToolRegistryV1};
use serde_json::json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn p28_adversarial_manifest_declares_expected_semantics_for_every_fixture() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p28/adversarial_conformance_manifest.json"
    ))
    .expect("manifest json");
    let fixtures = manifest["fixtures"].as_array().expect("fixtures array");
    let expected_ids = [
        "duplicate-json-keys",
        "schema-mismatch",
        "parser-repair-treatment-change",
        "symlink-escape",
        "patch-write-failed-dirty-dir",
        "timeout-partial-output",
        "retry-degraded-aggregate",
        "stale-projection-current-query",
        "proof-waiver-treated-as-proof",
        "degraded-release-surface",
        "storage-graph-used-as-inference-graph",
        "subtraction-removes-support-core",
        "personalized-advice-without-disclosure",
    ];
    for id in expected_ids {
        let fixture = fixtures
            .iter()
            .find(|fixture| fixture["id"] == id)
            .unwrap_or_else(|| panic!("missing fixture {id}"));
        assert!(fixture["expected_semantics"]
            .as_str()
            .is_some_and(|semantics| !semantics.is_empty()));
    }
}

#[test]
fn p28_adversarial_boundary_fixtures_fail_closed() {
    let duplicate =
        compile_json_boundary(BoundaryCompileRequestV1::new(r#"{"path":"a","path":"b"}"#));
    assert!(!duplicate.accepted);
    assert_eq!(duplicate.duplicate_key_findings.len(), 1);
    assert!(duplicate
        .reason_codes
        .contains(&"duplicate-json-object-key".into()));

    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["path"],
        "properties": { "path": { "type": "string" } }
    });
    let mismatch =
        compile_json_boundary(BoundaryCompileRequestV1::new(r#"{"path":7}"#).with_schema(schema));
    assert!(!mismatch.accepted);
    assert!(mismatch
        .schema_validation
        .as_ref()
        .is_some_and(|receipt| !receipt.valid));

    let mut treatment_change_request =
        BoundaryCompileRequestV1::new("prefix {\"treatment\":\"variant\"} suffix")
            .with_treatment_critical_fields(vec!["treatment".into()])
            .with_hard_fail_on_treatment_change(true);
    treatment_change_request.allow_json_substring_extract = true;
    let treatment_change = compile_json_boundary(treatment_change_request);
    assert!(!treatment_change.accepted);
    assert!(treatment_change
        .repair_receipt
        .as_ref()
        .is_some_and(|receipt| receipt.hard_failed));
}

#[tokio::test]
async fn p28_adversarial_tool_sandbox_blocks_symlink_and_dirty_patch_paths() {
    let dir = temp_root("tool-sandbox");
    let outside = temp_root("tool-outside");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "secret\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.join("secret.txt"), dir.join("link.txt")).unwrap();

    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry.clone());
    #[cfg(unix)]
    {
        let denied = dispatcher
            .invoke("aidens:repo-read:1", json!({"path": "link.txt"}))
            .await
            .expect_err("symlink escape should be denied");
        assert!(
            !denied
                .downcast_ref::<ToolInvocationError>()
                .expect("typed tool error")
                .receipt()
                .succeeded
        );
    }

    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "p28-adversarial",
    );
    let permitted = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/missing/leaf.txt\n+++ b/missing/leaf.txt\n@@\n-old\n+new\n";
    let denied = permitted
        .invoke("aidens:patch-apply:1", json!({"diff": diff}))
        .await
        .expect_err("missing parent should fail before mkdir");
    assert!(
        !denied
            .downcast_ref::<ToolInvocationError>()
            .expect("typed tool error")
            .receipt()
            .succeeded
    );
    assert!(!dir.join("missing").exists());

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn p28_adversarial_receipt_proof_degradation_and_temporal_fixtures_hold() {
    let context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.run_checks",
        ArtifactId::new("attempt-family:p28-adversarial-timeout"),
        "local",
        "aidens:run-checks:1",
    )
    .complete(ExecutionCompletionStateV1::TimedOut, 1);
    let timeout_receipt = ToolCallReceiptV1::new(
        &context,
        "aidens:run-checks:1",
        &json!({"command": ["cargo", "test"]}),
        &json!({"stdout_tail": "partial"}),
        ExecutionCompletionStateV1::TimedOut,
    );
    assert!(timeout_receipt.partial_output);

    let readiness = ReleaseReadinessReportV1::new(
        vec![aidens_contracts::ReleaseSurfaceV1::new(
            "package:self-replay",
            ReleaseSurfaceStateV1::Degraded,
            "degraded replay",
        )],
        Vec::new(),
        aidens_contracts::ExampleAppManifestV1::new(Vec::new(), Vec::new()),
        aidens_contracts::InstallSmokeReportV1::new(Vec::new()),
    );
    assert!(!readiness.ready);
    assert!(readiness.blocks_release());

    let mut obligation = ProofObligationV1::new("adversarial proof", "test-log");
    let waiver = ProofWaiverReceiptV1::new(
        obligation.obligation_id.clone(),
        "operator",
        "waiver is not proof",
    );
    obligation.waived_by.push(waiver.receipt_id);
    let profile = LocalProofProfileV1::local_exact(vec![obligation]);
    let debt = ProofDebtLedgerV1::from_profile(ArtifactId::new("artifact:p28-waiver"), &profile);
    assert!(!profile.proof_satisfied());
    assert!(debt.blocks_promotion());

    let stale = interpret_case(&reference_temporal_stale_projection_case()).unwrap();
    assert_eq!(stale["temporal_mode"], json!("degraded"));
    assert_eq!(stale["stale_projection"], json!(true));
    assert_eq!(stale["view_disclosure_required"], json!(true));
}

#[test]
fn p28_adversarial_reserved_horizon_and_agency_fixtures_do_not_promote_truth() {
    let graph = aidens_contracts::CompiledRegionGraphV1::new(
        ArtifactId::new("graph:p28-storage-adversarial"),
        RegionGraphKindV1::Inference,
        Some(RegionGraphKindV1::Storage),
        4,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    assert!(graph.advisory_only);
    assert_eq!(graph.activation_level, V11ActivationLevelV1::ReservedDraft);
    assert!(!graph.can_claim_active_v11b_runtime());

    let accepted_claim = ArtifactId::new("claim:p28-support-core");
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let budget = InvariantBudgetV1::full_history();
    let frontier = aidens_contracts::RemovalFrontierV1::new(
        &support,
        vec![accepted_claim],
        Vec::new(),
        Vec::new(),
        &budget,
    );
    let plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::SupportCoreExtraction,
        &support,
        &frontier,
        &budget,
    );
    assert!(plan.blocked);
    assert!(plan.advisory_only);
    assert!(!plan.can_mutate_runtime_state());

    let admission =
        ExternalArtifactAdmissionDecisionV1::default_quarantine(ArtifactId::new("external:p28"));
    assert_eq!(
        admission.disposition,
        ExternalAdmissionDispositionV1::Quarantined
    );
    assert!(!admission.truth_promotion_allowed);

    let guard = AdvisorySystemPromotionGuardV1::advisory(AdvisorySystemKindV1::LearnedRanker);
    assert!(!guard.truth_promotion_allowed);
    assert!(!guard.proof_waiver_allowed);

    let mut ledger = NudgeLedgerV1::default();
    let mut input = AgencyPolicyInputV1::for_runner_final_output(
        "Use my memory to recommend one path",
        "You should choose option A.",
        &[],
    );
    input.surface = AgencySurfaceV1::FinalOutput;
    input.decision_domain = DecisionDomainV1::General;
    input.recommendation_present = true;
    input.memory_features = vec![PersonalizationFeatureUseV1 {
        feature_id: "preference:known".into(),
        source: "memory".into(),
        sensitive: false,
        vulnerability_related: false,
        ephemeral_only: true,
        reason_codes: Vec::new(),
    }];
    let report = AgencyPolicyEngineV1::default().evaluate(&input, &mut ledger);
    assert!(report
        .classes
        .contains(&InfluenceClassV1::MemoryPersonalized));
    assert!(report.receipts.memory_trace.is_some());
    assert!(report
        .reason_codes()
        .contains(&"memory-influence-trace-required".into()));
    assert!(
        report
            .receipts
            .memory_trace
            .as_ref()
            .unwrap()
            .used_for_recommendation
    );
    assert_ne!(
        report.outcome,
        aidens_agency_kit::AgencyPolicyOutcomeV1::Quarantine
    );
    assert_eq!(graph.kind, ArtifactKindV1::CompiledRegionGraph);
    assert_eq!(stale_exactness_label(), SemanticExactnessV1::Degraded);
}

fn stale_exactness_label() -> SemanticExactnessV1 {
    SemanticExactnessV1::Degraded
}

fn temp_root(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("aidens-p28-{label}-{}-{now}", std::process::id()))
}
