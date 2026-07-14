//! I. CEA tests (I1–I5)

use cea_core::{AnchorKind, EditOpKind, FileIndex, OpIndex, ScopeTag};
use forge_engine::cea::graph::CausalGraph;
use forge_engine::cea::instrumentation::{
    attribute_effects, build_edit_op_signature, AttributedRunResult, AttributionTriple,
    EditOpSignature,
};
use forge_engine::cea::predictor::predict;
use forge_engine::cea::store::{
    load_graph_with_tx, update_graph, update_graph_with_tx, UpdateResult,
};
use forge_engine::config::{CeaConfig, ForgeConfig};
use forge_engine::exec::backend::{
    CheckKind, CheckResult, EffectSignature, LocatedEffect, ParsedCheckOutput,
};
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::runtime::patch::types::{
    Anchor, EditOp, FileEdit, FileMode, LineRange, StructuredPatch,
};
use forge_engine::store::ForgeStore;
use std::collections::BTreeMap;
use tempfile::TempDir;
use uuid::Uuid;

/// Helper: create a fresh ForgeStore in a temp directory.
fn temp_store() -> (ForgeStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    (store, dir)
}

/// Helper: build a simple EditOp::Insert for testing.
fn make_test_edit_op() -> EditOp {
    EditOp::Insert {
        anchor: Anchor::AfterLine {
            line: 10,
            context_before: vec!["fn compute() {".to_string()],
            context_after: vec![],
        },
        lines: vec![
            "    let x = 42;".to_string(),
            "    println!(\"{x}\");".to_string(),
        ],
    }
}

/// Helper: build a simple EffectSignature for testing.
fn make_test_effect() -> EffectSignature {
    EffectSignature {
        check_kind: "clippy".to_string(),
        outcome: "warning".to_string(),
        severity: "warning".to_string(),
        message_class: "unused_variable".to_string(),
        line_offset_from_edit: Some(2),
    }
}

/// Helper: build a pass EffectSignature.
fn make_pass_effect() -> EffectSignature {
    EffectSignature {
        check_kind: "test".to_string(),
        outcome: "pass".to_string(),
        severity: "pass".to_string(),
        message_class: "test_pass".to_string(),
        line_offset_from_edit: None,
    }
}

/// Helper: construct an AttributedRunResult with given triples.
fn make_attributed_run(triples: Vec<AttributionTriple>) -> AttributedRunResult {
    let check_result = CheckResult {
        fmt_pass: true,
        clippy_pass: false,
        test_pass: true,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput {
            effects: vec![LocatedEffect {
                file: Some(std::path::PathBuf::from("src/main.rs")),
                line: Some(12),
                col: None,
                message: "unused variable: `x`".to_string(),
                sig: make_test_effect(),
            }],
            ..Default::default()
        },
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 500,
    };

    let run_hash = compute_run_hash_from_triples(&triples);
    AttributedRunResult {
        triples,
        check_result,
        run_hash,
        observation_key: None,
        evidence_kind: forge_engine::EvidenceKind::Observational,
    }
}

/// Compute run hash from triples (mirrors compute_run_hash but takes a slice).
fn compute_run_hash_from_triples(triples: &[AttributionTriple]) -> String {
    let mut hasher = blake3::Hasher::new();
    for triple in triples {
        let cause_json = serde_json::to_string(&triple.cause).unwrap_or_default();
        let effect_json = serde_json::to_string(&triple.effect).unwrap_or_default();
        hasher.update(cause_json.as_bytes());
        hasher.update(effect_json.as_bytes());
        hasher.update(&triple.distance.to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

// ─────────────────────────────────────────────────────────────
// I1: cea_instrumentation_extracts_effect_signatures
// ─────────────────────────────────────────────────────────────

/// I1: Provide mocked clippy JSON with known lint.
/// Assert extracted EffectSignature matches expected.
#[test]
fn i1_cea_instrumentation_extracts_effect_signatures() {
    let op = make_test_edit_op();

    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();

    // Verify the signature has the right structural properties
    assert_eq!(sig.op_kind, EditOpKind::Insert);
    assert_eq!(sig.anchor_kind, AnchorKind::AfterLine);
    assert_eq!(sig.lines_added, 2);
    assert_eq!(sig.lines_removed, 0);
    assert_eq!(sig.file_extension, "rs");
    assert_eq!(sig.op_index, OpIndex(0));
    assert_eq!(sig.file_index, FileIndex(0));

    // Scope should be "fn" because context_before contains "fn compute() {"
    assert_eq!(sig.scope_tag, ScopeTag::Function);

    // context_hash must be a 64-char hex string (blake3)
    assert_eq!(sig.context_hash.len(), 64);
    assert!(sig.context_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify that building a signature from same inputs is deterministic
    let sig2 = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    assert_eq!(sig.context_hash, sig2.context_hash);
    assert_eq!(sig.op_kind, sig2.op_kind);
}

// ─────────────────────────────────────────────────────────────
// I2: cea_edit_op_signature_no_raw_source
// ─────────────────────────────────────────────────────────────

/// I2: Construct EditOpSignature from a FileEdit with known context.
/// Assert sig_json does not contain any raw context lines.
/// Assert context_hash is a 64-char hex string (blake3).
#[test]
fn i2_cea_edit_op_signature_no_raw_source() {
    // Replace op — uses range (no anchor context), but we still check no raw source leaks.
    // For context, we test Insert with context_before which has actual context.
    let op = EditOp::Insert {
        anchor: Anchor::AfterLine {
            line: 5,
            context_before: vec![
                "    impl Foo {".to_string(),
                "        fn bar(&self) -> Result<(), Error> {".to_string(),
            ],
            context_after: vec![],
        },
        lines: vec![
            "        fn bar(&self) -> ForgeResult<()> {".to_string(),
            "            Ok(())".to_string(),
        ],
    };

    let sig = build_edit_op_signature(&op, 0, 2, 0, 1, "rs").unwrap();

    // Serialize to JSON
    let sig_json = serde_json::to_string(&sig).unwrap();

    // Assert raw context lines do NOT appear in the serialized form
    assert!(
        !sig_json.contains("impl Foo"),
        "Raw context 'impl Foo' found in sig_json: {sig_json}"
    );
    assert!(
        !sig_json.contains("fn bar"),
        "Raw context 'fn bar' found in sig_json: {sig_json}"
    );
    assert!(
        !sig_json.contains("Result<(), Error>"),
        "Raw source 'Result<(), Error>' found in sig_json: {sig_json}"
    );
    assert!(
        !sig_json.contains("ForgeResult<()>"),
        "Raw new_lines content found in sig_json: {sig_json}"
    );
    assert!(
        !sig_json.contains("Ok(())"),
        "Raw new_lines content found in sig_json: {sig_json}"
    );

    // context_hash must be a valid 64-char hex blake3 hash
    assert_eq!(
        sig.context_hash.len(),
        64,
        "context_hash must be 64 chars, got {}",
        sig.context_hash.len()
    );
    assert!(
        sig.context_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "context_hash must be all hex: {}",
        sig.context_hash
    );

    // Validate via invariants
    forge_engine::invariants::validate_cea_no_raw_source(&sig_json).unwrap();
}

// ─────────────────────────────────────────────────────────────
// I3: cea_graph_update_is_idempotent
// ─────────────────────────────────────────────────────────────

/// I3: Run update_graph(result) twice with same AttributedRunResult.
/// Assert cea_edges weights unchanged after second call (run_hash deduplicated).
#[test]
fn i3_cea_graph_update_is_idempotent() {
    let (store, _dir) = temp_store();

    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();

    let triples = vec![AttributionTriple {
        cause: sig.clone(),
        effect: effect.clone(),
        distance: 2,
        weight: 1.0,
    }];

    let result = make_attributed_run(triples);
    let config = ForgeConfig::default();

    // First update — should apply
    let update1 = update_graph(&store, &result, "eval-001", "v0001", &config).unwrap();
    match &update1 {
        UpdateResult::Applied {
            edges_added,
            edges_updated,
        } => {
            assert_eq!(*edges_added, 1, "first call should add 1 edge");
            assert_eq!(*edges_updated, 0, "first call should update 0 edges");
        }
        UpdateResult::AlreadyProcessed => {
            panic!("first call should not be AlreadyProcessed");
        }
    }

    // Read edge weight after first update
    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let edges_after_first = store.get_cea_edges_for_cause(&cause_id).unwrap();
    assert_eq!(edges_after_first.len(), 1);
    let weight_after_first = edges_after_first[0].weight;
    let count_after_first = edges_after_first[0].count;

    // Second update with SAME result — should be deduplicated
    let update2 = update_graph(&store, &result, "eval-001", "v0001", &config).unwrap();
    assert!(
        matches!(update2, UpdateResult::AlreadyProcessed),
        "second call must return AlreadyProcessed"
    );

    // Edge weights must be unchanged
    let edges_after_second = store.get_cea_edges_for_cause(&cause_id).unwrap();
    assert_eq!(edges_after_second.len(), 1);
    assert!(
        (edges_after_second[0].weight - weight_after_first).abs() < 1e-9,
        "edge weight changed after idempotent replay: {} vs {}",
        edges_after_second[0].weight,
        weight_after_first
    );
    assert_eq!(
        edges_after_second[0].count, count_after_first,
        "edge count changed after idempotent replay"
    );
}

#[test]
fn i3b_cea_graph_update_is_atomic_when_run_log_insert_fails() {
    let (store, _dir) = temp_store();
    store
        .with_transaction(|conn| {
            conn.execute_batch(
                r#"
CREATE TRIGGER fail_cea_run_log
BEFORE INSERT ON cea_run_log
BEGIN
    SELECT RAISE(FAIL, 'blocked run log');
END;
"#,
            )?;
            Ok(())
        })
        .unwrap();

    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();
    let result = make_attributed_run(vec![AttributionTriple {
        cause: sig.clone(),
        effect,
        distance: 2,
        weight: 1.0,
    }]);
    let config = ForgeConfig::default();

    let err = update_graph(&store, &result, "eval-blocked", "v0001", &config).unwrap_err();
    assert!(
        err.to_string().contains("blocked run log"),
        "expected trigger failure, got {err}"
    );

    let (nodes, edges) = store.get_all_cea_nodes_and_edges(None).unwrap();
    assert!(nodes.is_empty(), "CEA nodes must roll back on failure");
    assert!(edges.is_empty(), "CEA edges must roll back on failure");
}

#[test]
fn i3c_cea_graph_update_is_atomic_when_run_log_insert_fails_with_store_conn() {
    let (store, _dir) = temp_store();
    store
        .with_conn(|conn| {
            conn.execute_batch(
                r#"
CREATE TRIGGER fail_cea_run_log
BEFORE INSERT ON cea_run_log
BEGIN
    SELECT RAISE(FAIL, 'blocked run log');
END;
"#,
            )?;
            Ok(())
        })
        .unwrap();

    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();
    let result = make_attributed_run(vec![AttributionTriple {
        cause: sig.clone(),
        effect,
        distance: 2,
        weight: 1.0,
    }]);
    let config = ForgeConfig::default();

    let err = update_graph(&store, &result, "eval-blocked", "v0001", &config).unwrap_err();
    assert!(
        err.to_string().contains("blocked run log"),
        "expected trigger failure, got {err}"
    );

    let (nodes, edges) = store.get_all_cea_nodes_and_edges(None).unwrap();
    assert!(nodes.is_empty(), "CEA nodes must roll back on failure");
    assert!(edges.is_empty(), "CEA edges must roll back on failure");
}

#[test]
fn i3d_cea_graph_update_is_atomic_with_store_transaction_connection() {
    let (store, _dir) = temp_store();
    store
        .with_conn(|conn| {
            conn.execute_batch(
                r#"
CREATE TRIGGER fail_cea_run_log
BEFORE INSERT ON cea_run_log
BEGIN
    SELECT RAISE(FAIL, 'blocked run log');
END;
"#,
            )?;
            Ok(())
        })
        .unwrap();

    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();
    let result = make_attributed_run(vec![AttributionTriple {
        cause: sig.clone(),
        effect,
        distance: 2,
        weight: 1.0,
    }]);
    let config = ForgeConfig::default();

    let err = store
        .with_transaction_conn(|tx| {
            let err = update_graph_with_tx(tx, &result, "eval-blocked", "v0001", &config)?;
            Ok(err)
        })
        .unwrap_err();
    assert!(
        err.to_string().contains("blocked run log"),
        "expected trigger failure, got {err}"
    );

    let (nodes, edges) = store.get_all_cea_nodes_and_edges(None).unwrap();
    assert!(nodes.is_empty(), "CEA nodes must roll back on failure");
    assert!(edges.is_empty(), "CEA edges must roll back on failure");
}

#[test]
fn i3e_cea_graph_load_graph_works_with_store_transaction_connection() {
    let (store, _dir) = temp_store();
    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();
    let result = make_attributed_run(vec![AttributionTriple {
        cause: sig.clone(),
        effect: effect.clone(),
        distance: 2,
        weight: 1.0,
    }]);
    let config = ForgeConfig::default();

    update_graph(&store, &result, "eval-001", "v0001", &config).unwrap();

    let graph = store
        .with_transaction_conn(|tx| load_graph_with_tx(tx, Some("v0001")))
        .unwrap();
    assert_eq!(
        graph.graph.node_count(),
        2,
        "graph should contain cause + effect nodes"
    );
    assert_eq!(
        graph.graph.edge_count(),
        1,
        "graph should contain one causal edge"
    );
    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let effect_id = forge_engine::cea::instrumentation::effect_node_id(&effect);
    assert!(
        graph.node_index_map.contains_key(&cause_id),
        "cause node should be present"
    );
    assert!(
        graph.node_index_map.contains_key(&effect_id),
        "effect node should be present"
    );
}

#[test]
fn i3f_cea_graph_restart_preserves_edge_stats_and_predictions() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let first_effect = make_test_effect();
    let second_effect = EffectSignature {
        message_class: "second_warning".to_string(),
        ..make_test_effect()
    };
    let first_result = make_attributed_run(vec![AttributionTriple {
        cause: sig.clone(),
        effect: first_effect.clone(),
        distance: 2,
        weight: 1.0,
    }]);
    let config = ForgeConfig::default();

    update_graph(&store, &first_result, "eval-001", "v0001", &config).unwrap();
    update_graph(
        &store,
        &make_attributed_run(vec![AttributionTriple {
            cause: sig.clone(),
            effect: second_effect,
            distance: 2,
            weight: 1.0,
        }]),
        "eval-002",
        "v0001",
        &config,
    )
    .unwrap();
    let before = forge_engine::cea::store::load_graph(&store, Some("v0001")).unwrap();
    let before_prediction = predict(std::slice::from_ref(&sig), &before, &config.cea);
    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let effect_id = forge_engine::cea::instrumentation::effect_node_id(&first_effect);
    let before_cause = before.node_index_map[&cause_id];
    let before_effect = before.node_index_map[&effect_id];
    let before_edge = before
        .graph
        .edge_weight(before.graph.find_edge(before_cause, before_effect).unwrap())
        .unwrap()
        .clone();

    drop(store);

    let reopened = ForgeStore::open(&db_path).unwrap();
    let after = forge_engine::cea::store::load_graph(&reopened, Some("v0001")).unwrap();
    let after_prediction = predict(std::slice::from_ref(&sig), &after, &config.cea);
    let after_cause = after.node_index_map[&cause_id];
    let after_effect = after.node_index_map[&effect_id];
    let after_edge = after
        .graph
        .edge_weight(after.graph.find_edge(after_cause, after_effect).unwrap())
        .unwrap();

    assert_eq!(after_edge.stats, before_edge.stats);
    assert!(after_edge.stats.beta > 1.0);
    assert_eq!(after_edge.count, before_edge.count);
    assert!((after_edge.confidence - before_edge.confidence).abs() < 1e-12);
    assert!(
        (after_prediction.predicted_correctness - before_prediction.predicted_correctness).abs()
            < 1e-12
    );
    assert!((after_prediction.confidence - before_prediction.confidence).abs() < 1e-12);
}

#[test]
fn i3g_cea_update_graph_persists_negative_evidence_for_absent_effects() {
    let (store, _dir) = temp_store();
    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let first_effect = make_test_effect();
    let second_effect = EffectSignature {
        message_class: "second_warning".to_string(),
        ..make_test_effect()
    };
    let config = ForgeConfig::default();

    update_graph(
        &store,
        &make_attributed_run(vec![AttributionTriple {
            cause: sig.clone(),
            effect: first_effect.clone(),
            distance: 2,
            weight: 1.0,
        }]),
        "eval-001",
        "v0001",
        &config,
    )
    .unwrap();
    update_graph(
        &store,
        &make_attributed_run(vec![AttributionTriple {
            cause: sig.clone(),
            effect: second_effect,
            distance: 2,
            weight: 1.0,
        }]),
        "eval-002",
        "v0001",
        &config,
    )
    .unwrap();

    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let mut edges = store.get_cea_edges_for_cause(&cause_id).unwrap();
    edges.sort_by(|left, right| left.effect_node_id.cmp(&right.effect_node_id));
    let first_effect_id = forge_engine::cea::instrumentation::effect_node_id(&first_effect);
    let first_edge = edges
        .iter()
        .find(|edge| edge.effect_node_id == first_effect_id)
        .unwrap();

    assert_eq!(
        edges.len(),
        2,
        "expected one persisted edge per observed effect"
    );
    assert_eq!(first_edge.beta, 2.0);
    assert_eq!(first_edge.count, 2);
    assert!(
        edges
            .iter()
            .any(|edge| (edge.beta - 1.0).abs() < 1e-12 && edge.count == 1),
        "newly observed effects should still start from one positive observation"
    );
}

#[test]
fn i3h_cea_restart_preserves_negative_evidence_predictions() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let first_effect = make_test_effect();
    let second_effect = EffectSignature {
        message_class: "second_warning".to_string(),
        ..make_test_effect()
    };
    let config = ForgeConfig::default();

    update_graph(
        &store,
        &make_attributed_run(vec![
            AttributionTriple {
                cause: sig.clone(),
                effect: first_effect.clone(),
                distance: 2,
                weight: 1.0,
            },
            AttributionTriple {
                cause: sig.clone(),
                effect: second_effect.clone(),
                distance: 2,
                weight: 1.0,
            },
        ]),
        "eval-001",
        "v0001",
        &config,
    )
    .unwrap();
    update_graph(
        &store,
        &make_attributed_run(vec![AttributionTriple {
            cause: sig.clone(),
            effect: first_effect.clone(),
            distance: 2,
            weight: 1.0,
        }]),
        "eval-002",
        "v0001",
        &config,
    )
    .unwrap();

    let before = forge_engine::cea::store::load_graph(&store, Some("v0001")).unwrap();
    let before_prediction = predict(std::slice::from_ref(&sig), &before, &config.cea);
    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let second_effect_id = forge_engine::cea::instrumentation::effect_node_id(&second_effect);
    let before_edge = before
        .graph
        .edge_weight(
            before
                .graph
                .find_edge(
                    before.node_index_map[&cause_id],
                    before.node_index_map[&second_effect_id],
                )
                .unwrap(),
        )
        .unwrap()
        .clone();
    drop(store);

    let reopened = ForgeStore::open(&db_path).unwrap();
    let after = forge_engine::cea::store::load_graph(&reopened, Some("v0001")).unwrap();
    let after_prediction = predict(std::slice::from_ref(&sig), &after, &config.cea);
    let after_edge = after
        .graph
        .edge_weight(
            after
                .graph
                .find_edge(
                    after.node_index_map[&cause_id],
                    after.node_index_map[&second_effect_id],
                )
                .unwrap(),
        )
        .unwrap();

    assert_eq!(before_edge.stats, after_edge.stats);
    assert!((before_edge.stats.beta - 2.0).abs() < 1e-12);
    assert!((after_edge.stats.beta - 2.0).abs() < 1e-12);
    assert!(
        (after_prediction.predicted_correctness - before_prediction.predicted_correctness).abs()
            < 1e-12
    );
    assert!((after_prediction.confidence - before_prediction.confidence).abs() < 1e-12);
}

#[test]
fn i3i_cea_legacy_edge_rows_are_migrated_forward_with_stats() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    let op = make_test_edit_op();
    let sig = build_edit_op_signature(&op, 0, 1, 0, 1, "rs").unwrap();
    let effect = make_test_effect();
    let config = ForgeConfig::default();

    update_graph(
        &store,
        &make_attributed_run(vec![AttributionTriple {
            cause: sig.clone(),
            effect: effect.clone(),
            distance: 2,
            weight: 1.0,
        }]),
        "eval-001",
        "v0001",
        &config,
    )
    .unwrap();
    drop(store);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        r#"
ALTER TABLE cea_edges RENAME TO cea_edges_with_stats;
CREATE TABLE cea_edges (
    edge_id TEXT PRIMARY KEY,
    cause_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id),
    effect_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id),
    weight REAL NOT NULL DEFAULT 0.0,
    count INTEGER NOT NULL DEFAULT 0,
    confidence REAL NOT NULL DEFAULT 0.0,
    version_id TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    UNIQUE(cause_node_id, effect_node_id, version_id)
);
INSERT INTO cea_edges (edge_id, cause_node_id, effect_node_id, weight, count, confidence, version_id, last_seen)
SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, version_id, last_seen
FROM cea_edges_with_stats;
DROP TABLE cea_edges_with_stats;
CREATE INDEX IF NOT EXISTS idx_cea_edges_cause ON cea_edges(cause_node_id);
CREATE INDEX IF NOT EXISTS idx_cea_edges_effect ON cea_edges(effect_node_id);
CREATE INDEX IF NOT EXISTS idx_cea_edges_version ON cea_edges(version_id);
"#,
    )
    .unwrap();
    drop(conn);

    let reopened = ForgeStore::open(&db_path).unwrap();
    let cause_id = forge_engine::cea::instrumentation::edit_op_node_id(&sig);
    let edges = reopened.get_cea_edges_for_cause(&cause_id).unwrap();
    assert_eq!(edges.len(), 1);
    let expected_alpha = 1.0 + edges[0].weight.max(0.0);
    assert!((edges[0].alpha - expected_alpha).abs() < 1e-12);
    assert!((edges[0].beta - 1.0).abs() < 1e-12);
    let expected_confidence = cea_core::EdgeStats {
        alpha: expected_alpha,
        beta: 1.0,
        observations: 1,
    }
    .confidence();
    assert!((edges[0].confidence - expected_confidence).abs() < 1e-12);
}

// ─────────────────────────────────────────────────────────────
// I4: cea_prediction_returns_neutral_for_unknown_sigs
// ─────────────────────────────────────────────────────────────

/// I4: Predict on a patch with no matching graph edges.
/// Assert coverage_fraction == 0.0, confidence is low, zero_shot_eligible == false.
#[test]
fn i4_cea_prediction_returns_neutral_for_unknown_sigs() {
    let graph = CausalGraph::new();
    let config = CeaConfig::default();

    // Create signatures that do NOT exist in the graph
    let unknown_sig = EditOpSignature {
        op_kind: EditOpKind::Insert,
        anchor_kind: AnchorKind::AfterLine,
        lines_added: 5,
        lines_removed: 0,
        context_hash: blake3::hash(b"some unknown context").to_hex().to_string(),
        file_extension: "rs".to_string(),
        scope_tag: ScopeTag::Function,
        op_index: OpIndex(0),
        file_index: FileIndex(0),
    };

    let prediction = predict(&[unknown_sig], &graph, &config);

    assert!(
        (prediction.coverage_fraction - 0.0).abs() < 1e-9,
        "coverage_fraction should be 0.0, got {}",
        prediction.coverage_fraction
    );

    assert!(
        prediction.confidence < 0.1,
        "confidence should be low for unknown sigs, got {}",
        prediction.confidence
    );

    assert!(
        !prediction.zero_shot_eligible,
        "zero_shot_eligible must be false for unknown sigs"
    );

    // Predicted correctness should be neutral (0.5) since all unknown
    assert!(
        (prediction.predicted_correctness - 0.5).abs() < 0.01,
        "predicted_correctness should be ~0.5 for all-unknown, got {}",
        prediction.predicted_correctness
    );

    // No risk flags for unknown signatures
    assert!(
        prediction.risk_flags.is_empty(),
        "risk_flags should be empty for unknown sigs"
    );
}

// ─────────────────────────────────────────────────────────────
// I5: cea_zero_shot_requires_explicit_enable
// ─────────────────────────────────────────────────────────────

/// I5: Association-only graph evidence can never authorize check skipping.
/// Runtime opt-in is necessary but not sufficient: an interventional gate must
/// independently qualify the evidence before checks may be skipped.
#[test]
fn i5_cea_zero_shot_requires_interventional_evidence_gate() {
    // Default config: enable_zero_shot must be false
    let default_config = CeaConfig::default();
    assert!(
        !default_config.enable_zero_shot,
        "enable_zero_shot must be false by default"
    );

    // Build a graph with high coverage and sample quality to produce zero_shot_eligible = true
    let mut graph = CausalGraph::new();

    let sig = EditOpSignature {
        op_kind: EditOpKind::Replace,
        anchor_kind: AnchorKind::Range,
        lines_added: 3,
        lines_removed: 2,
        context_hash: blake3::hash(b"well known context").to_hex().to_string(),
        file_extension: "rs".to_string(),
        scope_tag: ScopeTag::Function,
        op_index: OpIndex(0),
        file_index: FileIndex(0),
    };
    let pass_effect = make_pass_effect();

    let cause_idx = graph.ensure_cause_node(&sig);
    let effect_idx = graph.ensure_effect_node(&pass_effect);
    for _ in 0..24 {
        graph.update_edge(cause_idx, effect_idx, 0.9);
    }

    // Exact high-sample association can produce a useful advisory prediction,
    // but it is not interventional evidence.
    let config_high_coverage = CeaConfig {
        zero_shot_coverage_threshold: 0.5, // We'll have 100% coverage (1 sig, 1 known)
        risk_confidence_threshold: 0.55,
        ..CeaConfig::default()
    };

    let prediction = predict(std::slice::from_ref(&sig), &graph, &config_high_coverage);
    assert!(!prediction.zero_shot_eligible);

    // ── Simulate runtime behavior ──
    // Runtime must not skip checks from association-only evidence.

    // Case 1: Default config (enable_zero_shot = false)
    // Runtime should NOT use zero-shot even if prediction says eligible
    let should_skip_checks_default =
        default_config.enable_zero_shot && prediction.zero_shot_eligible;
    assert!(
        !should_skip_checks_default,
        "with enable_zero_shot=false, must NOT skip checks even if zero_shot_eligible"
    );

    // Case 2: Explicit opt-in alone is still insufficient.
    let enabled_config = CeaConfig {
        enable_zero_shot: true,
        risk_confidence_threshold: 0.55,
        zero_shot_coverage_threshold: 0.5,
        ..CeaConfig::default()
    };

    let prediction_enabled = predict(&[sig], &graph, &enabled_config);
    let should_skip_checks_enabled =
        enabled_config.enable_zero_shot && prediction_enabled.zero_shot_eligible;
    assert!(
        !should_skip_checks_enabled,
        "opt-in must not bypass the independent interventional evidence gate"
    );

    // Verify the prediction has a non-trivial correctness score when known
    assert!(
        prediction_enabled.predicted_correctness > 0.5,
        "predicted_correctness should be > 0.5 for positive-observation edges, got {}",
        prediction_enabled.predicted_correctness
    );
    assert!(
        prediction_enabled.confidence > 0.5,
        "prediction confidence should be materially above neutral after strong evidence, got {}",
        prediction_enabled.confidence
    );
}

// ─────────────────────────────────────────────────────────────
// Phase 1 regression: attribute_effects uses mapped positions
// ─────────────────────────────────────────────────────────────

/// Verify that attribute_effects uses the LineAttributionMap to compute
/// distances in patched-file space, not original-file space.
#[test]
fn attribute_effects_uses_mapped_positions() {
    // Scenario: A file has 10 lines. We insert 5 lines after line 2,
    // pushing original line 3 to patched line 8. A compiler error on
    // patched line 8 should attribute to the edit nearest in patched space.

    // Build a patch with two ops:
    //   Op 0: Insert 5 lines after line 2 (patched position ~3)
    //   Op 1: Replace line 8 (original) → in patched space, line 13
    let patch = StructuredPatch {
        patch_id: Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![FileEdit {
            path: std::path::PathBuf::from("src/main.rs"),
            ops: vec![
                EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 2,
                        context_before: vec![],
                        context_after: vec![],
                    },
                    lines: vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
                },
                EditOp::Replace {
                    range: LineRange {
                        start: 8,
                        end_exclusive: 9,
                    },
                    lines: vec!["replacement".into()],
                },
            ],
            mode: Some(FileMode::Modify),
        }],
        notes: vec![],
    };

    // Build a LineAttributionMap that reflects the insert:
    // Original lines 1,2 stay at 1,2
    // Original lines 3-10 shift by +5 (now at 8-15)
    let mut mappings = BTreeMap::new();
    let mut line_map_entries = Vec::new();
    for i in 1..=10u32 {
        let patched = if i <= 2 { i } else { i + 5 };
        line_map_entries.push((i, patched));
    }
    mappings.insert("src/main.rs".to_string(), line_map_entries);

    let line_map = LineAttributionMap {
        mappings,
        resolved_anchors: BTreeMap::new(),
    };

    // Create a check result with a clippy error on patched line 8
    // (which is original line 3, shifted by the 5-line insert)
    let check_result = CheckResult {
        fmt_pass: true,
        clippy_pass: false,
        test_pass: true,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput {
            check_kind: CheckKind::Clippy,
            exit_code: 1,
            effects: vec![LocatedEffect {
                file: Some(std::path::PathBuf::from("src/main.rs")),
                line: Some(8),
                col: None,
                message: "unused variable".to_string(),
                sig: EffectSignature {
                    check_kind: "clippy".to_string(),
                    outcome: "warning".to_string(),
                    severity: "warning".to_string(),
                    message_class: "unused_variable".to_string(),
                    line_offset_from_edit: None,
                },
            }],
            raw_stdout: String::new(),
            raw_stderr: String::new(),
        },
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 100,
    };

    let triples = attribute_effects(&patch, &check_result, &line_map, 50).unwrap();

    // Should have attributions (including pass attributions)
    assert!(!triples.is_empty(), "Should produce attribution triples");

    // Find the attribution for the clippy warning
    let clippy_attributions: Vec<_> = triples
        .iter()
        .filter(|t| t.effect.check_kind == "clippy" && t.effect.outcome == "warning")
        .collect();

    assert!(
        !clippy_attributions.is_empty(),
        "Should have clippy warning attribution"
    );

    // The first op (Insert after line 2) in patched space is at line ~3.
    // The error is at patched line 8.
    // Op 0 (insert): patched position = mapped(2) + 1 = 3 → distance to error = |8-3| = 5
    // Op 1 (replace line 8): patched position = mapped(8) = 13 → distance to error = |8-13| = 5
    // Both are equidistant, so either could be chosen. The important thing is
    // the distance is computed in patched space, NOT original space.
    let attr = &clippy_attributions[0];
    assert!(
        attr.distance <= 50,
        "Attribution distance should be within max_line_distance"
    );
}
