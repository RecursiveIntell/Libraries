use async_trait::async_trait;
use forge_engine::*;
use std::path::Path;

struct UnusedBackend;
#[async_trait]
impl ExecutionBackend for UnusedBackend {
    fn kind(&self) -> ExecutionBackendKind {
        ExecutionBackendKind::Host
    }
    async fn prepare_workspace(&self, _: &Path) -> ForgeResult<Workspace> {
        Err(ForgeError::Other("unused".into()))
    }
    async fn run_command(
        &self,
        _: &Path,
        _: &str,
        _: &[&str],
        _: &[(&str, &str)],
        _: u64,
    ) -> ForgeResult<CommandOutput> {
        Err(ForgeError::Other("unused".into()))
    }
    async fn collect_logs(
        &self,
        _: &CommandOutput,
        _: &CommandOutput,
        _: &CommandOutput,
    ) -> ForgeResult<LogBundle> {
        Err(ForgeError::Other("unused".into()))
    }
}
struct UnusedAdapter;
impl ProjectAdapter for UnusedAdapter {
    fn detect(_: &Path) -> bool
    where
        Self: Sized,
    {
        false
    }
    fn name(&self) -> &str {
        "unused"
    }
    fn check_commands(&self, _: &ForgeConfig) -> Vec<CheckCommand> {
        vec![]
    }
    fn parse_check_output(&self, _: &CheckCommand, _: &str, _: &str, _: i32) -> ParsedCheckOutput {
        ParsedCheckOutput::default()
    }
}
fn descriptor() -> BaselineDescriptor {
    BaselineDescriptor {
        source_kind: BaselineSourceKind::Explicit,
        commit_sha: Some("base".into()),
        dirty: false,
        untracked_count: 0,
        lockfile_hash: None,
        rustc_version: "rustc".into(),
        cargo_version: "cargo".into(),
        target_triple: "host".into(),
        env_fingerprint: "env".into(),
        submodule_state: vec![],
    }
}
fn effect(class: &str) -> LocatedEffect {
    LocatedEffect {
        file: Some("code.rs".into()),
        line: Some(2),
        col: None,
        message: class.into(),
        sig: EffectSignature {
            check_kind: "test".into(),
            outcome: "fail".into(),
            severity: "error".into(),
            message_class: class.into(),
            line_offset_from_edit: None,
        },
    }
}
fn check(effects: Vec<LocatedEffect>) -> CheckResult {
    CheckResult {
        fmt_pass: true,
        clippy_pass: true,
        test_pass: effects.is_empty(),
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput {
            check_kind: CheckKind::Test,
            exit_code: if effects.is_empty() { 0 } else { 1 },
            effects,
            raw_stdout: String::new(),
            raw_stderr: String::new(),
        },
        total_duration_ms: 1,
    }
}
fn patch() -> StructuredPatch {
    StructuredPatch {
        patch_id: uuid::Uuid::nil(),
        summary: "one op".into(),
        notes: vec![],
        edits: vec![FileEdit {
            path: "code.rs".into(),
            mode: None,
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec!["fn main() {}".into()],
                    context_after: vec![],
                },
                lines: vec!["// cause".into()],
            }],
        }],
    }
}
fn pair(baseline: CheckResult, patched: CheckResult) -> PairedTrialResult {
    let mut line_map = LineAttributionMap::default();
    line_map.resolved_anchors.insert(("code.rs".into(), 0), 2);
    PairedTrialResult {
        pair_index: 0,
        baseline_descriptor: descriptor(),
        patched_descriptor: descriptor(),
        baseline_result: baseline.clone(),
        patched_result: patched.clone(),
        line_map,
        diff: ExperimentDiff::from_paired(&baseline, &patched),
        comparable: true,
        comparability_reasons: vec![],
    }
}
fn engine(store: &ForgeStore) -> CausalAttributionEngine<'_> {
    static BACKEND: UnusedBackend = UnusedBackend;
    static ADAPTER: UnusedAdapter = UnusedAdapter;
    let config = Box::leak(Box::new(ForgeConfig::default()));
    CausalAttributionEngine::new(store, &BACKEND, &ADAPTER, config, "v1")
}

#[test]
fn stable_baseline_failure_is_excluded_and_new_effect_is_persisted() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let engine = engine(&store);
    let stable = engine
        .observe_pair(
            &patch(),
            &pair(check(vec![effect("stable")]), check(vec![effect("stable")])),
            "run-stable",
            "eval",
        )
        .unwrap();
    assert_eq!(stable.triple_count, 0);
    assert_eq!(stable.coverage.total_edges, 0);
    let new = engine
        .observe_pair(
            &patch(),
            &pair(
                check(vec![effect("stable")]),
                check(vec![effect("stable"), effect("new")]),
            ),
            "run-new",
            "eval",
        )
        .unwrap();
    assert_eq!(new.triple_count, 1);
    assert_eq!(new.coverage.total_edges, 1);
    assert!(new.verify_integrity());
    let mut tampered = new.clone();
    tampered.triple_count += 1;
    assert!(!tampered.verify_integrity());
}

#[test]
fn prediction_gate_always_returns_run_checks_with_reasons_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let engine = engine(&store);
    let receipt = engine.predict_patch(&[]).unwrap();
    assert_eq!(receipt.gate.disposition, PredictionDisposition::RunChecks);
    assert!(receipt
        .gate
        .reasons
        .contains(&PredictionGateReason::DisabledOptIn));
    assert!(receipt
        .gate
        .reasons
        .contains(&PredictionGateReason::MissingInterventionalEvidence));
}

#[test]
fn receipt_separates_patch_experiment_from_observational_attribution() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let receipt = engine(&store)
        .observe_pair(
            &patch(),
            &pair(check(vec![]), check(vec![effect("new")])),
            "run",
            "eval",
        )
        .unwrap();
    assert_eq!(
        receipt.experiment_evidence_kind,
        EvidenceKind::PairedInterventional
    );
    assert_eq!(
        receipt.attribution_evidence_kind,
        EvidenceKind::Observational
    );
    assert_eq!(receipt.regression_count, 1);
    assert_eq!(receipt.improvement_count, 0);
    assert_eq!(receipt.stable_count, 0);
}

#[test]
fn same_message_class_at_a_different_location_is_not_stable() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let mut shifted = effect("same");
    shifted.file = Some("other.rs".into());
    shifted.line = Some(99);
    let receipt = engine(&store)
        .observe_pair(
            &patch(),
            &pair(check(vec![effect("same")]), check(vec![shifted])),
            "run",
            "eval",
        )
        .unwrap();
    assert_eq!(receipt.regression_count, 1);
    assert_eq!(receipt.improvement_count, 1);
    assert_eq!(receipt.stable_count, 0);
}

#[test]
fn fixed_baseline_failure_is_improvement_but_never_a_proximity_edge() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let receipt = engine(&store)
        .observe_pair(
            &patch(),
            &pair(check(vec![effect("fixed")]), check(vec![])),
            "run",
            "eval",
        )
        .unwrap();
    assert_eq!(receipt.improvement_count, 1);
    assert_eq!(receipt.triple_count, 0);
    assert_eq!(receipt.coverage.total_edges, 0);
}

#[test]
fn independent_pairs_are_not_deduplicated_and_observational_edges_remain_advisory() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let engine = engine(&store);
    let first = engine
        .observe_pair(
            &patch(),
            &pair(check(vec![]), check(vec![effect("new")])),
            "run-a",
            "eval",
        )
        .unwrap();
    let second = engine
        .observe_pair(
            &patch(),
            &pair(check(vec![]), check(vec![effect("new")])),
            "run-b",
            "eval",
        )
        .unwrap();
    assert_ne!(first.run_digest, second.run_digest);
    assert_eq!(first.coverage.total_edges, 1);
    assert_eq!(second.coverage.total_edges, 1);
    let graph = load_graph(&store, Some("v1")).unwrap();
    let edge = graph.graph.edge_weights().next().unwrap();
    assert_eq!(edge.stats.observations, 2);
    assert!(!first
        .prediction_disposition
        .eq(&PredictionDisposition::MaySkipChecks));
    assert!(!second
        .prediction_disposition
        .eq(&PredictionDisposition::MaySkipChecks));
}

#[test]
fn every_prediction_gate_rejection_reason_is_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let engine = engine(&store);
    let prediction = CausalPrediction {
        predicted_correctness: 0.0,
        predicted_novelty: 0.0,
        confidence: 0.0,
        coverage_fraction: 0.0,
        zero_shot_eligible: false,
        risk_flags: vec![RiskFlag {
            op_signature: build_edit_op_signature(&patch().edits[0].ops[0], 0, 1, 0, 1, "rs")
                .unwrap(),
            predicted_effect: effect("risk").sig,
            confidence: 1.0,
            historical_weight: 1.0,
        }],
    };
    let gate = engine.prediction_gate(&prediction, 0, true, false, false);
    assert_eq!(gate.disposition, PredictionDisposition::RunChecks);
    for reason in [
        PredictionGateReason::DisabledOptIn,
        PredictionGateReason::InsufficientIndependentRuns,
        PredictionGateReason::LowCoverage,
        PredictionGateReason::FuzzyOnlyEvidence,
        PredictionGateReason::ScopeOrConfigMismatch,
        PredictionGateReason::MissingInterventionalEvidence,
        PredictionGateReason::RiskFlags,
        PredictionGateReason::UnknownEffects,
    ] {
        assert!(gate.reasons.contains(&reason), "missing {reason:?}");
    }
}
