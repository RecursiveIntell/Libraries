#![allow(clippy::expect_used)]
#![allow(deprecated)]

//! Tests for the Forge export seam: deterministic keys, receipts, and canonical envelopes.

use forge_engine::experiment::{EffectKind, TypedLocatedEffect};
use forge_engine::export::{compute_export_key, EpisodeExport, RENDERING_VERSION};
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::{
    BaselineOrPatch, BundleScope, Covariates, EffectRelationLineageHint,
    EffectRelationLineageSource, HypothesisRelationLineageHint, PairComparability,
    RefutationArtifact, RefutationArtifactOutcome, RefutationArtifactType,
    RefutationRelationLineageHint, RelationLineageHints, Treatment, VerificationTrial,
    VerificationTrialRelationLineageHint,
};
use forge_engine::{
    AssessmentCategory, CausalHypothesis, ClaimStrength, ContradictionState, EvidenceAssessment,
    ExperimentDiff, ExperimentEvidenceBundle, ForgeStore, HypothesisEdge, HypothesisEdgeKind,
    HypothesisStatus, ReceiptKind, ReceiptRef, ReceiptStorage, SampleSupport, VerificationState,
};
use forge_memory_bridge::transform_envelope_v3;
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder, ProjectionQuery};
use semantic_memory_forge::{
    ExportRecord, EXPORT_ENVELOPE_V1_SCHEMA, EXPORT_ENVELOPE_V2_SCHEMA, EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{AttemptId, ClaimVersionId, RelationVersionId, ScopeKey, TrialId};
use tempfile::TempDir;

fn open_test_memory_store(base_dir: &std::path::Path) -> MemoryStore {
    let config = MemoryConfig {
        base_dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).unwrap()
}

fn test_bundle(bundle_id: &str) -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.into(),
        candidate_id: "c-export".into(),
        eval_id: "e-export".into(),
        version_id: "v0001".into(),
        scores: ScoreVector {
            correctness: 0.9,
            novelty: 0.2,
            stability: 0.6,
            weighted_total: 0.7,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: "h-1".into(),
            cause_signature: "cause".into(),
            effect_signature: "effect".into(),
            confidence: 0.5,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        verification: None,
        trace_id: Some("trace-export".into()),
        experiment_diff: None,
        attribution_json: None,
        assessment: None,
        warnings: vec!["threat:sample-size".into()],
        created_at: "2026-03-07T00:00:00Z".into(),
        run_id: Some("run-export".into()),
        attempt_id: Some("attempt-export".into()),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        causal_question: None,
        unit_definition: None,
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
        primary_effect: Some(TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: None,
            line: None,
            message: "test regression detected".into(),
            in_baseline: false,
            in_patched: true,
        }),
        all_effects: vec![
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: None,
                line: None,
                message: "test regression detected".into(),
                in_baseline: false,
                in_patched: true,
            },
            TypedLocatedEffect {
                kind: EffectKind::PerformanceImprovement,
                file: Some(std::path::PathBuf::from("src/lib.rs")),
                line: Some(42),
                message: "perf improved on benchmark".into(),
                in_baseline: true,
                in_patched: false,
            },
        ],
        hypothesis_edges: vec![],
        receipts: vec![],
        verification_trials: vec![],
        refutation_artifacts: vec![],
        sealed: false,
    }
}

#[test]
fn claim_record_carries_supersedes_claim_version_id_when_present() {
    let mut bundle = test_bundle("b-supersede");
    let previous_version = ClaimVersionId::new("cv-prev-001");
    bundle.supersedes_claim_version_id = Some(previous_version.clone());

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let claim = envelope
        .records
        .iter()
        .find_map(|record| {
            if let ExportRecord::Claim(claim) = record {
                Some(claim)
            } else {
                None
            }
        })
        .expect("claim record should be present");

    assert_eq!(
        claim.supersedes_claim_version_id,
        Some(previous_version),
        "living-memory export should preserve real prior claim version lineage"
    );
}

#[test]
fn claim_record_does_not_synthesize_claim_lineage() {
    let mut bundle = test_bundle("b-no-supersede");
    bundle.supersedes_claim_version_id = None;

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let claim = envelope
        .records
        .iter()
        .find_map(|record| {
            if let ExportRecord::Claim(claim) = record {
                Some(claim)
            } else {
                None
            }
        })
        .expect("claim record should be present");

    assert!(
        claim.supersedes_claim_version_id.is_none(),
        "lineage should remain None when no real prior claim version is known"
    );
}

#[test]
fn claim_and_relation_records_carry_stable_version_ids_and_bundle_valid_time() {
    let bundle = test_bundle("b-versioned");

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope_a = export.to_export_envelope_v1(&bundle).unwrap();
    let envelope_b = export.to_export_envelope_v1(&bundle).unwrap();

    let claim_a = envelope_a
        .records
        .iter()
        .find_map(|record| match record {
            ExportRecord::Claim(claim) => Some(claim),
            _ => None,
        })
        .expect("claim record should be present");
    let claim_b = envelope_b
        .records
        .iter()
        .find_map(|record| match record {
            ExportRecord::Claim(claim) => Some(claim),
            _ => None,
        })
        .expect("claim record should be present");

    assert!(
        claim_a.claim_version_id.is_some(),
        "export should carry a stable claim version id"
    );
    assert_eq!(claim_a.claim_version_id, claim_b.claim_version_id);
    assert_eq!(
        claim_a.valid_from.as_deref(),
        Some(bundle.created_at.as_str()),
        "claim valid_from should come from bundle recorded time"
    );

    let relations_a: Vec<_> = envelope_a
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::Relation(relation) => Some(relation),
            _ => None,
        })
        .collect();
    let relations_b: Vec<_> = envelope_b
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::Relation(relation) => Some(relation),
            _ => None,
        })
        .collect();

    assert!(
        !relations_a.is_empty(),
        "bundle should emit export relations"
    );
    for (left, right) in relations_a.iter().zip(relations_b.iter()) {
        assert!(
            left.relation_version_id.is_some(),
            "export relations should carry stable relation version ids"
        );
        assert_eq!(left.relation_version_id, right.relation_version_id);
        assert_eq!(left.valid_from, Some(bundle.created_at.clone()));
    }
}

#[test]
fn evidence_refs_bind_to_exported_claim_version() {
    let bundle = test_bundle("b-evidence-refs");
    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let claim = envelope
        .records
        .iter()
        .find_map(|record| match record {
            ExportRecord::Claim(claim) => Some(claim),
            _ => None,
        })
        .expect("claim record should be present");
    let claim_id = claim
        .claim_id
        .as_ref()
        .expect("claim id should be present")
        .clone();
    let claim_version_id = claim
        .claim_version_id
        .as_ref()
        .expect("claim version id should be present")
        .clone();

    let evidence_refs: Vec<_> = envelope
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::EvidenceRef(evidence) => Some(evidence),
            _ => None,
        })
        .collect();

    assert!(
        !evidence_refs.is_empty(),
        "bundle should emit evidence refs"
    );
    for evidence_ref in evidence_refs {
        assert_eq!(evidence_ref.claim_id, claim_id);
        assert_eq!(
            evidence_ref.claim_version_id,
            Some(claim_version_id.clone()),
            "evidence refs should bind to the exported claim version"
        );
    }
}

#[test]
fn relation_records_carry_supersedes_relation_version_id_when_hinted() {
    let mut bundle = test_bundle("b-relation-lineage");
    let primary_effect = bundle.primary_effect.clone().expect("primary effect");
    let all_effect = bundle
        .all_effects
        .iter()
        .find(|effect| effect.message == "perf improved on benchmark")
        .cloned()
        .expect("distinct all-effect relation");
    let diff_effect = TypedLocatedEffect {
        kind: EffectKind::Timeout,
        file: Some(std::path::PathBuf::from("src/bin/cli.rs")),
        line: Some(77),
        message: "verification timeout surfaced".into(),
        in_baseline: false,
        in_patched: true,
    };
    let trial_id = TrialId::new("trial-lineage");
    let attempt_id = AttemptId::new("attempt-lineage");

    bundle.hypothesis_edges.push(HypothesisEdge {
        edge_id: "edge-lineage".into(),
        source_edit: "edit:cli-timeout".into(),
        target_effect: "effect:timeout".into(),
        kind: HypothesisEdgeKind::CausesRegression,
        status: HypothesisStatus::Supported,
        confidence: 0.91,
        evidence_ids: vec!["receipt-1".into()],
        contradiction_ids: vec![],
        verification_status: VerificationState::PlanGenerated,
    });
    bundle.verification_trials.push(VerificationTrial {
        trial_id: trial_id.clone(),
        attempt_id: attempt_id.clone(),
        baseline_or_patch: BaselineOrPatch::Baseline,
        completed: true,
        receipts: vec!["trial-log".into()],
    });
    bundle.refutation_artifacts.push(RefutationArtifact {
        artifact_id: "artifact-lineage".into(),
        artifact_type: RefutationArtifactType::Placebo,
        trial_id: Some(trial_id.clone()),
        attempt_id: Some(attempt_id.clone()),
        outcome: RefutationArtifactOutcome::Passed,
        estimate_delta: Some(0.0),
        details: Some("placebo held steady".into()),
    });
    bundle.experiment_diff = Some(ExperimentDiff {
        effects: vec![diff_effect.clone()],
        regressions: 1,
        improvements: 0,
        stable_failures: 0,
        stable_passes: 0,
        statistically_meaningful: true,
        sample_warning: None,
    });
    bundle.relation_lineage_hints = RelationLineageHints {
        effect_relations: vec![
            EffectRelationLineageHint {
                source: EffectRelationLineageSource::PrimaryEffect,
                kind: primary_effect.kind.clone(),
                file: primary_effect.file.clone(),
                line: primary_effect.line,
                message: primary_effect.message.clone(),
                in_baseline: primary_effect.in_baseline,
                in_patched: primary_effect.in_patched,
                supersedes_relation_version_id: RelationVersionId::new("rel-ver-primary"),
            },
            EffectRelationLineageHint {
                source: EffectRelationLineageSource::AllEffect,
                kind: all_effect.kind.clone(),
                file: all_effect.file.clone(),
                line: all_effect.line,
                message: all_effect.message.clone(),
                in_baseline: all_effect.in_baseline,
                in_patched: all_effect.in_patched,
                supersedes_relation_version_id: RelationVersionId::new("rel-ver-all"),
            },
            EffectRelationLineageHint {
                source: EffectRelationLineageSource::ExperimentDiff,
                kind: diff_effect.kind.clone(),
                file: diff_effect.file.clone(),
                line: diff_effect.line,
                message: diff_effect.message.clone(),
                in_baseline: diff_effect.in_baseline,
                in_patched: diff_effect.in_patched,
                supersedes_relation_version_id: RelationVersionId::new("rel-ver-diff"),
            },
        ],
        hypothesis_relations: vec![HypothesisRelationLineageHint {
            edge_id: "edge-lineage".into(),
            supersedes_relation_version_id: RelationVersionId::new("rel-ver-hypothesis"),
        }],
        verification_trial_relations: vec![VerificationTrialRelationLineageHint {
            trial_id: trial_id.clone(),
            baseline_or_patch: BaselineOrPatch::Baseline,
            attempt_id: Some(attempt_id.clone()),
            supersedes_relation_version_id: RelationVersionId::new("rel-ver-trial"),
        }],
        refutation_relations: vec![RefutationRelationLineageHint {
            artifact_id: "artifact-lineage".into(),
            artifact_type: Some(RefutationArtifactType::Placebo),
            supersedes_relation_version_id: RelationVersionId::new("rel-ver-refutation"),
        }],
    };

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();
    let relations: Vec<_> = envelope
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::Relation(relation) => Some(relation),
            _ => None,
        })
        .collect();

    let primary_relation = relations
        .iter()
        .find(|relation| relation.predicate == "primary_effect_test_failure")
        .expect("primary relation");
    assert_eq!(
        primary_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-primary"))
    );

    let all_effect_relation = relations
        .iter()
        .find(|relation| relation.predicate == "all_effect_performance_improvement")
        .expect("all-effect relation");
    assert_eq!(
        all_effect_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-all"))
    );

    let diff_relation = relations
        .iter()
        .find(|relation| relation.predicate == "experiment_diff_timeout")
        .expect("experiment-diff relation");
    assert_eq!(
        diff_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-diff"))
    );

    let hypothesis_relation = relations
        .iter()
        .find(|relation| relation.object_anchor["edge_id"].as_str() == Some("edge-lineage"))
        .expect("hypothesis relation");
    assert_eq!(
        hypothesis_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-hypothesis"))
    );

    let verification_trial_relation = relations
        .iter()
        .find(|relation| relation.object_anchor["trial_id"].as_str() == Some("trial:trial-lineage"))
        .expect("verification trial relation");
    assert_eq!(
        verification_trial_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-trial"))
    );

    let refutation_relation = relations
        .iter()
        .find(|relation| relation.object_anchor["artifact_id"].as_str() == Some("artifact-lineage"))
        .expect("refutation relation");
    assert_eq!(
        refutation_relation.supersedes_relation_version_id,
        Some(RelationVersionId::new("rel-ver-refutation"))
    );
}

#[test]
fn relation_records_do_not_synthesize_relation_lineage_when_unknown() {
    let mut bundle = test_bundle("b-relation-lineage-absent");
    bundle.hypothesis_edges.push(HypothesisEdge {
        edge_id: "edge-no-lineage".into(),
        source_edit: "edit:latent".into(),
        target_effect: "effect:regression".into(),
        kind: HypothesisEdgeKind::AssociatedWithStableFailure,
        status: HypothesisStatus::Proposed,
        confidence: 0.44,
        evidence_ids: vec![],
        contradiction_ids: vec![],
        verification_status: VerificationState::Unverified,
    });
    bundle.verification_trials.push(VerificationTrial {
        trial_id: TrialId::new("trial-no-lineage"),
        attempt_id: AttemptId::new("attempt-no-lineage"),
        baseline_or_patch: BaselineOrPatch::Patched,
        completed: false,
        receipts: vec![],
    });
    bundle.refutation_artifacts.push(RefutationArtifact {
        artifact_id: "artifact-no-lineage".into(),
        artifact_type: RefutationArtifactType::DummyOutcome,
        trial_id: Some(TrialId::new("trial-no-lineage")),
        attempt_id: Some(AttemptId::new("attempt-no-lineage")),
        outcome: RefutationArtifactOutcome::Inconclusive {
            reason: "timed out before completing".into(),
        },
        estimate_delta: None,
        details: None,
    });
    bundle.experiment_diff = Some(ExperimentDiff {
        effects: vec![TypedLocatedEffect {
            kind: EffectKind::WarningRegression,
            file: Some(std::path::PathBuf::from("src/lib.rs")),
            line: Some(19),
            message: "warning regression persisted".into(),
            in_baseline: false,
            in_patched: true,
        }],
        regressions: 1,
        improvements: 0,
        stable_failures: 0,
        stable_passes: 0,
        statistically_meaningful: false,
        sample_warning: Some("single-trial only".into()),
    });

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();
    let relations: Vec<_> = envelope
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::Relation(relation) => Some(relation),
            _ => None,
        })
        .collect();

    assert!(
        !relations.is_empty(),
        "rich bundles should still export relation rows"
    );
    assert!(
        relations
            .iter()
            .all(|relation| relation.supersedes_relation_version_id.is_none()),
        "export must not mint fake relation lineage when no hints are present"
    );
}

#[test]
fn export_key_is_deterministic() {
    let k1 = compute_export_key("b-001", 1, "default");
    let k2 = compute_export_key("b-001", 1, "default");
    assert_eq!(k1, k2);
    assert_eq!(k1.len(), 64, "blake3 hex should be 64 chars");
}

#[test]
fn export_key_varies_with_inputs() {
    let k1 = compute_export_key("b-001", 1, "default");
    let k2 = compute_export_key("b-002", 1, "default");
    let k3 = compute_export_key("b-001", 2, "default");
    let k4 = compute_export_key("b-001", 1, "other_ns");

    assert_ne!(k1, k2, "different bundle_id must produce different key");
    assert_ne!(
        k1, k3,
        "different rendering_version must produce different key"
    );
    assert_ne!(k1, k4, "different namespace must produce different key");
}

#[test]
fn episode_export_from_bundle_populates_all_fields() {
    let bundle = test_bundle("b-export-1");
    let export = EpisodeExport::from_bundle(&bundle, "test-ns");

    assert_eq!(export.bundle_id, "b-export-1");
    assert_eq!(export.rendering_version, RENDERING_VERSION);
    assert_eq!(export.namespace, "test-ns");
    assert!(!export.export_key.is_empty());
    assert!(!export.content.is_empty());
    assert_eq!(export.meta["type"], "forge_evidence");
    assert_eq!(export.meta["bundle_id"], "b-export-1");
    // Phase 5: metadata includes claim_strength
    assert!(export.meta["claim_strength"].as_str().is_some());
}

#[test]
fn export_receipt_idempotent() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let bundle = test_bundle("b-idem");
    let export = EpisodeExport::from_bundle(&bundle, "default");

    // First insert should succeed
    let first = export.persist_receipt(&store, None).unwrap();
    assert!(first, "first insert should return true");

    // Second insert with same key should be ignored (INSERT OR IGNORE)
    let second = export.persist_receipt(&store, None).unwrap();
    assert!(!second, "duplicate insert should return false");
}

#[test]
fn already_exported_check() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let bundle = test_bundle("b-check");
    let export = EpisodeExport::from_bundle(&bundle, "default");

    assert!(!export.already_exported(&store).unwrap());

    export.persist_receipt(&store, Some(true)).unwrap();

    assert!(export.already_exported(&store).unwrap());
}

#[tokio::test]
async fn export_bundle_idempotent_skip() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let bundle = test_bundle("b-async");

    // First export
    let export1 = forge_engine::export_bundle(&bundle, "default", &store)
        .await
        .unwrap();

    // Second export should short-circuit (already receipted)
    let export2 = forge_engine::export_bundle(&bundle, "default", &store)
        .await
        .unwrap();

    assert_eq!(export1.envelope_id, export2.envelope_id);
    assert_eq!(export1.content_digest, export2.content_digest);
}

#[test]
fn export_receipt_with_write_through_status() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let bundle = test_bundle("b-wt");
    let export = EpisodeExport::from_bundle(&bundle, "default");

    // Persist with write_through_ok = Some(true)
    export.persist_receipt(&store, Some(true)).unwrap();
    assert!(export.already_exported(&store).unwrap());

    // Different bundle, write_through_ok = Some(false)
    let bundle2 = test_bundle("b-wt-fail");
    let export2 = EpisodeExport::from_bundle(&bundle2, "default");
    export2.persist_receipt(&store, Some(false)).unwrap();
    assert!(export2.already_exported(&store).unwrap());
}

#[test]
fn rendering_version_is_three() {
    assert_eq!(RENDERING_VERSION, 3);
}

#[tokio::test]
async fn export_bundle_returns_canonical_v3_envelope() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let bundle = test_bundle("b-canonical-v3");
    let envelope = forge_engine::export_bundle(&bundle, "default", &store)
        .await
        .unwrap();

    assert_eq!(envelope.schema_version, EXPORT_ENVELOPE_V3_SCHEMA);
    assert_eq!(envelope.scope_key.namespace, "default");
    assert_eq!(
        envelope
            .export_meta
            .as_ref()
            .and_then(|meta| meta.run_id.as_deref()),
        Some("run-export")
    );
    let evidence_bundle = envelope
        .evidence_bundle
        .as_ref()
        .expect("canonical v3 should carry the canonical evidence bundle");
    let mut expected_bundle = bundle.to_canonical_evidence_bundle();
    expected_bundle.claim_ids = evidence_bundle.claim_ids.clone();
    expected_bundle.created_at = evidence_bundle.created_at.clone();
    assert_eq!(
        serde_json::to_value(evidence_bundle).unwrap(),
        serde_json::to_value(expected_bundle).unwrap(),
        "canonical v3 export should carry the same canonical evidence bundle payload as the forge adapter"
    );
    assert!(
        envelope
            .records
            .iter()
            .any(|record| record.semantics.is_some()),
        "canonical v3 should carry semantics for records that can be enriched"
    );
}

#[tokio::test]
async fn to_export_envelope_v3_survives_bridge_transform_and_memory_import() {
    let dir = TempDir::new().unwrap();
    let memory = open_test_memory_store(dir.path());
    let mut bundle = test_bundle("b-v3-survive");
    bundle.supersedes_claim_version_id = Some(ClaimVersionId::new("c-old-v3"));

    let export = EpisodeExport::from_bundle(&bundle, "v3-survive-ns");
    let envelope = export.to_export_envelope_v3(&bundle).unwrap();
    let batch = transform_envelope_v3(&envelope).unwrap();
    let expected_evidence_bundle = envelope
        .evidence_bundle
        .as_ref()
        .expect("canonical V3 export should carry the canonical evidence bundle")
        .clone();

    let expected_claim_version = envelope
        .records
        .iter()
        .find_map(|record| match &record.record {
            semantic_memory_forge::ExportRecord::Claim(claim) => claim.claim_version_id.clone(),
            _ => None,
        })
        .expect("claim record should be present");
    let imported = memory.import_projection_batch(&batch).await.unwrap();

    assert_eq!(imported.status, "complete");

    let claims = memory
        .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
            "v3-survive-ns",
        )))
        .await
        .unwrap();

    assert!(
        claims
            .iter()
            .any(|claim| claim.claim_version_id == expected_claim_version),
        "imported claim versions must be queryable"
    );

    let imported_claim = claims
        .iter()
        .find(|claim| claim.claim_version_id == expected_claim_version)
        .expect("expected imported claim");

    assert!(
        imported_claim.supersedes_claim_version_id.as_ref()
            == Some(&ClaimVersionId::new("c-old-v3")),
        "supersedes lineage should not be synthesized or lost"
    );
    assert!(
        imported_claim
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("kernel_semantics_v3"))
            .is_some(),
        "canonical V3 import should keep kernel semantics explicit in stored metadata"
    );

    let mut had_claim_semantics = false;
    for record in &envelope.records {
        if let semantic_memory_forge::ExportRecord::Claim(_) = &record.record {
            had_claim_semantics = record
                .semantics
                .as_ref()
                .and_then(|semantics| {
                    semantics.derivation_seed_ids.iter().find(|seed| {
                        seed.as_str() == "supersedes_claim_version:claim-version:c-old-v3"
                    })
                })
                .is_some();
        }
    }
    assert!(
        had_claim_semantics,
        "claim semantics should carry explicit supersedes-derived seed"
    );

    let imports = memory
        .query_projection_imports(Some("v3-survive-ns"), 10)
        .await
        .unwrap();
    let import = imports.first().expect("expected projection import receipt");
    assert_eq!(
        import.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );
    assert_eq!(
        import.evidence_bundle_id.as_deref(),
        Some(expected_evidence_bundle.id.as_str())
    );
    assert_eq!(
        import
            .evidence_bundle_json
            .as_ref()
            .and_then(|bundle| bundle.get("id"))
            .and_then(serde_json::Value::as_str),
        Some(expected_evidence_bundle.id.as_str())
    );
    assert!(
        import.kernel_payload_json.is_some(),
        "canonical V3 import receipts should remain rebuildable"
    );
    assert_eq!(
        batch
            .evidence_bundle
            .as_ref()
            .map(|bundle| bundle.id.as_str()),
        Some(expected_evidence_bundle.id.as_str())
    );
}

#[test]
fn claim_semantics_intentionally_group_without_constraint_seed_kind() {
    let bundle = test_bundle("b-claim-semantics");
    let export = EpisodeExport::from_bundle(&bundle, "claim-semantics-ns");
    let envelope = export.to_export_envelope_v3(&bundle).unwrap();

    let claim_semantics = envelope
        .records
        .iter()
        .find_map(|record| match &record.record {
            ExportRecord::Claim(_) => record.semantics.as_ref(),
            _ => None,
        })
        .expect("claim semantics should be present");

    assert_eq!(claim_semantics.constraint_seed_kind, None);
    assert!(claim_semantics.assertion_group_id.is_some());
    assert!(claim_semantics.claim_family_id.is_some());
}

#[test]
fn episode_export_builds_canonical_forge_envelope() {
    let bundle = test_bundle("b-envelope");
    let export = EpisodeExport::from_bundle(&bundle, "canon-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    assert_eq!(
        envelope.envelope_id.as_str(),
        format!("envelope:{}", export.export_key)
    );
    assert_eq!(envelope.schema_version, EXPORT_ENVELOPE_V1_SCHEMA);
    assert_eq!(envelope.source_authority, "forge");
    assert_eq!(envelope.scope_key.namespace, "canon-ns");
    assert_eq!(
        envelope.trace_ctx.as_ref().unwrap().trace_id,
        "trace-export"
    );
    assert!(envelope.records.len() >= 5);

    let mut saw_claim = false;
    let mut saw_episode = false;
    let mut saw_relation = false;
    let mut saw_alias = false;
    let mut saw_evidence = false;

    let mut relation_count = 0usize;
    let mut evidence_handles = Vec::new();

    for record in &envelope.records {
        match record {
            ExportRecord::Claim(claim) => {
                saw_claim = true;
                assert_eq!(claim.predicate, "forge_evidence_bundle");
                assert_eq!(claim.projection_family, "forge_verification");
                assert_eq!(claim.content, export.content);
            }
            ExportRecord::Episode(episode) => {
                saw_episode = true;
                assert_eq!(episode.document_id, "run-export");
                assert_eq!(episode.effect_type, "test_failure");
                assert_eq!(episode.outcome, "verified");
                assert_eq!(episode.experiment_id.as_deref(), Some("run-export"));
                assert_eq!(episode.cause_ids.len(), 3);
                assert_eq!(
                    episode.metadata.as_ref().unwrap()["attempt_id"],
                    "attempt-export"
                );
            }
            ExportRecord::Relation(_) => {
                saw_relation = true;
                relation_count += 1;
            }
            ExportRecord::EntityAlias(_) => {
                saw_alias = true;
            }
            ExportRecord::EvidenceRef(evidence) => {
                saw_evidence = true;
                evidence_handles.push(evidence.fetch_handle.as_str());
            }
        }
    }

    assert!(saw_claim, "claim record should be present");
    assert!(saw_episode, "episode record should be present");
    assert!(saw_relation, "relation record(s) should be present");
    assert!(saw_alias, "alias record(s) should be present");
    assert!(saw_evidence, "evidence-ref record(s) should be present");
    assert!(
        relation_count >= 2,
        "primary and all effects should both emit relations"
    );
    assert!(evidence_handles.contains(&"forge:bundle:b-envelope"));
    assert!(evidence_handles.contains(&"forge:attempt:attempt-export"));
    assert!(evidence_handles.contains(&"forge:run:run-export"));
}

#[test]
fn episode_export_builds_canonical_forge_v2_envelope() {
    let bundle = test_bundle("b-envelope-v2");
    let export = EpisodeExport::from_bundle(&bundle, "canon-v2-ns");
    let envelope = export.to_export_envelope_v2(&bundle).unwrap();

    assert_eq!(envelope.schema_version, EXPORT_ENVELOPE_V2_SCHEMA);
    assert_eq!(envelope.scope_key.namespace, "canon-v2-ns");
    assert_eq!(
        envelope
            .export_meta
            .as_ref()
            .and_then(|meta| meta.run_id.as_deref()),
        Some("run-export")
    );
    assert_eq!(
        envelope
            .export_meta
            .as_ref()
            .and_then(|meta| meta.comparability_snapshot_version.as_deref()),
        None
    );

    let evidence_bundle = envelope
        .evidence_bundle
        .as_ref()
        .expect("v2 envelope should carry canonical evidence bundle");
    let mut expected_bundle = bundle.to_canonical_evidence_bundle();
    expected_bundle.claim_ids = evidence_bundle.claim_ids.clone();
    expected_bundle.created_at = evidence_bundle.created_at.clone();

    assert_eq!(evidence_bundle.id.as_str(), "b-envelope-v2");
    assert_eq!(evidence_bundle.claim_ids.len(), 1);
    assert_eq!(
        evidence_bundle
            .trace_ctx
            .as_ref()
            .map(|ctx| ctx.trace_id.as_str()),
        Some("trace-export")
    );
    assert_eq!(
        serde_json::to_value(evidence_bundle).unwrap(),
        serde_json::to_value(expected_bundle).unwrap(),
        "V2 export should reuse the canonical local->Forge evidence adapter and only layer claim binding on top"
    );
}

/// Caller-managed export and convenience helper produce identical episode payloads.
#[test]
fn caller_managed_and_helper_produce_identical_payloads() {
    let bundle = test_bundle("b-identical");

    // Caller-managed path: create EpisodeExport manually
    let caller_export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let caller_meta = bundle.to_episode_meta();
    let caller_content = bundle.to_episode_content();

    // The export helper produces the same content/meta
    assert_eq!(caller_export.meta, caller_meta);
    assert_eq!(caller_export.content, caller_content);
}

#[test]
fn export_with_hypothesis_edges_includes_relations() {
    let mut bundle = test_bundle("b-hypothesis-edge");
    bundle.all_effects.clear();
    bundle.hypothesis_edges.push(HypothesisEdge {
        edge_id: "edge-1".into(),
        source_edit: "edit:add_file".into(),
        target_effect: "effect:timeout".into(),
        kind: HypothesisEdgeKind::CausesRegression,
        status: HypothesisStatus::Proposed,
        confidence: 0.83,
        evidence_ids: vec!["r1".into(), "r2".into()],
        contradiction_ids: vec![],
        verification_status: VerificationState::Unverified,
    });

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let relation_preds: Vec<String> = envelope
        .records
        .iter()
        .filter_map(|record| {
            if let ExportRecord::Relation(rel) = record {
                Some(rel.predicate.clone())
            } else {
                None
            }
        })
        .collect();

    assert!(relation_preds
        .iter()
        .any(|pred| pred.starts_with("hypothesis_edge_")));
}

#[test]
fn export_without_attempt_or_run_keeps_single_evidence_ref() {
    let mut bundle = test_bundle("b-no-identifiers");
    bundle.run_id = None;
    bundle.attempt_id = None;

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();
    let evidence_refs: Vec<_> = envelope
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::EvidenceRef(evidence) => Some(evidence.fetch_handle.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(evidence_refs, vec!["forge:bundle:b-no-identifiers"]);
}

#[test]
fn export_includes_receipt_evidence_refs() {
    let mut bundle = test_bundle("b-evidence-receipt");
    bundle.receipts.push(ReceiptRef {
        receipt_id: "r-001".into(),
        kind: ReceiptKind::TrialLog,
        storage: ReceiptStorage::StoreRow {
            table: "receipts".into(),
            key: "trial-001".into(),
        },
        content_hash: "abcd1234".into(),
        trace_id: Some("trace-receipt".into()),
        replay_handle: Some("replay:trial:001".into()),
    });

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();
    let receipt_refs: Vec<_> = envelope
        .records
        .iter()
        .filter_map(|record| match record {
            ExportRecord::EvidenceRef(evidence)
                if evidence.fetch_handle == "forge:receipt:r-001" =>
            {
                Some(evidence.fetch_handle.as_str())
            }
            _ => None,
        })
        .collect();

    assert_eq!(receipt_refs, vec!["forge:receipt:r-001"]);
}

#[test]
fn export_includes_experiment_diff_relations() {
    let mut bundle = test_bundle("b-experiment-diff");
    bundle.primary_effect = Some(TypedLocatedEffect {
        kind: EffectKind::TestFailure,
        file: Some("src/main.rs".into()),
        line: Some(120),
        message: "new failing test".into(),
        in_baseline: false,
        in_patched: true,
    });
    bundle.all_effects.clear();
    bundle.all_effects.push(TypedLocatedEffect {
        kind: EffectKind::PerformanceRegression,
        file: Some("src/lib.rs".into()),
        line: Some(45),
        message: "regression surfaced".into(),
        in_baseline: false,
        in_patched: true,
    });
    bundle.hypothesis_edges.push(HypothesisEdge {
        edge_id: "edge-2".into(),
        source_edit: "edit:main.rs:120".into(),
        target_effect: "failing test added".into(),
        kind: HypothesisEdgeKind::AssociatedWithStableFailure,
        status: HypothesisStatus::Supported,
        confidence: 0.88,
        evidence_ids: vec!["r2".into()],
        contradiction_ids: vec![],
        verification_status: VerificationState::Verified,
    });
    bundle.experiment_diff = Some(ExperimentDiff {
        effects: vec![
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: Some("src/main.rs".into()),
                line: Some(120),
                message: "new failing test".into(),
                in_baseline: false,
                in_patched: true,
            },
            TypedLocatedEffect {
                kind: EffectKind::PerformanceImprovement,
                file: Some("src/lib.rs".into()),
                line: Some(45),
                message: "regression improved".into(),
                in_baseline: true,
                in_patched: false,
            },
        ],
        regressions: 1,
        improvements: 1,
        stable_failures: 0,
        stable_passes: 0,
        statistically_meaningful: true,
        sample_warning: Some("sample_warning".into()),
    });

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let relation_predicates: Vec<String> = envelope
        .records
        .iter()
        .filter_map(|record| {
            if let ExportRecord::Relation(rel) = record {
                Some(rel.predicate.clone())
            } else {
                None
            }
        })
        .collect();

    assert!(
        relation_predicates
            .iter()
            .any(|pred| pred.starts_with("primary_effect_")),
        "primary_effect relations must be exported"
    );
    assert!(
        relation_predicates
            .iter()
            .any(|pred| pred.starts_with("all_effect_")),
        "all_effect relations must be exported"
    );
    assert!(
        relation_predicates
            .iter()
            .any(|pred| pred.starts_with("hypothesis_edge_")),
        "hypothesis edge relations must be exported"
    );
    assert!(
        relation_predicates
            .iter()
            .any(|pred| pred.starts_with("experiment_diff_")),
        "experiment diff relations must be exported"
    );
    assert!(
        relation_predicates.len() >= 4,
        "rich bundles should export at least four relation rows"
    );
}

#[test]
fn export_includes_verification_trials_and_refutation_artifacts() {
    let mut bundle = test_bundle("b-verification-artifacts");
    bundle.verification_trials = vec![
        VerificationTrial {
            trial_id: TrialId::new("trial-baseline-1"),
            attempt_id: AttemptId::new("attempt-v1"),
            baseline_or_patch: BaselineOrPatch::Baseline,
            completed: true,
            receipts: vec!["baseline-log".into()],
        },
        VerificationTrial {
            trial_id: TrialId::new("trial-patched-1"),
            attempt_id: AttemptId::new("attempt-v1"),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed: true,
            receipts: vec!["patched-log".into()],
        },
    ];

    bundle.refutation_artifacts = vec![
        RefutationArtifact {
            artifact_id: "art-placebo-1".into(),
            artifact_type: RefutationArtifactType::Placebo,
            trial_id: Some(TrialId::new("trial-baseline-1")),
            attempt_id: Some(AttemptId::new("attempt-v1")),
            outcome: RefutationArtifactOutcome::Passed,
            estimate_delta: Some(0.0),
            details: Some("placebo produced null effect".into()),
        },
        RefutationArtifact {
            artifact_id: "art-subsample-1".into(),
            artifact_type: RefutationArtifactType::SubsampleStability,
            trial_id: Some(TrialId::new("trial-patched-1")),
            attempt_id: Some(AttemptId::new("attempt-v1")),
            outcome: RefutationArtifactOutcome::Failed {
                reason: "high variance across folds".into(),
            },
            estimate_delta: Some(1.2),
            details: Some("variance check exceeded threshold".into()),
        },
    ];

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();

    let relation_preds: Vec<String> = envelope
        .records
        .iter()
        .filter_map(|record| {
            if let ExportRecord::Relation(relation) = record {
                Some(relation.predicate.clone())
            } else {
                None
            }
        })
        .collect();

    assert!(relation_preds
        .iter()
        .any(|pred| pred == "verification_trial_baseline"));
    assert!(relation_preds
        .iter()
        .any(|pred| pred == "verification_trial_patched"));
    assert!(relation_preds
        .iter()
        .any(|pred| pred.starts_with("verification_refutation_placebo")));
    assert!(relation_preds
        .iter()
        .any(|pred| pred.starts_with("verification_refutation_subsample_stability")));

    let relation_predicate_counts = relation_preds
        .iter()
        .filter(|pred| pred.starts_with("verification_refutation_"))
        .count();
    assert_eq!(relation_predicate_counts, 2);
}

#[test]
fn claim_metadata_carries_projected_verification_summary() {
    let mut bundle = test_bundle("b-verification-summary");
    bundle.causal_question = Some("Did the patch remove the regression?".into());
    bundle.unit_definition = Some("paired forge run".into());
    bundle.bundle_scope = Some(BundleScope {
        workload_id: "bench-a".into(),
        backend_family: "cargo".into(),
        selected_checks: vec!["cargo test".into(), "cargo bench".into()],
        timeout_class: "short".into(),
        config_flags: vec!["--all-features".into()],
    });
    bundle.pair_comparability = Some(PairComparability {
        valid: true,
        violations: vec![],
    });
    bundle.identification_rationale =
        Some("baseline and patched trials share workload and flags".into());
    bundle.known_threats = vec!["cache effects".into()];
    bundle.treatment = Some(Treatment {
        kind: "patch_applied".into(),
        patch_hash: "patch-123".into(),
        patch_summary: "edit src/lib.rs".into(),
    });
    bundle.covariates = Some(Covariates {
        env_fingerprint: "env-123".into(),
        dependency_fingerprint: Some("deps-123".into()),
        config_flags: vec!["--all-features".into()],
        workload_id: "bench-a".into(),
        selected_checks: vec!["cargo test".into(), "cargo bench".into()],
        adjacent_edits: false,
        adjacent_edit_signatures: vec![],
    });
    bundle.assessment = Some(EvidenceAssessment {
        reproducibility: AssessmentCategory::Strong,
        isolation: AssessmentCategory::Strong,
        contradiction_state: ContradictionState::Clean,
        sample_support: SampleSupport::Sufficient,
    });
    bundle.verification_trials = vec![
        VerificationTrial {
            trial_id: TrialId::new("trial-summary-baseline"),
            attempt_id: AttemptId::new("attempt-summary"),
            baseline_or_patch: BaselineOrPatch::Baseline,
            completed: true,
            receipts: vec!["receipt:baseline".into()],
        },
        VerificationTrial {
            trial_id: TrialId::new("trial-summary-patched"),
            attempt_id: AttemptId::new("attempt-summary"),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed: true,
            receipts: vec!["receipt:patched".into()],
        },
    ];
    bundle.refutation_artifacts = vec![RefutationArtifact {
        artifact_id: "artifact-placebo".into(),
        artifact_type: RefutationArtifactType::Placebo,
        trial_id: Some(TrialId::new("trial-summary-baseline")),
        attempt_id: Some(AttemptId::new("attempt-summary")),
        outcome: RefutationArtifactOutcome::Passed,
        estimate_delta: Some(0.02),
        details: Some("placebo preserved the null effect".into()),
    }];

    let export = EpisodeExport::from_bundle(&bundle, "test-ns");
    let envelope = export.to_export_envelope_v1(&bundle).unwrap();
    let claim = envelope
        .records
        .iter()
        .find_map(|record| match record {
            ExportRecord::Claim(claim) => Some(claim),
            _ => None,
        })
        .expect("claim record should be present");
    let metadata = claim
        .metadata
        .as_ref()
        .expect("claim metadata should exist");

    assert_eq!(
        metadata["comparability_snapshot"]["workload_id"],
        serde_json::json!("bench-a")
    );
    assert_eq!(
        metadata["comparability_snapshot"]["comparable"],
        serde_json::json!(true)
    );
    assert_eq!(
        metadata["comparability_snapshot_version"],
        serde_json::json!("bench-a:cargo:short")
    );
    assert_eq!(
        metadata["verification_summary"]["lifecycle_state"],
        serde_json::json!("verified")
    );
    assert_eq!(
        metadata["verification_summary"]["promotion_state"]["state"],
        serde_json::json!("eligible")
    );
    assert_eq!(
        metadata["promotion_state"]["state"],
        serde_json::json!("eligible")
    );
    assert_eq!(
        metadata["verification_summary"]["completed_trial_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        metadata["verification_summary"]["passed_refutation_count"],
        serde_json::json!(1)
    );
    assert_eq!(
        metadata["verification_trial_family"]
            .as_array()
            .expect("verification trials should serialize")
            .len(),
        2
    );
    assert_eq!(
        metadata["refutation_artifacts"]
            .as_array()
            .expect("refutation artifacts should serialize")
            .len(),
        1
    );
    assert_eq!(
        metadata["estimator_meta"]["version"],
        serde_json::json!("v0001")
    );
}

#[tokio::test]
async fn export_bundle_projects_store_promotions_into_claim_metadata() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();
    let bundle = test_bundle("b-promoted");

    store
        .insert_promotion(
            "basis-42",
            &bundle.candidate_id,
            "{}",
            "{}",
            "{}",
            "checksum-42",
            None,
        )
        .unwrap();

    let envelope = forge_engine::export_bundle(&bundle, "default", &store)
        .await
        .unwrap();
    let claim = envelope
        .records
        .iter()
        .find_map(|record| match &record.record {
            semantic_memory_forge::ExportRecord::Claim(claim) => Some(claim),
            _ => None,
        })
        .expect("claim record should be present");
    let metadata = claim
        .metadata
        .as_ref()
        .expect("claim metadata should exist");

    assert_eq!(
        metadata["promotion_state"]["state"],
        serde_json::json!("promoted")
    );
    assert_eq!(
        metadata["promotion_state"]["version_id"],
        serde_json::json!("basis-42")
    );
    assert_eq!(
        metadata["verification_summary"]["promotion_state"]["state"],
        serde_json::json!("promoted")
    );
    assert_eq!(
        metadata["verification_summary"]["promotion_state"]["version_id"],
        serde_json::json!("basis-42")
    );
    assert!(
        metadata["verification_summary"]["promotion_state"]["promoted_at"]
            .as_str()
            .is_some(),
        "store-backed promotion state should carry promotion timestamp"
    );
}
