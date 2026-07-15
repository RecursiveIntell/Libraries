#![allow(dead_code, deprecated)]

use forge_engine::export::EpisodeExport;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::{
    AssessmentCategory, CausalHypothesis, ClaimStrength, ContradictionState, EvidenceAssessment,
    ExperimentEvidenceBundle, ForgeConfig, ForgeStore, HypothesisStatus, SampleSupport,
};
use forge_memory_bridge::{transform_envelope_v2, transform_envelope_v3};
use forge_pilot::{bootstrap, LoopConfig, LoopRunnerResources};
use knowledge_runtime::config::{EntityConfig, ProjectionConfig, QueryConfig};
use knowledge_runtime::{RuntimeConfig, Scope};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use stack_ids::{
    AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub fn open_memory_store(base_dir: &Path) -> MemoryStore {
    let config = MemoryConfig {
        base_dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).unwrap()
}

pub fn open_forge_store(base_dir: &Path) -> ForgeStore {
    fs::create_dir_all(base_dir).unwrap();
    ForgeStore::open(&base_dir.join("forge.db")).unwrap()
}

pub async fn latest_import_batch(
    memory_store: &MemoryStore,
    namespace: &str,
) -> semantic_memory::ProjectionImportLogEntry {
    memory_store
        .query_projection_imports(Some(namespace), 10)
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.status == "complete")
        .unwrap()
}

pub async fn latest_bootstrap_manifest(
    memory_store: &MemoryStore,
    namespace: &str,
) -> forge_pilot::BootstrapManifestSnapshot {
    let log = latest_import_batch(memory_store, namespace).await;
    let batch = log.rebuildable_kernel_batch_v3().unwrap().unwrap();
    bootstrap::manifest_from_batch(&batch).unwrap()
}

pub fn sample_bundle(bundle_id: &str) -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.into(),
        candidate_id: format!("candidate-{bundle_id}"),
        eval_id: format!("eval-{bundle_id}"),
        version_id: "v0001".into(),
        supersedes_claim_version_id: Some(ClaimVersionId::new(format!("previous-{bundle_id}"))),
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 0.9,
            novelty: 0.2,
            stability: 0.7,
            weighted_total: 0.8,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis-{bundle_id}"),
            cause_signature: "baseline".into(),
            effect_signature: "evidence".into(),
            confidence: 0.6,
            status: HypothesisStatus::Proposed,
            support_count: 1,
            contradiction_count: 0,
        }],
        verification: None,
        trace_id: Some(format!("trace-{bundle_id}")),
        experiment_diff: None,
        attribution_json: None,
        assessment: Some(EvidenceAssessment {
            reproducibility: AssessmentCategory::Adequate,
            isolation: AssessmentCategory::Adequate,
            contradiction_state: ContradictionState::Clean,
            sample_support: SampleSupport::Marginal,
        }),
        warnings: vec!["test fixture".into()],
        created_at: "2026-03-11T00:00:00Z".into(),
        run_id: Some(format!("run-{bundle_id}")),
        attempt_id: Some(format!("attempt-{bundle_id}")),
        causal_question: Some("Does this fixture remain queryable?".into()),
        unit_definition: Some("test bundle".into()),
        bundle_scope: None,
        pair_comparability: None,
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: None,
        known_threats: vec![],
        patch_hash: None,
        treatment: None,
        outcome: Some("verified".into()),
        covariates: None,
        promotion_state: None,
        primary_effect: None,
        all_effects: vec![],
        hypothesis_edges: vec![],
        receipts: vec![],
        verification_trials: vec![],
        refutation_artifacts: vec![],
        sealed: false,
    }
}

pub fn persist_bundle_in_forge(store: &ForgeStore, bundle: &ExperimentEvidenceBundle) {
    let warnings_json = serde_json::to_string(&bundle.warnings).unwrap();
    let scores_json = serde_json::to_string(&bundle.scores).unwrap();
    let hypotheses_json = serde_json::to_string(&bundle.hypotheses).unwrap();
    let verification_plan_json = bundle
        .verification
        .as_ref()
        .map(|value| serde_json::to_string(value).unwrap());
    let diff_json = bundle
        .experiment_diff
        .as_ref()
        .map(|value| serde_json::to_string(value).unwrap());
    let assessment_json = bundle
        .assessment
        .as_ref()
        .map(|value| serde_json::to_string(value).unwrap());

    store
        .insert_evidence_bundle(
            &bundle.bundle_id,
            &bundle.candidate_id,
            &bundle.eval_id,
            &bundle.version_id,
            bundle.trace_id.as_deref().unwrap_or("trace:test"),
            &scores_json,
            &hypotheses_json,
            verification_plan_json.as_deref(),
            diff_json.as_deref(),
            assessment_json.as_deref(),
            &warnings_json,
        )
        .unwrap();
}

pub fn promoted_bundle(bundle_id: &str) -> ExperimentEvidenceBundle {
    let mut bundle = sample_bundle(bundle_id);
    bundle.promotion_state = Some(semantic_memory_forge::PromotionState::Promoted {
        version_id: Some(format!("version-{bundle_id}")),
        promoted_at: Some("2026-03-11T00:00:00Z".into()),
    });
    bundle
}

pub async fn import_v3_bundle(
    memory_store: &MemoryStore,
    forge_store: &ForgeStore,
    namespace: &str,
    bundle: &ExperimentEvidenceBundle,
) {
    let envelope = forge_engine::export_bundle(bundle, namespace, forge_store)
        .await
        .unwrap();
    let batch = transform_envelope_v3(&envelope).unwrap();
    memory_store.import_projection_batch(&batch).await.unwrap();
}

pub async fn import_v2_bundle_without_kernel_payload(
    memory_store: &MemoryStore,
    namespace: &str,
    bundle: &ExperimentEvidenceBundle,
) {
    let export = EpisodeExport::from_bundle(bundle, namespace);
    #[allow(deprecated)]
    let envelope = export.to_export_envelope_v2(bundle).unwrap();
    let batch = transform_envelope_v2(&envelope).unwrap();
    memory_store.import_projection_batch(&batch).await.unwrap();
}

pub async fn import_thin_v3_batch(
    memory_store: &MemoryStore,
    forge_store: &ForgeStore,
    namespace: &str,
    bundle: &ExperimentEvidenceBundle,
) {
    let envelope = forge_engine::export_bundle(bundle, namespace, forge_store)
        .await
        .unwrap();
    let mut batch = transform_envelope_v3(&envelope).unwrap();
    for record in &mut batch.records {
        record.semantics = None;
    }
    memory_store.import_projection_batch(&batch).await.unwrap();
}

pub async fn import_promoted_hyperedge_batch(
    memory_store: &MemoryStore,
    namespace: &str,
    batch_id: &str,
) {
    use semantic_memory_forge::{
        ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV3,
        ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
        ProjectionVisibilityClass, EXPORT_ENVELOPE_V3_SCHEMA,
    };

    let scope = ScopeKey::namespace_only(namespace);
    let records = vec![
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: Some(ClaimId::new(format!("{batch_id}-claim-1"))),
                claim_version_id: Some(ClaimVersionId::new(format!("{batch_id}-claim-1"))),
                subject_entity_id: EntityId::new(format!("{batch_id}-entity-1")),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: "entity supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: Some(serde_json::json!({
                    "verification_summary": {
                        "lifecycle_state": "verified",
                        "promotion_state": {
                            "state": "promoted",
                            "version_id": format!("version-{batch_id}"),
                            "promoted_at": "2026-03-11T00:00:00Z"
                        },
                        "completed_trial_count": 1,
                        "passed_refutation_count": 0,
                        "failed_refutation_count": 0,
                        "notes": ["promoted fixture"]
                    },
                    "promotion_state": {
                        "state": "promoted",
                        "version_id": format!("version-{batch_id}"),
                        "promoted_at": "2026-03-11T00:00:00Z"
                    }
                })),
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new(format!("claim-family:{batch_id}"))),
                assertion_group_id: Some(AssertionGroupId::new(format!("assertion-group:{batch_id}"))),
                relation_group_id: None,
                joint_evidence_group_id: None,
                constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                treatment_hint: None,
                outcome_hint: None,
                confounder_hint: None,
                instrument_hint: None,
                effect_modifier_hint: None,
                contradiction_candidate_group_id: None,
                mutual_exclusion_group_id: None,
                comparability_snapshot_version: None,
                nuisance_snapshot: None,
                projection_visibility_class: ProjectionVisibilityClass::Standard,
                export_confidence_class: ExportConfidenceClass::Verified,
                derivation_seed_ids: vec![],
                review_priority_hint: None,
            }),
        },
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: Some(ClaimId::new(format!("{batch_id}-claim-2"))),
                claim_version_id: Some(ClaimVersionId::new(format!("{batch_id}-claim-2"))),
                subject_entity_id: EntityId::new(format!("{batch_id}-entity-2")),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 0.95,
                content: "peer entity supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new(format!("claim-family:{batch_id}"))),
                assertion_group_id: Some(AssertionGroupId::new(format!("assertion-group:{batch_id}"))),
                relation_group_id: None,
                joint_evidence_group_id: None,
                constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                treatment_hint: None,
                outcome_hint: None,
                confounder_hint: None,
                instrument_hint: None,
                effect_modifier_hint: None,
                contradiction_candidate_group_id: None,
                mutual_exclusion_group_id: None,
                comparability_snapshot_version: None,
                nuisance_snapshot: None,
                projection_visibility_class: ProjectionVisibilityClass::Standard,
                export_confidence_class: ExportConfidenceClass::Verified,
                derivation_seed_ids: vec![],
                review_priority_hint: None,
            }),
        },
    ];

    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some(format!("run:{batch_id}")),
        direct_write: false,
        comparability_snapshot_version: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
    };
    let digest =
        ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .unwrap();
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(format!("envelope:{batch_id}")),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
        export_meta: Some(export_meta),
        evidence_bundle: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records,
    };
    let batch = transform_envelope_v3(&envelope).unwrap();
    memory_store.import_projection_batch(&batch).await.unwrap();
}

pub fn base_loop_config(scope: Scope) -> LoopConfig {
    let mut config = LoopConfig::default_for_scope(scope.clone(), ".");
    config.memory_dir = "./memory".into();
    config.forge_db_path = "./forge.db".into();
    config.runtime_config = RuntimeConfig {
        default_scope: scope,
        query: QueryConfig::default(),
        entity: EntityConfig::default(),
        projection: ProjectionConfig::default(),
        strict_temporal: false,
        strict_scope: false,
    };
    config.forge_config = ForgeConfig {
        execution_backend_preference: "host".into(),
        ..ForgeConfig::default()
    };
    // Tests need a permissive policy to exercise execution paths.
    // Production default is deny (AUTH-001 fix); tests explicitly opt in.
    use verification_policy::PolicySnapshot;
    config.policy_snapshots = vec![PolicySnapshot::permissive(
        "forge-pilot.test",
        "2026-03-12T00:00:00Z",
    )];
    config
}

pub fn point_config_at_dir(config: &mut LoopConfig, base_dir: &Path) {
    config.workspace_path = base_dir.to_string_lossy().to_string();
    config.memory_dir = base_dir.join("memory").to_string_lossy().to_string();
    config.forge_db_path = base_dir.join("forge.db").to_string_lossy().to_string();
}

pub fn write_source_file(base_dir: &Path, relative_path: &str, body: &str) {
    let path = base_dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

pub fn resources(
    memory_store: MemoryStore,
    forge_store: ForgeStore,
    config: &LoopConfig,
) -> LoopRunnerResources {
    LoopRunnerResources::from_memory_store(memory_store, forge_store, config.runtime_config.clone())
        .unwrap()
}

pub fn write_patch_fixture(base_dir: &Path) -> (PathBuf, forge_engine::StructuredPatch) {
    let fixture_dir = base_dir.join("patch-fixture");
    fs::create_dir_all(fixture_dir.join("src")).unwrap();
    fs::write(
        fixture_dir.join("Cargo.toml"),
        r#"[package]
name = "patch-fixture"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(
        fixture_dir.join("src/lib.rs"),
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn adds_numbers() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
    )
    .unwrap();

    let patch = forge_engine::StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "fix arithmetic".into(),
        edits: vec![forge_engine::FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![forge_engine::EditOp::Replace {
                range: forge_engine::LineRange {
                    start: 2,
                    end_exclusive: 3,
                },
                lines: vec!["    a + b".into()],
            }],
            mode: Some(forge_engine::FileMode::Modify),
        }],
        notes: vec!["deterministic fixture patch".into()],
    };

    (fixture_dir, patch)
}

pub fn first_import_scope_query(namespace: &str) -> semantic_memory::ProjectionQuery {
    semantic_memory::ProjectionQuery::new(ScopeKey::namespace_only(namespace))
}

pub fn tempdir() -> TempDir {
    TempDir::new().unwrap()
}
