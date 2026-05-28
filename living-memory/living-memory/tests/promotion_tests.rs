#![allow(clippy::expect_used)]

use forge_engine::config::ForgeConfig;
use forge_engine::lab::emitters::AlgebraSpec;
use forge_engine::lab::evidence::{
    AssessmentCategory, ContradictionState, EvidenceAssessment, SampleSupport,
};
use forge_engine::lab::promote::{promote, CausalFingerprint};
use forge_engine::store::db::PromotionRow;
use forge_engine::{ForgeError, ForgeStore, ScoreVector};
use tempfile::TempDir;

fn open_test_store() -> (TempDir, ForgeStore) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    (dir, store)
}

fn insert_candidate(store: &ForgeStore, candidate_id: &str) {
    let spec = serde_json::to_string(&AlgebraSpec::default()).unwrap();
    store
        .insert_candidate(candidate_id, &spec, "[]", "active")
        .unwrap();
}

fn insert_eval_run(
    store: &ForgeStore,
    candidate_id: &str,
    eval_id: &str,
    correctness: f64,
    weighted_total: f64,
    violations_json: &str,
    cea_run_hash: Option<&str>,
) {
    let scores = ScoreVector {
        correctness,
        novelty: 0.1,
        stability: 0.0,
        weighted_total,
        cea_confidence: None,
        cea_predicted_correctness: None,
    };
    store
        .insert_eval_run(
            eval_id,
            candidate_id,
            "task-1",
            "host",
            0,
            &format!("mindstate:{eval_id}"),
            &format!("patch:{eval_id}"),
            &format!("sig:{eval_id}"),
            &serde_json::to_string(&scores).unwrap(),
            violations_json,
            "inline:test",
            cea_run_hash,
        )
        .unwrap();
}

fn insert_evidence_assessment(
    store: &ForgeStore,
    candidate_id: &str,
    bundle_id: &str,
    assessment: EvidenceAssessment,
) {
    store
        .insert_evidence_bundle(
            bundle_id,
            candidate_id,
            &format!("eval:{bundle_id}"),
            "v0001",
            &format!("trace:{bundle_id}"),
            "{}",
            "[]",
            None,
            None,
            Some(&serde_json::to_string(&assessment).unwrap()),
            "[]",
        )
        .unwrap();
}

fn clean_assessment() -> EvidenceAssessment {
    EvidenceAssessment {
        reproducibility: AssessmentCategory::Strong,
        isolation: AssessmentCategory::Strong,
        contradiction_state: ContradictionState::Clean,
        sample_support: SampleSupport::Sufficient,
    }
}

fn latest_promotion(store: &ForgeStore) -> PromotionRow {
    store
        .get_latest_promotion()
        .unwrap()
        .expect("promotion should exist")
}

fn candidate_status(store: &ForgeStore, candidate_id: &str) -> String {
    let conn = rusqlite::Connection::open(store.path()).unwrap();
    conn.query_row(
        "SELECT status FROM candidates WHERE candidate_id = ?1",
        [candidate_id],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn promotion_rejects_missing_verification_evidence() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    insert_candidate(&store, "cand-no-evidence");
    insert_eval_run(&store, "cand-no-evidence", "eval-1", 1.0, 0.20, "[]", None);

    let err = promote(&store, "cand-no-evidence", &config).unwrap_err();
    match err {
        ForgeError::PromotionFailed { criterion, .. } => {
            assert_eq!(criterion, "verification_evidence");
        }
        other => panic!("expected PromotionFailed, got {other:?}"),
    }
}

#[test]
fn promotion_rejects_when_improvement_does_not_clear_baseline() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    insert_candidate(&store, "baseline");
    insert_eval_run(
        &store,
        "baseline",
        "eval-base-1",
        1.0,
        0.30,
        "[]",
        Some("edge-a"),
    );
    insert_eval_run(
        &store,
        "baseline",
        "eval-base-2",
        1.0,
        0.30,
        "[]",
        Some("edge-b"),
    );
    let baseline_fp = CausalFingerprint {
        dominant_edge_hashes: vec!["edge-a".into(), "edge-b".into()],
        checksum: blake3::hash(b"edge-a,edge-b").to_hex().to_string(),
    };
    store
        .insert_promotion(
            "v0001",
            "baseline",
            &serde_json::to_string(&AlgebraSpec::default()).unwrap(),
            "{}",
            "{}",
            "baseline-checksum",
            Some(&serde_json::to_string(&baseline_fp).unwrap()),
        )
        .unwrap();

    insert_candidate(&store, "cand-low-improvement");
    insert_eval_run(
        &store,
        "cand-low-improvement",
        "eval-cand-1",
        1.0,
        0.33,
        "[]",
        Some("edge-a"),
    );
    insert_eval_run(
        &store,
        "cand-low-improvement",
        "eval-cand-2",
        1.0,
        0.31,
        "[]",
        Some("edge-b"),
    );
    insert_evidence_assessment(
        &store,
        "cand-low-improvement",
        "bundle-low-improvement",
        clean_assessment(),
    );

    let err = promote(&store, "cand-low-improvement", &config).unwrap_err();
    match err {
        ForgeError::PromotionFailed { criterion, .. } => {
            assert_eq!(criterion, "weighted_improvement");
        }
        other => panic!("expected PromotionFailed, got {other:?}"),
    }
}

#[test]
fn promotion_rejects_contradicted_evidence_assessment() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    insert_candidate(&store, "cand-contradicted");
    insert_eval_run(&store, "cand-contradicted", "eval-1", 1.0, 0.20, "[]", None);
    let mut assessment = clean_assessment();
    assessment.contradiction_state = ContradictionState::HasContradictions;
    insert_evidence_assessment(
        &store,
        "cand-contradicted",
        "bundle-contradicted",
        assessment,
    );

    let err = promote(&store, "cand-contradicted", &config).unwrap_err();
    match err {
        ForgeError::PromotionFailed { criterion, .. } => {
            assert_eq!(criterion, "contradiction_state");
        }
        other => panic!("expected PromotionFailed, got {other:?}"),
    }
}

#[test]
fn promotion_succeeds_with_deterministic_eval_and_evidence_gate() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    insert_candidate(&store, "cand-promote");
    insert_eval_run(
        &store,
        "cand-promote",
        "eval-1",
        1.0,
        0.22,
        "[]",
        Some("edge-a"),
    );
    insert_eval_run(
        &store,
        "cand-promote",
        "eval-2",
        1.0,
        0.21,
        "[]",
        Some("edge-b"),
    );
    insert_evidence_assessment(&store, "cand-promote", "bundle-promote", clean_assessment());

    let promoted = promote(&store, "cand-promote", &config).unwrap();

    assert_eq!(promoted.candidate_id, "cand-promote");
    assert_eq!(promoted.version_id, "v0001");
    assert_eq!(candidate_status(&store, "cand-promote"), "promoted");

    let stored = latest_promotion(&store);
    assert_eq!(stored.candidate_id, "cand-promote");
    assert_eq!(stored.version_id, "v0001");
    assert!(stored.cea_fingerprint_json.is_some());
}

#[test]
fn promotion_reads_assessment_from_canonical_bundle_not_legacy_split_column() {
    let (_dir, store) = open_test_store();
    let config = ForgeConfig::default();

    insert_candidate(&store, "cand-canonical-assessment");
    insert_eval_run(
        &store,
        "cand-canonical-assessment",
        "eval-1",
        1.0,
        0.22,
        "[]",
        Some("edge-a"),
    );
    insert_eval_run(
        &store,
        "cand-canonical-assessment",
        "eval-2",
        1.0,
        0.21,
        "[]",
        Some("edge-b"),
    );
    insert_evidence_assessment(
        &store,
        "cand-canonical-assessment",
        "bundle-canonical-assessment",
        clean_assessment(),
    );

    let stale_assessment = serde_json::to_string(&EvidenceAssessment {
        reproducibility: AssessmentCategory::Weak,
        isolation: AssessmentCategory::Weak,
        contradiction_state: ContradictionState::HasContradictions,
        sample_support: SampleSupport::Insufficient,
    })
    .unwrap();

    let conn = rusqlite::Connection::open(store.path()).unwrap();
    conn.execute(
        "UPDATE evidence_bundles SET assessment_json = ?1 WHERE bundle_id = ?2",
        rusqlite::params![stale_assessment, "bundle-canonical-assessment"],
    )
    .unwrap();
    drop(conn);

    let promoted = promote(&store, "cand-canonical-assessment", &config).unwrap();
    assert_eq!(promoted.candidate_id, "cand-canonical-assessment");
    assert_eq!(
        candidate_status(&store, "cand-canonical-assessment"),
        "promoted"
    );
}
