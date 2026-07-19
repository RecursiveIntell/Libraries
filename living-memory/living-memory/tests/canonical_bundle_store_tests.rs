#![allow(clippy::expect_used, clippy::unwrap_used)]

use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::{BundleScope, Covariates, Treatment};
use forge_engine::{ClaimStrength, ExperimentEvidenceBundle, ForgeStore};
use semantic_memory_forge::PromotionState;
use tempfile::TempDir;

fn execution_bundle(bundle_id: &str) -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.into(),
        candidate_id: "procedure:exact-source".into(),
        eval_id: "evaluation:sealed".into(),
        version_id: "aidens-execution-evidence-v1".into(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 1.0,
            novelty: 0.0,
            stability: 0.0,
            weighted_total: 1.0,
            cea_confidence: Some(1.0),
            cea_predicted_correctness: None,
        },
        hypotheses: Vec::new(),
        verification: None,
        trace_id: Some("0af7651916cd43dd8448eb211c80319c".into()),
        experiment_diff: None,
        attribution_json: Some("{\"owner\":\"cea\"}".into()),
        assessment: None,
        warnings: vec![
            "single-arm execution; no comparative effect claim".into(),
            "exact source-tree scope only".into(),
        ],
        created_at: "2026-07-18T00:00:00Z".into(),
        run_id: Some("run:exact".into()),
        attempt_id: Some("attempt:exact".into()),
        causal_question: Some(
            "Did the exact source-bound patch complete its declared sealed checks?".into(),
        ),
        unit_definition: Some("one exact patch execution on one frozen source tree".into()),
        bundle_scope: Some(BundleScope {
            workload_id: "source-tree:before".into(),
            backend_family: "sealed-container".into(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            timeout_class: "900s".into(),
            config_flags: vec!["network:none".into(), "rootfs:read-only".into()],
        }),
        pair_comparability: None,
        claim_strength: ClaimStrength::ExecutionVerifiedNoComparison,
        identification_rationale: Some(
            "observed execution only; no baseline or generalization estimator".into(),
        ),
        known_threats: vec![
            "single source tree".into(),
            "no cross-task learner treatment".into(),
        ],
        patch_hash: Some("patch:digest".into()),
        treatment: Some(Treatment {
            kind: "exact_source_bound_patch".into(),
            patch_hash: "patch:digest".into(),
            patch_summary: "one immutable structured patch".into(),
        }),
        outcome: Some("fmt, clippy, and tests passed in sealed execution".into()),
        covariates: Some(Covariates {
            env_fingerprint: "sandbox-capability:digest".into(),
            dependency_fingerprint: Some("source-tree:before".into()),
            config_flags: vec!["network:none".into()],
            workload_id: "source-tree:before".into(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            adjacent_edits: false,
            adjacent_edit_signatures: Vec::new(),
        }),
        promotion_state: Some(PromotionState::NotPromoted),
        primary_effect: None,
        all_effects: Vec::new(),
        hypothesis_edges: Vec::new(),
        receipts: Vec::new(),
        verification_trials: Vec::new(),
        refutation_artifacts: Vec::new(),
        sealed: true,
    }
}

#[test]
fn canonical_bundle_round_trips_after_reopen_and_identical_retry_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let bundle = execution_bundle("bundle:exact");

    {
        let store = ForgeStore::open(&db_path).unwrap();
        store.insert_canonical_evidence_bundle(&bundle).unwrap();
        store.insert_canonical_evidence_bundle(&bundle).unwrap();
    }

    let reopened = ForgeStore::open(&db_path).unwrap();
    let loaded = reopened
        .get_canonical_evidence_bundle(&bundle.bundle_id)
        .unwrap()
        .expect("typed canonical bundle must survive reopen");
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&bundle).unwrap()
    );
    assert_eq!(
        loaded.claim_strength,
        ClaimStrength::ExecutionVerifiedNoComparison
    );
    assert_eq!(
        loaded.claim_strength.to_string(),
        "verified local execution with no comparative effect claim"
    );
}

#[test]
fn canonical_bundle_identity_conflict_fails_closed() {
    let dir = TempDir::new().unwrap();
    let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
    let original = execution_bundle("bundle:conflict");
    store.insert_canonical_evidence_bundle(&original).unwrap();

    let mut conflicting = original;
    conflicting.candidate_id = "procedure:different".into();
    let error = store
        .insert_canonical_evidence_bundle(&conflicting)
        .unwrap_err();
    assert_eq!(error.kind(), "evidence_conflict");

    let retained = store
        .get_canonical_evidence_bundle("bundle:conflict")
        .unwrap()
        .unwrap();
    assert_eq!(retained.candidate_id, "procedure:exact-source");
}
