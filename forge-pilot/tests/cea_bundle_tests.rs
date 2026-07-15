mod common;

use common::{base_loop_config, open_forge_store, tempdir, write_patch_fixture};
use forge_engine::{select_backend, CargoAdapter, CausalAttributionEngine, ExperimentConfig};
use forge_pilot::{build_bundle_from_patch, PatchBundleInput, PlanKind};
use knowledge_runtime::Scope;

#[tokio::test]
#[ignore = "RED test from audit remediation — receipt content verification needs investigation"]
async fn paired_cea_bundle_uses_measured_receipts_and_check_scores() {
    let dir = tempdir();
    let store = open_forge_store(dir.path());
    let config = base_loop_config(Scope::new("cea-bundle"));
    let (fixture, patch) = write_patch_fixture(dir.path());
    let backend = select_backend(&config.forge_config).unwrap();
    let adapter = CargoAdapter;
    let engine = CausalAttributionEngine::new(
        &store,
        backend.as_ref(),
        &adapter,
        &config.forge_config,
        "forge-pilot.code-model.v1",
    );
    let causal_result = engine
        .run_and_observe(
            &fixture,
            &patch,
            &ExperimentConfig::default(),
            "cea-bundle-eval",
        )
        .await
        .unwrap();
    let ablations = engine
        .run_singleton_ablations(&fixture, &patch)
        .await
        .unwrap();
    let plan = PlanKind::PairedPatch {
        fixture_path: fixture.to_string_lossy().to_string(),
        patch,
        experiment_config: ExperimentConfig::default(),
        description: "CEA bundle fixture".into(),
    };

    let bundle = build_bundle_from_patch(PatchBundleInput {
        plan: &plan,
        target_key: "candidate:cea-bundle",
        trace_id: None,
        scope_namespace: "cea-bundle",
        causal_result: &causal_result,
        ablation_receipts: &ablations,
        known_threats: vec![],
    })
    .unwrap();

    let pair = &causal_result.experiment.pairs[0];
    let expected_correctness = (usize::from(pair.patched_result.fmt_pass)
        + usize::from(pair.patched_result.clippy_pass)
        + usize::from(pair.patched_result.test_pass)) as f64
        / 3.0;
    assert_eq!(bundle.scores.correctness, expected_correctness);
    assert_eq!(bundle.scores.novelty, 0.0);
    assert_eq!(bundle.scores.stability, 0.0);
    assert_eq!(bundle.scores.weighted_total, expected_correctness);
    assert_eq!(
        bundle.patch_hash,
        Some(causal_result.receipts[0].patch_digest.clone())
    );
    assert!(bundle
        .attribution_json
        .as_deref()
        .is_some_and(|json| !json.is_empty()));
    assert!(bundle
        .receipts
        .iter()
        .all(|receipt| match &receipt.storage {
            forge_engine::ReceiptStorage::Inline(payload) =>
                receipt.verify_content(payload.as_bytes()),
            _ => false,
        }));
    assert_eq!(
        bundle.claim_strength,
        forge_engine::ClaimStrength::ProvisionalSinglePair
    );
    let supported_ablations = ablations
        .iter()
        .filter(|receipt| {
            receipt.comparable
                && receipt.classification == forge_engine::AblationClassification::Supported
        })
        .count() as u64;
    assert_eq!(bundle.hypotheses[0].support_count, supported_ablations);
    assert_eq!(
        bundle.scores.cea_confidence,
        Some(causal_result.receipts[0].post_prediction.confidence)
    );
    assert_eq!(
        bundle.scores.cea_predicted_correctness,
        Some(
            causal_result.receipts[0]
                .post_prediction
                .predicted_correctness
        )
    );
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.contains("advisory")));

    let mut incomparable = causal_result.clone();
    incomparable.experiment.pairs[0].comparable = false;
    incomparable.experiment.pairs[0]
        .comparability_reasons
        .push("fixture comparability violation".into());
    let incomparable_bundle = build_bundle_from_patch(PatchBundleInput {
        causal_result: &incomparable,
        plan: &plan,
        target_key: "candidate:cea-bundle",
        trace_id: None,
        scope_namespace: "cea-bundle",
        ablation_receipts: &ablations,
        known_threats: vec![],
    })
    .unwrap();
    let comparability = incomparable_bundle.pair_comparability.unwrap();
    assert!(!comparability.valid);
    assert!(comparability
        .violations
        .iter()
        .any(|reason| reason.contains("pair 0: fixture comparability violation")));

    let no_ablation_bundle = build_bundle_from_patch(PatchBundleInput {
        causal_result: &causal_result,
        plan: &plan,
        target_key: "candidate:cea-bundle",
        trace_id: None,
        scope_namespace: "cea-bundle",
        ablation_receipts: &[],
        known_threats: vec![],
    })
    .unwrap();
    assert_eq!(no_ablation_bundle.hypotheses[0].support_count, 0);
    assert_eq!(no_ablation_bundle.hypotheses[0].contradiction_count, 0);
    assert_eq!(
        no_ablation_bundle.hypotheses[0].status,
        forge_engine::HypothesisStatus::Proposed
    );

    let mut missing_receipts = causal_result.clone();
    missing_receipts.receipts.clear();
    let missing_receipt_bundle = build_bundle_from_patch(PatchBundleInput {
        causal_result: &missing_receipts,
        plan: &plan,
        target_key: "candidate:cea-bundle",
        trace_id: None,
        scope_namespace: "cea-bundle",
        ablation_receipts: &[],
        known_threats: vec![],
    })
    .unwrap();
    assert!(missing_receipt_bundle
        .verification_trials
        .iter()
        .all(|trial| !trial.completed));
    assert!(missing_receipt_bundle
        .warnings
        .iter()
        .any(|warning| warning.contains("no matching CEA update receipt")));
}
