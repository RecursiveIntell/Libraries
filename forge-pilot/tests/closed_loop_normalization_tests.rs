mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store, point_config_at_dir,
    resources, sample_bundle, tempdir, write_source_file,
};
use forge_pilot::{
    observe_scope, score_targets, CanonicalCaseClass, LawfulStepKind, LoopRunner, PilotHistory,
    TargetKind,
};
use knowledge_runtime::Scope;

#[tokio::test]
async fn target_taxonomy_normalizes_to_canonical_case_classes() {
    assert_eq!(
        TargetKind::ActiveSyndrome {
            signature: "sig".into()
        }
        .canonical_case_class(),
        CanonicalCaseClass::ContradictionInvestigation
    );
    assert_eq!(
        TargetKind::ThinExport {
            marker: "thin".into()
        }
        .canonical_case_class(),
        CanonicalCaseClass::ThinExportGap
    );
    assert_eq!(
        TargetKind::ScopeStale {
            last_import_at: None
        }
        .canonical_case_class(),
        CanonicalCaseClass::ScopeFreshness
    );
}

#[tokio::test]
async fn decision_audit_records_cheapest_and_blocked_steps_deterministically() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-normalization-audit");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn audit_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("audit-1"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let candidates = score_targets(&observation, &PilotHistory::default(), &config);
    let candidate = candidates.first().unwrap();
    let audit = forge_pilot::decide::build_decision_audit(candidate);

    assert!(audit.cheapest_admissible.is_some());
    assert!(audit
        .fallback_steps
        .iter()
        .all(|step| step.step_kind >= audit.cheapest_admissible.unwrap()));
    assert!(audit
        .blocked_steps
        .iter()
        .all(|blocked| !blocked.reason.is_empty()));
}

#[tokio::test]
async fn loop_iteration_report_carries_normalization_and_lineage_receipts() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-normalization-loop");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn loop_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("loop-normalized"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    let iteration = &report.iterations[0];

    assert!(iteration.target_normalization.is_some());
    assert!(iteration.decision_audit.is_some());
    assert!(iteration.lineage_receipt.is_some());
    assert!(iteration.export_trace.is_some());
    assert!(iteration.stop_rule_evaluation.is_some());
    assert!(matches!(
        iteration
            .decision_audit
            .as_ref()
            .unwrap()
            .cheapest_admissible,
        Some(
            LawfulStepKind::ContractSchemaCheck
                | LawfulStepKind::ProvenanceReceiptAudit
                | LawfulStepKind::TemporalConsistencyCheck
                | LawfulStepKind::ExactReplay
                | LawfulStepKind::PairedComparativeCheck
                | LawfulStepKind::ExactOracleSlice
                | LawfulStepKind::ConservativeOracleSlice
                | LawfulStepKind::MinimalPerturbationRefuter
                | LawfulStepKind::NuisanceComparabilityAudit
                | LawfulStepKind::HumanReviewRequest
                | LawfulStepKind::CanonicalExportRequest
                | LawfulStepKind::CanonicalImportRequest
        )
    ));
}
