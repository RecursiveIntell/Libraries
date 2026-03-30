use std::collections::HashMap;
use std::path::PathBuf;

use check_runner::{CheckKind, CheckResult, EffectSignature, LocatedEffect, ParsedCheckOutput};
use typed_patch::{Anchor, EditOp, FileEdit, LineAttributionMap, LineRange, StructuredPatch};
use uuid::Uuid;

use crate::attribution::{build_line_map_index, AttributedRunResult, AttributionConfig};
use crate::{attribute_effects_with_config, build_edit_op_signature, global_pass_effect_signature};
use crate::{predict, AttributionTriple, CausalGraph};

fn sample_patch() -> StructuredPatch {
    StructuredPatch {
        patch_id: Uuid::nil(),
        summary: "sample patch".to_string(),
        edits: vec![
            FileEdit {
                path: PathBuf::from("src/lib.rs"),
                ops: vec![
                    EditOp::Insert {
                        anchor: Anchor::AfterLine {
                            line: 10,
                            context_before: vec![
                                "pub async fn compute() {".to_string(),
                                "    // comment fn fake()".to_string(),
                            ],
                            context_after: vec!["}".to_string()],
                        },
                        lines: vec!["    let x = 1;".to_string()],
                    },
                    EditOp::Replace {
                        range: LineRange {
                            start: 12,
                            end_exclusive: 14,
                        },
                        lines: vec!["    let y = 2;".to_string()],
                    },
                ],
                mode: None,
            },
            FileEdit {
                path: PathBuf::from("src/other.rs"),
                ops: vec![EditOp::Delete {
                    range: LineRange {
                        start: 4,
                        end_exclusive: 5,
                    },
                }],
                mode: None,
            },
        ],
        notes: Vec::new(),
    }
}

fn sample_line_map() -> LineAttributionMap {
    let mut mappings = std::collections::BTreeMap::new();
    mappings.insert("src/lib.rs".to_string(), vec![(10, 11), (12, 13), (13, 14)]);
    mappings.insert("src/other.rs".to_string(), vec![(4, 4)]);

    LineAttributionMap {
        mappings,
        resolved_anchors: std::collections::BTreeMap::new(),
    }
}

fn effect(
    check_kind: CheckKind,
    file: Option<&str>,
    line: Option<u32>,
    outcome: &str,
    severity: &str,
    message_class: &str,
) -> LocatedEffect {
    let check_kind_name = match check_kind {
        CheckKind::Fmt => "fmt",
        CheckKind::Clippy => "clippy",
        CheckKind::Test => "test",
    };

    LocatedEffect {
        file: file.map(PathBuf::from),
        line,
        col: None,
        message: message_class.to_string(),
        sig: EffectSignature {
            check_kind: check_kind_name.to_string(),
            outcome: outcome.to_string(),
            severity: severity.to_string(),
            message_class: message_class.to_string(),
            line_offset_from_edit: None,
        },
    }
}

fn parsed_output(check_kind: CheckKind, effects: Vec<LocatedEffect>) -> ParsedCheckOutput {
    ParsedCheckOutput {
        check_kind,
        exit_code: if effects.is_empty() { 0 } else { 1 },
        effects,
        raw_stdout: String::new(),
        raw_stderr: String::new(),
    }
}

fn check_result(
    fmt_pass: bool,
    clippy_pass: bool,
    test_pass: bool,
    fmt_effects: Vec<LocatedEffect>,
    clippy_effects: Vec<LocatedEffect>,
    test_effects: Vec<LocatedEffect>,
) -> CheckResult {
    CheckResult {
        fmt_pass,
        clippy_pass,
        test_pass,
        fmt_output: parsed_output(CheckKind::Fmt, fmt_effects),
        clippy_output: parsed_output(CheckKind::Clippy, clippy_effects),
        test_output: parsed_output(CheckKind::Test, test_effects),
        total_duration_ms: 1,
    }
}

#[test]
fn proportional_attribution_normalizes_per_effect() {
    let patch = sample_patch();
    let line_map = sample_line_map();
    let config = AttributionConfig {
        max_line_distance: 3,
        candidate_radius_multiplier: 2,
        top_k_causes: 2,
        softmax_temperature: 1.0,
        ..AttributionConfig::default()
    };
    let result = check_result(
        false,
        false,
        false,
        vec![effect(
            CheckKind::Fmt,
            Some("src/lib.rs"),
            Some(12),
            "fail",
            "error",
            "fmt_error",
        )],
        Vec::new(),
        Vec::new(),
    );

    let triples = attribute_effects_with_config(&patch, &result, &line_map, &config).unwrap();
    assert_eq!(triples.len(), 2);

    let total = triples.iter().map(|triple| triple.weight).sum::<f64>();
    assert!(
        (total - 1.0).abs() < 1e-9,
        "weights must sum to 1, got {total}"
    );
    assert!(triples.iter().all(|triple| triple.weight > 0.0));
}

#[test]
fn differential_pass_attribution_requires_clean_category_and_changed_files() {
    let patch = sample_patch();
    let line_map = sample_line_map();
    let config = AttributionConfig::default();
    let result = check_result(
        true,
        true,
        false,
        Vec::new(),
        vec![effect(
            CheckKind::Clippy,
            Some("src/unmodified.rs"),
            Some(4),
            "fail",
            "warning",
            "clippy_warning",
        )],
        vec![effect(
            CheckKind::Test,
            Some("src/lib.rs"),
            Some(13),
            "fail",
            "test_fail",
            "test_failure",
        )],
    );

    let triples = attribute_effects_with_config(&patch, &result, &line_map, &config).unwrap();
    let pass_triples = triples
        .iter()
        .filter(|triple| triple.effect.outcome == "pass")
        .collect::<Vec<_>>();
    let clippy_failures = triples
        .iter()
        .filter(|triple| triple.effect.check_kind == "clippy" && triple.effect.outcome != "pass")
        .count();

    assert_eq!(
        pass_triples.len(),
        patch.edits.iter().map(|edit| edit.ops.len()).sum::<usize>()
    );
    assert!(pass_triples
        .iter()
        .all(|triple| triple.effect == global_pass_effect_signature("fmt")));
    assert_eq!(
        clippy_failures, 0,
        "effects on unchanged files must not be attributed"
    );
    assert!(triples
        .iter()
        .any(|triple| triple.effect.check_kind == "test" && triple.effect.outcome != "pass"));
}

#[test]
fn confidence_is_higher_for_consistent_reinforcement_than_alternating() {
    let patch = sample_patch();
    let signature = build_edit_op_signature(
        &patch.edits[0].ops[0],
        0,
        patch.edits[0].ops.len(),
        0,
        patch.edits.len(),
        "rs",
    )
    .unwrap();
    let config = AttributionConfig::default();
    let failure_effect = effect(
        CheckKind::Fmt,
        Some("src/lib.rs"),
        Some(12),
        "fail",
        "error",
        "fmt_error",
    )
    .sig;
    let pass_effect = global_pass_effect_signature("fmt");
    let failure_run = AttributedRunResult::new(
        vec![AttributionTriple {
            cause: signature.clone(),
            effect: failure_effect.clone(),
            distance: 0,
            weight: 1.0,
        }],
        check_result(false, true, true, Vec::new(), Vec::new(), Vec::new()),
    );
    let pass_run = AttributedRunResult::new(
        vec![AttributionTriple {
            cause: signature.clone(),
            effect: pass_effect,
            distance: 0,
            weight: 1.0,
        }],
        check_result(true, true, true, Vec::new(), Vec::new(), Vec::new()),
    );

    let mut consistent = CausalGraph::new();
    for _ in 0..8 {
        consistent.ingest_run(&failure_run, &config).unwrap();
    }
    let consistent_cause = consistent.ensure_cause_node(&signature);
    let consistent_failure = consistent.ensure_effect_node(&failure_effect);
    let consistent_edge = consistent
        .graph
        .find_edge(consistent_cause, consistent_failure)
        .and_then(|edge| consistent.graph.edge_weight(edge))
        .unwrap()
        .clone();

    let mut alternating = CausalGraph::new();
    for _ in 0..4 {
        alternating.ingest_run(&failure_run, &config).unwrap();
        alternating.ingest_run(&pass_run, &config).unwrap();
    }
    let alternating_cause = alternating.ensure_cause_node(&signature);
    let alternating_failure = alternating.ensure_effect_node(&failure_effect);
    let alternating_failure = alternating
        .graph
        .find_edge(alternating_cause, alternating_failure)
        .and_then(|edge| alternating.graph.edge_weight(edge))
        .unwrap()
        .clone();

    assert!(consistent_edge.confidence > alternating_failure.confidence);
}

#[test]
fn run_hash_is_stable_across_triple_ordering() {
    let patch = sample_patch();
    let line_map = sample_line_map();
    let config = AttributionConfig {
        max_line_distance: 3,
        candidate_radius_multiplier: 2,
        top_k_causes: 2,
        softmax_temperature: 1.0,
        ..AttributionConfig::default()
    };
    let result = check_result(
        false,
        true,
        true,
        vec![effect(
            CheckKind::Fmt,
            Some("src/lib.rs"),
            Some(12),
            "fail",
            "error",
            "fmt_error",
        )],
        Vec::new(),
        Vec::new(),
    );

    let triples = attribute_effects_with_config(&patch, &result, &line_map, &config).unwrap();
    let forward = AttributedRunResult::new(triples.clone(), result.clone());
    let mut reversed = triples;
    reversed.reverse();
    let backward = AttributedRunResult::new(reversed, result);

    assert_eq!(forward.run_hash, backward.run_hash);
}

#[test]
fn save_load_roundtrip_preserves_predictions_and_decay() {
    let patch = sample_patch();
    let line_map = sample_line_map();
    let config = AttributionConfig {
        decay_factor: 0.9,
        ..AttributionConfig::default()
    };
    let result = check_result(
        false,
        true,
        true,
        vec![effect(
            CheckKind::Fmt,
            Some("src/lib.rs"),
            Some(12),
            "fail",
            "error",
            "fmt_error",
        )],
        Vec::new(),
        Vec::new(),
    );

    let triples = attribute_effects_with_config(&patch, &result, &line_map, &config).unwrap();
    let attributed = AttributedRunResult::new(triples, result);

    let mut graph = CausalGraph::new();
    graph.ingest_run(&attributed, &config).unwrap();
    let before_weight = graph.graph.edge_weights().next().unwrap().weight;
    graph.ingest_run(&attributed, &config).unwrap();
    let after_weight = graph.graph.edge_weights().next().unwrap().weight;
    assert!(
        after_weight < before_weight + 1.0,
        "decay should dampen accumulation"
    );

    let prediction_input = vec![build_edit_op_signature(
        &patch.edits[0].ops[0],
        0,
        patch.edits[0].ops.len(),
        0,
        patch.edits.len(),
        "rs",
    )
    .unwrap()];
    let prediction_before = predict(&prediction_input, &graph, 0.5, 0.2);

    let path = std::env::temp_dir().join(format!("cea-core-roundtrip-{}.bin", std::process::id()));
    graph.save(&path).unwrap();
    let loaded = CausalGraph::load(&path).unwrap();
    let prediction_after = predict(&prediction_input, &loaded, 0.5, 0.2);

    assert_eq!(graph.coverage_summary(), loaded.coverage_summary());
    assert_eq!(prediction_before, prediction_after);
    let _ = std::fs::remove_file(path);
}

#[test]
fn line_map_lookup_is_indexed_by_hash_map() {
    let line_map = sample_line_map();
    let indexed = build_line_map_index(&line_map);
    let file_map = indexed.mappings.get("src/lib.rs").unwrap();
    let _: &HashMap<u32, u32> = file_map;
    assert_eq!(file_map.get(&10), Some(&11));
}

#[test]
fn legacy_signature_fields_deserialize_cleanly() {
    let payload = r#"{
        "op_kind":"Insert",
        "anchor_kind":"AfterLine",
        "lines_added":1,
        "lines_removed":0,
        "context_hash":"abc",
        "file_extension":"rs",
        "scope_tag":"fn",
        "op_index":"first",
        "file_index":"only"
    }"#;

    let signature: crate::EditOpSignature = serde_json::from_str(payload).unwrap();
    assert_eq!(signature.op_index.0, 0);
    assert_eq!(signature.file_index.0, 0);
}

#[test]
fn low_sample_perfect_history_remains_conservative() {
    let confidence = crate::calibration::advisory_confidence(
        crate::calibration::conservative_reliability(2.0, 1.0),
        1.0,
        3.0,
        1,
        crate::calibration::MIN_SAMPLES_PER_SIGNATURE,
    );

    assert!(
        confidence < 0.2,
        "low sample data should stay conservative, got {confidence}"
    );
}

#[test]
fn high_fuzzy_coverage_is_capable_of_capping_confidence() {
    let confidence = crate::calibration::advisory_confidence(
        crate::calibration::conservative_reliability(80.0, 1.0),
        0.3,
        81.0,
        1,
        crate::calibration::MIN_SAMPLES_PER_SIGNATURE,
    );

    assert!(
        confidence < 0.5,
        "fuzzy coverage should be capped, got {confidence}"
    );
}

#[test]
fn strong_exact_signal_with_high_sample_can_still_be_confident() {
    let confidence = crate::calibration::advisory_confidence(
        crate::calibration::conservative_reliability(30.0, 1.0),
        1.0,
        31.0,
        1,
        crate::calibration::MIN_SAMPLES_PER_SIGNATURE,
    );

    assert!(
        confidence > 0.6,
        "high sample exact signal should become high-confidence, got {confidence}"
    );
}

#[test]
fn contradictions_lower_conservative_confidence() {
    let clean = crate::calibration::advisory_confidence(
        crate::calibration::conservative_reliability(30.0, 1.0),
        1.0,
        31.0,
        1,
        crate::calibration::MIN_SAMPLES_PER_SIGNATURE,
    );
    let noisy = crate::calibration::advisory_confidence(
        crate::calibration::conservative_reliability(30.0, 20.0),
        1.0,
        50.0,
        1,
        crate::calibration::MIN_SAMPLES_PER_SIGNATURE,
    );

    assert!(
        noisy < clean,
        "contradictions should reduce confidence: clean={clean} noisy={noisy}"
    );
}
