mod common;

use forge_engine::{
    CargoAdapter, ExperimentConfig, ExperimentMode, HostBackend, HypothesisStatus,
    PairedExperimentRunner, ReceiptStorage,
};
use forge_pilot::act::{OracleExecution, PlanKind};
use forge_pilot::bundle_builder::{
    build_bundle_from_oracle, build_bundle_from_patch, OracleBundleInput, PatchBundleInput,
};

#[test]
fn missing_oracle_outcome_cannot_fabricate_support_pair_or_completed_trial() {
    let oracle = OracleExecution {
        assessment: None,
        delta_parity: None,
        temporal_replay: None,
        refutation: None,
    };
    let bundle = build_bundle_from_oracle(OracleBundleInput {
        plan: &PlanKind::OracleConservative,
        target_key: "test",
        trace_id: None,
        scope_namespace: "isolated",
        oracle_execution: &oracle,
        known_threats: vec![],
    });
    assert!(bundle.pair_comparability.is_none());
    assert_eq!(bundle.hypotheses[0].status, HypothesisStatus::Proposed);
    assert_eq!(bundle.hypotheses[0].support_count, 0);
    assert!(bundle.verification_trials.is_empty());
    assert!(bundle.primary_effect.is_none());
    assert_eq!(bundle.scores.weighted_total, 0.0);
    assert!(bundle.promotion_state.is_none());
    let raw = bundle.to_canonical_evidence_bundle();
    assert_eq!(raw.trial_count, 0);
    let restored =
        forge_engine::ExperimentEvidenceBundle::from_canonical_evidence_bundle(&raw).unwrap();
    assert!(
        restored.pair_comparability.is_none(),
        "not-applicable must survive rehydration"
    );
    assert!(
        restored.treatment.is_none(),
        "an unpaired oracle cannot rehydrate as patch_applied"
    );
    assert_eq!(
        raw.verification_summary.unwrap().lifecycle_state,
        semantic_memory_forge::VerificationLifecycleState::Unverified
    );
}

#[tokio::test]
async fn repeated_raw_trials_and_exact_patch_identity_survive_bundle_and_v3_export() {
    let dir = common::tempdir();
    let (fixture, patch) = common::write_patch_fixture(dir.path());
    let cfg = forge_engine::ForgeConfig {
        execution_backend_preference: "host".into(),
        ..Default::default()
    };
    let backend = HostBackend::new(&cfg);
    let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
    let experiment = ExperimentConfig {
        mode: ExperimentMode::RepeatedPaired,
        trial_count: 3,
        ..Default::default()
    };
    let result = runner.run(&fixture, &patch, &experiment).await.unwrap();
    let plan = PlanKind::PairedPatch {
        fixture_path: fixture.to_string_lossy().into(),
        patch,
        experiment_config: experiment,
        description: "test only".into(),
    };
    let bundle = build_bundle_from_patch(PatchBundleInput {
        plan: &plan,
        target_key: "test",
        trace_id: None,
        scope_namespace: "isolated",
        experiment_result: &result,
        known_threats: vec![],
    });
    assert_eq!(bundle.verification_trials.len(), 6);
    assert_eq!(
        bundle.patch_hash.as_deref(),
        Some(result.patch_hash.as_str())
    );
    assert_eq!(
        bundle.pair_comparability.as_ref().unwrap().valid,
        result.pair_comparability.valid
    );
    assert_eq!(
        bundle.scores.weighted_total, 0.0,
        "diagnostic counters cannot invent acceptance utility"
    );
    for (raw, projected) in result.trials.iter().zip(&bundle.verification_trials) {
        assert_eq!(raw.trial_id, projected.trial_id.to_string());
        assert_eq!(raw.completed, projected.completed);
    }
    let ReceiptStorage::Inline(payload) = &bundle.receipts[0].storage else {
        panic!("missing inline trial receipts")
    };
    assert!(bundle.receipts[0].verify_content(payload.as_bytes()));
    let value: serde_json::Value = serde_json::from_str(payload).unwrap();
    assert_eq!(value["trials"].as_array().unwrap().len(), 6);
    assert!(value["trials"][5]["check_outputs"][2]["raw_stdout"].is_string());
    let raw = bundle.to_canonical_evidence_bundle();
    assert_eq!(raw.verification_trials.len(), 6);
    assert_eq!(raw.trial_count, 6);
    assert!(
        !raw.variance_aware,
        "replicated commands did not estimate task-level variance"
    );
    let memory = common::open_memory_store(dir.path());
    let forge = common::open_forge_store(dir.path());
    let first = forge_pilot::canonical_roundtrip(&bundle, "isolated", &forge, &memory)
        .await
        .unwrap();
    assert_eq!(first.import_result.status, "complete");
    let replay = forge_pilot::canonical_roundtrip(&bundle, "isolated", &forge, &memory)
        .await
        .unwrap();
    // Export attempts have distinct observed timestamps; that field is outside
    // the canonical content digest. Immutable material and import identity must
    // remain equal. Do not assert a false byte-for-byte timestamp replay law.
    let mut first_material = serde_json::to_value(&first.envelope).unwrap();
    let mut replay_material = serde_json::to_value(&replay.envelope).unwrap();
    first_material
        .as_object_mut()
        .unwrap()
        .remove("exported_at");
    replay_material
        .as_object_mut()
        .unwrap()
        .remove("exported_at");
    // ForgeExportMeta repeats the same export-attempt timestamp (export.rs
    // build_canonical_export_material); it is not immutable experiment data.
    for material in [&mut first_material, &mut replay_material] {
        assert!(material
            .get_mut("export_meta")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("exported_at")
            .is_some());
    }
    assert_eq!(
        first.envelope.content_digest,
        replay.envelope.content_digest
    );
    assert_eq!(
        stack_ids::ContentDigest::compute_str(&first_material.to_string()),
        stack_ids::ContentDigest::compute_str(&replay_material.to_string()),
        "canonical export material changed on replay",
    );
    let imports = memory
        .query_projection_imports(Some("isolated"), 10)
        .await
        .unwrap();
    assert_eq!(
        imports.len(),
        1,
        "idempotent replay must not create a second import"
    );
}

#[tokio::test]
async fn no_op_and_incomparable_results_do_not_gain_support() {
    let dir = common::tempdir();
    let (fixture, mut patch) = common::write_patch_fixture(dir.path());
    // Keep an explicit syntactic patch but replace the source line with itself.
    let text = std::fs::read_to_string(fixture.join("src/lib.rs")).unwrap();
    if let forge_engine::EditOp::Replace { range, lines } = &mut patch.edits[0].ops[0] {
        *lines = text
            .lines()
            .skip(range.start as usize - 1)
            .take((range.end_exclusive - range.start) as usize)
            .map(str::to_owned)
            .collect();
    } else {
        panic!("unexpected fixture patch")
    }
    let cfg = forge_engine::ForgeConfig::default();
    let backend = HostBackend::new(&cfg);
    let mut result = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg)
        .run(&fixture, &patch, &ExperimentConfig::default())
        .await
        .unwrap();
    let plan = PlanKind::PairedPatch {
        fixture_path: fixture.to_string_lossy().into(),
        patch,
        experiment_config: ExperimentConfig::default(),
        description: "no-op negative control".into(),
    };
    for incomparable in [false, true] {
        if incomparable {
            result.pair_comparability.valid = false;
            result
                .pair_comparability
                .violations
                .push("injected provenance drift".into());
        }
        let bundle = build_bundle_from_patch(PatchBundleInput {
            plan: &plan,
            target_key: "test",
            trace_id: None,
            scope_namespace: "isolated",
            experiment_result: &result,
            known_threats: vec![],
        });
        assert_eq!(bundle.hypotheses[0].status, HypothesisStatus::Proposed);
        assert_eq!(bundle.hypotheses[0].support_count, 0);
        assert_eq!(bundle.scores.weighted_total, 0.0);
        assert!(bundle.promotion_state.is_none());
        assert_eq!(
            bundle.pair_comparability.as_ref().unwrap().valid,
            !incomparable
        );
    }
}
