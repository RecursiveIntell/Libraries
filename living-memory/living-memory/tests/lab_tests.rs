//! H. Lab pipeline tests (H1–H4)

use forge_engine::lab::suite::TaskExpected;
use forge_engine::*;
use tempfile::TempDir;

const DEFAULT_EXPECTED: TaskExpected = TaskExpected {
    require_fmt: true,
    require_clippy: true,
    require_tests: true,
};

fn open_test_store() -> (TempDir, ForgeStore) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    (dir, store)
}

/// H1: evaluate_one_candidate_one_task_mocked_patch
#[test]
fn h1_evaluate_one_candidate_one_task_mocked_patch() {
    let (_dir, store) = open_test_store();

    // Insert a candidate
    let spec = serde_json::to_string(&AlgebraSpec::default()).unwrap();
    store
        .insert_candidate("candidate-1", &spec, "[]", "active")
        .unwrap();

    // Create a mocked check result
    let check = CheckResult {
        fmt_pass: true,
        clippy_pass: true,
        test_pass: true,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 100,
    };

    let correctness = compute_correctness(&check, &DEFAULT_EXPECTED);
    assert!(
        correctness > 0.0,
        "All checks pass, correctness should be > 0"
    );
    assert!(
        (correctness - 1.0).abs() < 0.01,
        "All checks pass: raw correctness should be 1.0"
    );

    // Persist eval run
    let result = EvalRunResult {
        eval_id: "eval-1".to_string(),
        candidate_id: "candidate-1".to_string(),
        task_id: "task-1".to_string(),
        scores: ScoreVector {
            correctness,
            novelty: 0.0,
            stability: 0.0,
            weighted_total: correctness,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        check_result: Some(check),
        patch_hash: "abc".to_string(),
        structural_sig: "def".to_string(),
        mindstate_hash: "ghi".to_string(),
    };

    persist_eval_run(&store, &result, "host", "[]", "inline:test", None).unwrap();

    // Verify persisted
    let runs = store.get_eval_runs_for_candidate("candidate-1").unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].eval_id, "eval-1");
}

/// H2: archive_insert_replaces_lower_score
#[test]
fn h2_archive_insert_replaces_lower_score() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![forge_engine::FileEdit {
            path: std::path::PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Insert {
                anchor: forge_engine::Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change".to_string()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec![],
    };

    // Insert candidate A with score 0.7 (correctness > gate 0.95 required)
    let scores_a = ScoreVector {
        correctness: 0.96,
        novelty: 0.1,
        stability: 0.0,
        weighted_total: 0.7,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    let result_a = archive_insert(&store, "cand-A", &scores_a, &patch, &config, None).unwrap();
    assert!(
        result_a == ArchiveUpdate::Inserted,
        "First insert should succeed"
    );

    // Insert candidate B with higher score
    let scores_b = ScoreVector {
        correctness: 0.98,
        novelty: 0.15,
        stability: 0.0,
        weighted_total: 0.9,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    let result_b = archive_insert(&store, "cand-B", &scores_b, &patch, &config, None).unwrap();
    assert_eq!(result_b, ArchiveUpdate::Replaced);
}

/// H3: archive_insert_below_gate_discarded
#[test]
fn h3_archive_insert_below_gate_discarded() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![forge_engine::FileEdit {
            path: std::path::PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Insert {
                anchor: forge_engine::Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change".to_string()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec![],
    };

    // Insert with correctness below gate (0.80 < 0.95)
    let scores = ScoreVector {
        correctness: 0.80,
        novelty: 0.1,
        stability: 0.0,
        weighted_total: 0.6,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    let result = archive_insert(&store, "cand-low", &scores, &patch, &config, None).unwrap();
    assert_eq!(result, ArchiveUpdate::BelowGate);
}

/// H4: archive_preserves_higher_score
#[test]
fn h4_archive_preserves_higher_score() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![forge_engine::FileEdit {
            path: std::path::PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Insert {
                anchor: forge_engine::Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change".to_string()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec![],
    };

    // Insert A with score 0.98
    let scores_a = ScoreVector {
        correctness: 0.98,
        novelty: 0.15,
        stability: 0.0,
        weighted_total: 0.9,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    archive_insert(&store, "cand-A", &scores_a, &patch, &config, None).unwrap();

    // Insert B with lower score 0.96
    let scores_b = ScoreVector {
        correctness: 0.96,
        novelty: 0.1,
        stability: 0.0,
        weighted_total: 0.8,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    let result = archive_insert(&store, "cand-B", &scores_b, &patch, &config, None).unwrap();
    assert_eq!(
        result,
        ArchiveUpdate::NoChange,
        "Lower score should not replace higher"
    );
}

/// H5: Raw scoring model — correctness is always in [0, 1], task weights applied only in weighted_total.
#[test]
fn h5_raw_scoring_model() {
    let (_dir, store) = open_test_store();

    // All checks fail
    let check_fail = CheckResult {
        fmt_pass: false,
        clippy_pass: false,
        test_pass: false,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 50,
    };
    let c_fail = compute_correctness(&check_fail, &DEFAULT_EXPECTED);
    assert!(
        (c_fail - 0.0).abs() < 1e-9,
        "All-fail correctness should be 0.0, got {c_fail}"
    );

    // Only fmt passes
    let check_fmt = CheckResult {
        fmt_pass: true,
        clippy_pass: false,
        test_pass: false,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 50,
    };
    let c_fmt = compute_correctness(&check_fmt, &DEFAULT_EXPECTED);
    assert!(
        (c_fmt - 0.10).abs() < 1e-9,
        "Fmt-only correctness should be 0.10, got {c_fmt}"
    );

    // All checks pass
    let check_all = CheckResult {
        fmt_pass: true,
        clippy_pass: true,
        test_pass: true,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 50,
    };
    let c_all = compute_correctness(&check_all, &DEFAULT_EXPECTED);
    assert!(
        (c_all - 1.0).abs() < 1e-9,
        "All-pass correctness should be 1.0, got {c_all}"
    );

    // Verify compute_scores applies task weights in weighted_total
    let task = EvalTask {
        task_id: "t1".to_string(),
        prompt: "Test".to_string(),
        constraints: forge_engine::lab::suite::TaskConstraints {
            allow_test_modifications: false,
            max_files_changed: None,
        },
        weights: forge_engine::lab::suite::TaskWeights {
            correctness: 0.5,
            novelty: 0.3,
            stability: 0.2,
        },
        expected: forge_engine::lab::suite::TaskExpected {
            require_fmt: true,
            require_clippy: true,
            require_tests: true,
        },
        cea: forge_engine::lab::suite::TaskCea {
            instrument: false,
            risk_threshold_override: None,
        },
    };
    let config = ForgeConfig::default();

    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![forge_engine::FileEdit {
            path: std::path::PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Insert {
                anchor: forge_engine::Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// new".to_string()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec![],
    };

    let scores = compute_scores(&check_all, &patch, &task, &store, "q1", &config).unwrap();
    // Raw correctness should be 1.0
    assert!(
        (scores.correctness - 1.0).abs() < 1e-9,
        "scores.correctness should be raw 1.0, got {}",
        scores.correctness
    );
    // weighted_total should apply task weight: 0.5 * 1.0 + ...
    assert!(
        scores.weighted_total <= 1.0 && scores.weighted_total > 0.0,
        "weighted_total should be in (0, 1], got {}",
        scores.weighted_total
    );
}

/// H6: Anti-fluff guard — identical tags + identical structural hash → novelty 0.
#[test]
fn h6_anti_fluff_guard() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "test".to_string(),
        edits: vec![forge_engine::FileEdit {
            path: std::path::PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Insert {
                anchor: forge_engine::Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// new line".to_string()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec![],
    };

    // Compute strategy tags for this patch to simulate a prior trace
    let tags = forge_engine::extract_strategy_tags(&patch);
    let tags_json = serde_json::to_string(&tags).unwrap();

    // Compute the structural hash that the anti-fluff guard uses
    let structural_hash = {
        let mut h = blake3::Hasher::new();
        for edit in &patch.edits {
            h.update(edit.path.to_string_lossy().as_bytes());
            h.update(&(edit.ops.len() as u64).to_le_bytes());
        }
        h.finalize().to_hex().to_string()
    };

    // Insert a prior trace with matching tags and structural hash as structural_sig
    store
        .insert_answer_trace(
            "trace-1",
            "q-fluff",
            "v0001",
            &tags_json,
            "patch-hash",
            &structural_hash, // the structural_sig field stores the structural sig
            "{}",
        )
        .unwrap();

    // Now compute novelty — should be 0 because tags match AND structural hash matches
    let novelty =
        forge_engine::lab::evaluate::compute_novelty_score(&patch, &store, "q-fluff", &config)
            .unwrap();
    assert!(
        (novelty - 0.0).abs() < 1e-9,
        "Novelty should be 0.0 due to anti-fluff guard, got {novelty}"
    );
}
