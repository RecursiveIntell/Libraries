#![allow(clippy::expect_used)]

use constraint_compiler::{compile_batch, CompilerPolicy};
use forge_engine::experiment::{EffectKind, TypedLocatedEffect};
use forge_engine::export::EpisodeExport;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::{CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle, HypothesisStatus};
use forge_memory_bridge::{transform_envelope_v3, ImportProjectionRecord};
use kernel_oracles::{evaluate_causal_refuter, OracleRefutationOutcome};
use knowledge_runtime::{
    adapters::semantic_memory::SemanticMemoryAdapter, config::ProjectionConfig, KnowledgeRuntime,
    RuntimeConfig, Scope,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use stack_ids::RelationVersionId;
use tempfile::TempDir;

fn open_store(base_dir: &std::path::Path) -> MemoryStore {
    let config = MemoryConfig {
        base_dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).unwrap()
}

fn runtime_for_store(store: MemoryStore, namespace: &str) -> KnowledgeRuntime {
    let adapter = SemanticMemoryAdapter::new(store);
    KnowledgeRuntime::new(
        RuntimeConfig {
            default_scope: Scope::new(namespace),
            query: Default::default(),
            entity: Default::default(),
            projection: ProjectionConfig {
                staleness_threshold_secs: 3600,
                import_staleness_threshold_secs: 0,
                persist: false,
            },
            strict_temporal: false,
            strict_scope: false,
        },
        adapter,
    )
    .unwrap()
}

fn canonical_bundle(bundle_id: &str, effect_count: usize) -> ExperimentEvidenceBundle {
    let all_effects = (0..effect_count)
        .map(|index| TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: Some(std::path::PathBuf::from(format!("src/test_{index}.rs"))),
            line: Some(10 + index as u32),
            message: format!("kernel witness effect {index}"),
            in_baseline: false,
            in_patched: true,
        })
        .collect();

    ExperimentEvidenceBundle {
        bundle_id: bundle_id.into(),
        candidate_id: format!("candidate-{bundle_id}"),
        eval_id: format!("eval-{bundle_id}"),
        version_id: "v0001".into(),
        scores: ScoreVector {
            correctness: 0.9,
            novelty: 0.2,
            stability: 0.8,
            weighted_total: 0.85,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis-{bundle_id}"),
            cause_signature: "cause".into(),
            effect_signature: "effect".into(),
            confidence: 0.7,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        verification: None,
        trace_id: Some(format!("trace-{bundle_id}")),
        experiment_diff: None,
        attribution_json: None,
        assessment: None,
        warnings: vec![],
        created_at: "2026-03-07T00:00:00Z".into(),
        run_id: Some(format!("run-{bundle_id}")),
        attempt_id: Some(format!("attempt-{bundle_id}")),
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
        primary_effect: None,
        all_effects,
        hypothesis_edges: vec![],
        receipts: vec![],
        verification_trials: vec![],
        refutation_artifacts: vec![],
        sealed: false,
    }
}

fn effect_relation_ids(
    batch: &forge_memory_bridge::ProjectionImportBatchV3,
) -> Vec<RelationVersionId> {
    let mut ids = batch
        .records
        .iter()
        .filter_map(|record| match &record.record {
            ImportProjectionRecord::RelationVersion(relation)
                if relation.predicate == "all_effect_test_failure" =>
            {
                Some(relation.relation_version_id.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

#[tokio::test]
async fn living_memory_export_reaches_runtime_advisory_and_kernel_refuter() {
    let dir = TempDir::new().unwrap();
    let bundle = canonical_bundle("roundtrip", 2);
    let export = EpisodeExport::from_bundle(&bundle, "cf-living-roundtrip");
    let envelope = export.to_export_envelope_v3(&bundle).unwrap();
    let batch = transform_envelope_v3(&envelope).unwrap();
    let relation_ids = effect_relation_ids(&batch);
    let memory = open_store(dir.path());

    let imported = memory.import_projection_batch(&batch).await.unwrap();
    assert_eq!(imported.status, "complete");

    let runtime = runtime_for_store(memory.clone(), "cf-living-roundtrip");
    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("cf-living-roundtrip")))
        .await
        .unwrap()
        .expect("expected advisory from imported living-memory V3 batch");

    assert!(advisory.advisory_only);
    assert!(advisory.message_count >= 1);
    assert!(
        advisory
            .degradation_markers
            .contains(&"thin_export".to_string()),
        "living-memory canonical export should degrade explicitly when alias-style records stay thin"
    );

    assert_eq!(relation_ids.len(), 2);
    let compiled = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "kernel-conformance.living-memory-roundtrip".into(),
            include_hyperedges: true,
        },
    );
    let refutation = evaluate_causal_refuter(&compiled, relation_ids[0].as_str(), 1);

    match refutation.outcome {
        OracleRefutationOutcome::FlipWitness { removed_node_ids } => {
            assert_eq!(removed_node_ids, vec![relation_ids[1].as_str().to_string()]);
        }
        other => {
            panic!("expected flip witness from canonical living-memory roundtrip, got {other:?}")
        }
    }
}

#[tokio::test]
async fn living_memory_export_preserves_budgeted_no_flip_found_on_large_relation_group() {
    let dir = TempDir::new().unwrap();
    let bundle = canonical_bundle("budget", 11);
    let export = EpisodeExport::from_bundle(&bundle, "cf-living-budget");
    let envelope = export.to_export_envelope_v3(&bundle).unwrap();
    let batch = transform_envelope_v3(&envelope).unwrap();
    let relation_ids = effect_relation_ids(&batch);
    let memory = open_store(dir.path());

    let imported = memory.import_projection_batch(&batch).await.unwrap();
    assert_eq!(imported.status, "complete");
    assert_eq!(relation_ids.len(), 11);

    let compiled = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "kernel-conformance.living-memory-budget".into(),
            include_hyperedges: true,
        },
    );
    let refutation = evaluate_causal_refuter(&compiled, relation_ids[0].as_str(), 10);

    match refutation.outcome {
        OracleRefutationOutcome::NoFlipFound { searched_budget } => {
            assert!(
                searched_budget >= 10,
                "bounded search should stop once the configured budget is exhausted"
            );
        }
        other => panic!("expected budgeted NoFlipFound, got {other:?}"),
    }
}
