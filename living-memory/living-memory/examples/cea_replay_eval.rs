//! Deterministic, offline mechanics evaluation for public CEA APIs.
//!
//! The fixture lane deliberately exercises local graph prediction and one tiny
//! Cargo fixture through the public engine. It is not an external benchmark.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cea_core::{
    predict_with_config, AnchorKind, CausalGraph, EditOpKind, EditOpSignature, FileIndex, OpIndex,
    PredictionConfig, ScopeTag,
};
use forge_engine::{
    Anchor, CargoAdapter, CausalAttributionEngine, EditOp, FileEdit, ForgeConfig, ForgeStore,
    HostBackend, LineRange, PredictionDisposition, StructuredPatch,
};
use serde::Serialize;

const SCHEMA_VERSION: &str = "cea_replay_eval_receipt_v1";
const FIXTURE_LANE: &str = "deterministic_local_mechanics_only";

#[derive(Debug)]
struct ReplayCase {
    label: &'static str,
    kind: &'static str,
    signature: EditOpSignature,
    graph: CausalGraph,
    correct: bool,
    risk: bool,
    fuzzy_enabled: bool,
}

#[derive(Debug, Serialize)]
struct FixtureReceipt {
    id: String,
    label: String,
    lane: String,
}

#[derive(Debug, Serialize)]
struct CoverageReceipt {
    exact_cases: u32,
    fuzzy_cases: u32,
    unknown_cases: u32,
    exact_mean: f64,
    fuzzy_mean: f64,
    unknown_mean: f64,
}

#[derive(Debug, Serialize)]
struct RiskMetrics {
    precision: f64,
    recall: f64,
    true_positive: u32,
    false_positive: u32,
    false_negative: u32,
}

#[derive(Debug, Serialize)]
struct CalibrationBucket {
    lower_inclusive: f64,
    upper_inclusive: f64,
    count: u32,
    mean_prediction: f64,
    empirical_rate: f64,
}

#[derive(Debug, Serialize)]
struct AblationMetrics {
    localization_accuracy: f64,
    intervention_count: u32,
    supported_operation_indexes: Vec<usize>,
}

#[derive(Debug, Serialize)]
struct BaselineComparison {
    available: bool,
    brier_score: Option<f64>,
    risk_precision: Option<f64>,
    risk_recall: Option<f64>,
    reason: String,
}

#[derive(Debug, Serialize)]
struct Receipt {
    schema_version: &'static str,
    generated_at: String,
    elapsed_ms: u64,
    fixture_lane: &'static str,
    fixtures: Vec<FixtureReceipt>,
    coverage: CoverageReceipt,
    brier_score: f64,
    risk: RiskMetrics,
    calibration_buckets: Vec<CalibrationBucket>,
    ablation: AblationMetrics,
    false_negative_count: u32,
    runtime_ms: u64,
    baseline_comparisons: BTreeMap<String, BaselineComparison>,
    configuration: serde_json::Value,
    claim_boundary: serde_json::Value,
}

fn signature(context: &str, index: u32) -> EditOpSignature {
    EditOpSignature {
        op_kind: EditOpKind::Replace,
        anchor_kind: AnchorKind::Range,
        lines_added: 1,
        lines_removed: 1,
        context_hash: blake3::hash(context.as_bytes()).to_hex().to_string(),
        file_extension: "rs".into(),
        scope_tag: ScopeTag::Function,
        op_index: OpIndex(index),
        file_index: FileIndex(0),
    }
}

fn graph_for(cause: &EditOpSignature, outcome: &str, observations: usize) -> CausalGraph {
    let mut graph = CausalGraph::new();
    let cause_index = graph.ensure_cause_node(cause);
    let effect = forge_engine::EffectSignature {
        check_kind: "test".into(),
        outcome: outcome.into(),
        severity: if outcome == "pass" { "info" } else { "error" }.into(),
        message_class: format!("fixture-{outcome}"),
        line_offset_from_edit: Some(0),
    };
    let effect_index = graph.ensure_effect_node(&effect);
    for _ in 0..observations {
        graph.update_edge(cause_index, effect_index, 1.0);
    }
    graph
}

fn contradictory_graph(cause: &EditOpSignature) -> CausalGraph {
    let mut graph = graph_for(cause, "pass", 6);
    let cause_index = graph.ensure_cause_node(cause);
    let effect = forge_engine::EffectSignature {
        check_kind: "test".into(),
        outcome: "fail".into(),
        severity: "error".into(),
        message_class: "fixture-contradiction".into(),
        line_offset_from_edit: Some(0),
    };
    let effect_index = graph.ensure_effect_node(&effect);
    for _ in 0..6 {
        graph.update_edge(cause_index, effect_index, 1.0);
    }
    graph
}

fn replay_cases() -> Vec<ReplayCase> {
    let exact_safe = signature("exact-safe", 0);
    let exact_risk = signature("exact-risk", 0);
    let fuzzy_known = signature("fuzzy-known", 0);
    let fuzzy_candidate = signature("fuzzy-candidate", 0);
    let unknown = signature("unknown", 0);
    let low_sample = signature("low-sample", 0);
    let high_sample = signature("high-sample", 0);
    let contradictory = signature("contradictory", 0);
    vec![
        ReplayCase {
            label: "exact-known-safe",
            kind: "exact",
            signature: exact_safe.clone(),
            graph: graph_for(&exact_safe, "pass", 8),
            correct: true,
            risk: false,
            fuzzy_enabled: false,
        },
        ReplayCase {
            label: "exact-known-risk",
            kind: "exact",
            signature: exact_risk.clone(),
            graph: graph_for(&exact_risk, "fail", 8),
            correct: false,
            risk: true,
            fuzzy_enabled: false,
        },
        ReplayCase {
            label: "structural-fuzzy-enabled",
            kind: "fuzzy",
            signature: fuzzy_candidate,
            graph: graph_for(&fuzzy_known, "pass", 8),
            correct: true,
            risk: false,
            fuzzy_enabled: true,
        },
        ReplayCase {
            label: "unknown-signature",
            kind: "unknown",
            signature: unknown,
            graph: CausalGraph::new(),
            correct: false,
            risk: true,
            fuzzy_enabled: false,
        },
        ReplayCase {
            label: "low-sample-history",
            kind: "exact",
            signature: low_sample.clone(),
            graph: graph_for(&low_sample, "pass", 1),
            correct: true,
            risk: false,
            fuzzy_enabled: false,
        },
        ReplayCase {
            label: "high-sample-history",
            kind: "exact",
            signature: high_sample.clone(),
            graph: graph_for(&high_sample, "pass", 12),
            correct: true,
            risk: false,
            fuzzy_enabled: false,
        },
        ReplayCase {
            label: "contradictory-evidence",
            kind: "exact",
            signature: contradictory.clone(),
            graph: contradictory_graph(&contradictory),
            correct: false,
            risk: true,
            fuzzy_enabled: false,
        },
    ]
}

fn run_replay_cases(cases: Vec<ReplayCase>) -> Vec<(ReplayCase, forge_engine::CausalPrediction)> {
    cases
        .into_iter()
        .map(|case| {
            let config = PredictionConfig {
                enable_fuzzy_matching: case.fuzzy_enabled,
                ..PredictionConfig::default()
            };
            let prediction =
                predict_with_config(std::slice::from_ref(&case.signature), &case.graph, &config);
            (case, prediction)
        })
        .collect()
}

fn two_operation_patch() -> StructuredPatch {
    StructuredPatch {
        patch_id: uuid::Uuid::nil(),
        summary: "deterministic two-operation ablation fixture".into(),
        notes: vec![],
        edits: vec![FileEdit {
            path: "src/lib.rs".into(),
            mode: None,
            ops: vec![
                EditOp::Replace {
                    range: LineRange {
                        start: 2,
                        end_exclusive: 3,
                    },
                    lines: vec!["    panic!(\"cea injected failure\");".into()],
                },
                EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 3,
                        context_before: vec!["}".into()],
                        context_after: vec![],
                    },
                    lines: vec!["// harmless ablation companion".into()],
                },
            ],
        }],
    }
}

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cea/two_op_ablation")
}

fn run_ablation() -> Result<AblationMetrics, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let store =
        ForgeStore::open(&temp.path().join("cea-eval.db")).map_err(|error| error.to_string())?;
    let config = ForgeConfig::default();
    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let engine = CausalAttributionEngine::new(&store, &backend, &adapter, &config, "cea-eval-v1");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let receipts = runtime
        .block_on(engine.run_singleton_ablations(&fixture_path(), &two_operation_patch()))
        .map_err(|error| error.to_string())?;
    let supported = receipts
        .iter()
        .filter(|receipt| {
            matches!(
                receipt.classification,
                forge_engine::AblationClassification::Supported
            )
        })
        .map(|receipt| receipt.operation_index)
        .collect::<Vec<_>>();
    let correctly_localized = supported == vec![0];
    Ok(AblationMetrics {
        localization_accuracy: if correctly_localized { 1.0 } else { 0.0 },
        intervention_count: receipts.len() as u32,
        supported_operation_indexes: supported,
    })
}

fn evaluation_gate_is_fail_closed() -> Result<bool, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let store =
        ForgeStore::open(&temp.path().join("gate.db")).map_err(|error| error.to_string())?;
    let config = ForgeConfig::default();
    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let engine = CausalAttributionEngine::new(&store, &backend, &adapter, &config, "cea-eval-v1");
    Ok(engine
        .predict_patch(&[])
        .map_err(|error| error.to_string())?
        .gate
        .disposition
        == PredictionDisposition::RunChecks)
}

fn brier_score(points: &[(f64, bool)]) -> f64 {
    points
        .iter()
        .map(|(p, label)| (p - f64::from(*label as u8)).powi(2))
        .sum::<f64>()
        / points.len().max(1) as f64
}

fn calibration_bucket(prediction: f64) -> usize {
    (prediction.clamp(0.0, 1.0) * 10.0).floor().min(9.0) as usize
}

fn calibration(points: &[(f64, bool)]) -> Vec<CalibrationBucket> {
    (0..10)
        .map(|index| {
            let entries = points
                .iter()
                .filter(|(prediction, _)| calibration_bucket(*prediction) == index)
                .collect::<Vec<_>>();
            let count = entries.len() as u32;
            CalibrationBucket {
                lower_inclusive: index as f64 / 10.0,
                upper_inclusive: (index + 1) as f64 / 10.0,
                count,
                mean_prediction: if count == 0 {
                    0.0
                } else {
                    entries.iter().map(|(p, _)| p).sum::<f64>() / count as f64
                },
                empirical_rate: if count == 0 {
                    0.0
                } else {
                    entries
                        .iter()
                        .map(|(_, label)| u8::from(*label) as f64)
                        .sum::<f64>()
                        / count as f64
                },
            }
        })
        .collect()
}

fn risk_metrics(predicted: &[bool], actual: &[bool]) -> RiskMetrics {
    let (tp, fp, fn_) = predicted.iter().zip(actual).fold(
        (0_u32, 0_u32, 0_u32),
        |(tp, fp, fn_), (prediction, label)| match (*prediction, *label) {
            (true, true) => (tp + 1, fp, fn_),
            (true, false) => (tp, fp + 1, fn_),
            (false, true) => (tp, fp, fn_ + 1),
            (false, false) => (tp, fp, fn_),
        },
    );
    RiskMetrics {
        precision: ratio(tp, tp + fp),
        recall: ratio(tp, tp + fn_),
        true_positive: tp,
        false_positive: fp,
        false_negative: fn_,
    }
}

fn ratio(numerator: u32, denominator: u32) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
fn fixture_id(label: &str) -> String {
    blake3::hash(format!("cea-eval-v1:{label}").as_bytes()).to_hex()[..16].to_string()
}

fn finite_unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn receipt_schema_is_valid(json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    [
        "schema_version",
        "generated_at",
        "elapsed_ms",
        "fixtures",
        "coverage",
        "brier_score",
        "risk",
        "calibration_buckets",
        "ablation",
        "false_negative_count",
        "runtime_ms",
        "baseline_comparisons",
        "configuration",
        "claim_boundary",
    ]
    .iter()
    .all(|key| object.contains_key(*key))
}

fn validate_receipt(receipt: &Receipt) -> Result<(), String> {
    if !finite_unit(receipt.brier_score)
        || !finite_unit(receipt.risk.precision)
        || !finite_unit(receipt.risk.recall)
        || !finite_unit(receipt.ablation.localization_accuracy)
    {
        return Err("top-level metric was not finite and within [0, 1]".into());
    }
    for bucket in &receipt.calibration_buckets {
        if !finite_unit(bucket.mean_prediction) || !finite_unit(bucket.empirical_rate) {
            return Err("calibration metric was not finite and within [0, 1]".into());
        }
    }
    for baseline in receipt.baseline_comparisons.values() {
        for metric in [
            baseline.brier_score,
            baseline.risk_precision,
            baseline.risk_recall,
        ]
        .into_iter()
        .flatten()
        {
            if !finite_unit(metric) {
                return Err("baseline metric was not finite and within [0, 1]".into());
            }
        }
    }
    let json = serde_json::to_string(receipt).map_err(|error| error.to_string())?;
    if !receipt_schema_is_valid(&json) {
        return Err("receipt did not satisfy its required schema".into());
    }
    Ok(())
}

fn unavailable_baseline_comparisons() -> BTreeMap<String, BaselineComparison> {
    BTreeMap::from([
        (
            "full_check_oracle".into(),
            BaselineComparison {
                available: false,
                brier_score: None,
                risk_precision: None,
                risk_recall: None,
                reason: "prediction cases use synthetic labeled graphs and do not execute a checker oracle"
                    .into(),
            },
        ),
        (
            "naive_proximity".into(),
            BaselineComparison {
                available: false,
                brier_score: None,
                risk_precision: None,
                risk_recall: None,
                reason: "prediction cases do not contain executable source locations for an independent proximity baseline"
                    .into(),
            },
        ),
    ])
}

fn output_path() -> Result<PathBuf, String> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--output"), Some(path), None) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Err("usage: cea_replay_eval --output <requested-receipt-path>; unknown arguments are rejected".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_path().map_err(std::io::Error::other)?;
    let started = Instant::now();
    let cases = replay_cases();
    let results = run_replay_cases(cases);
    let points = results
        .iter()
        .map(|(case, prediction)| (prediction.predicted_correctness, case.correct))
        .collect::<Vec<_>>();
    let predicted_risk = results
        .iter()
        .map(|(_, prediction)| !prediction.risk_flags.is_empty())
        .collect::<Vec<_>>();
    let actual_risk = results
        .iter()
        .map(|(case, _)| case.risk)
        .collect::<Vec<_>>();
    let risk = risk_metrics(&predicted_risk, &actual_risk);
    let false_negative_count = risk.false_negative;
    let coverage_for = |kind: &str| -> Vec<f64> {
        results
            .iter()
            .filter(|(case, _)| case.kind == kind)
            .map(|(_, prediction)| prediction.coverage_fraction)
            .collect()
    };
    let mean = |items: Vec<f64>| {
        if items.is_empty() {
            0.0
        } else {
            items.iter().sum::<f64>() / items.len() as f64
        }
    };
    let exact = coverage_for("exact");
    let fuzzy = coverage_for("fuzzy");
    let unknown = coverage_for("unknown");
    let ablation = run_ablation().map_err(std::io::Error::other)?;
    let prediction_gate_is_fail_closed =
        evaluation_gate_is_fail_closed().map_err(std::io::Error::other)?;
    if !prediction_gate_is_fail_closed {
        return Err("prediction gate did not fail closed".into());
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let baselines = unavailable_baseline_comparisons();
    let receipt = Receipt {
        schema_version: SCHEMA_VERSION,
        generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        elapsed_ms,
        fixture_lane: FIXTURE_LANE,
        fixtures: results
            .iter()
            .map(|(case, _)| FixtureReceipt {
                id: fixture_id(case.label),
                label: case.label.into(),
                lane: FIXTURE_LANE.into(),
            })
            .chain(std::iter::once(FixtureReceipt {
                id: fixture_id("two-operation-ablation"),
                label: "two-operation-ablation".into(),
                lane: "real_cargo_fixture".into(),
            }))
            .collect(),
        coverage: CoverageReceipt {
            exact_cases: exact.len() as u32,
            fuzzy_cases: fuzzy.len() as u32,
            unknown_cases: unknown.len() as u32,
            exact_mean: mean(exact),
            fuzzy_mean: mean(fuzzy),
            unknown_mean: mean(unknown),
        },
        brier_score: brier_score(&points),
        risk,
        calibration_buckets: calibration(&points),
        false_negative_count,
        runtime_ms: elapsed_ms,
        ablation,
        baseline_comparisons: baselines,
        configuration: serde_json::json!({ "offline": true, "fuzzy_matching": "enabled only in structural-fuzzy-enabled evaluation case", "prediction_gate_outcome": if prediction_gate_is_fail_closed { "run_checks" } else { "unexpected" }, "fixture_lane": FIXTURE_LANE }),
        claim_boundary: serde_json::json!({ "validated": "deterministic local engine mechanics and bounded ablation localization", "not_validated": ["external superiority", "production readiness", "safe zero-shot validation"] }),
    };
    validate_receipt(&receipt).map_err(std::io::Error::other)?;
    let encoded = serde_json::to_vec_pretty(&receipt)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, encoded)?;
    println!("wrote {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metric_formulas_are_defined() {
        assert_eq!(brier_score(&[(1.0, true), (0.0, false)]), 0.0);
        assert_eq!(brier_score(&[(0.5, true)]), 0.25);
    }
    #[test]
    fn bucket_boundaries_are_stable() {
        assert_eq!(calibration_bucket(0.0), 0);
        assert_eq!(calibration_bucket(0.1), 1);
        assert_eq!(calibration_bucket(1.0), 9);
    }
    #[test]
    fn fixture_ids_are_deterministic() {
        assert_eq!(fixture_id("exact-known"), fixture_id("exact-known"));
        assert_ne!(fixture_id("exact-known"), fixture_id("unknown"));
    }
    #[test]
    fn receipt_has_required_schema() {
        let receipt = serde_json::json!({ "schema_version": SCHEMA_VERSION, "generated_at": "2026-07-13T00:00:00Z", "elapsed_ms": 1, "fixtures": [], "coverage": {}, "brier_score": 0.0, "risk": {}, "calibration_buckets": [], "ablation": {}, "false_negative_count": 0, "runtime_ms": 1, "baseline_comparisons": {}, "configuration": {}, "claim_boundary": {} });
        assert!(receipt_schema_is_valid(&receipt.to_string()));
    }

    #[test]
    fn unavailable_baselines_never_derive_predictions_from_fixture_labels() {
        let baselines = unavailable_baseline_comparisons();
        assert_eq!(baselines.len(), 2);
        assert!(baselines.values().all(|baseline| !baseline.available));
        assert!(baselines.values().all(|baseline| {
            baseline.brier_score.is_none()
                && baseline.risk_precision.is_none()
                && baseline.risk_recall.is_none()
        }));
    }
}
