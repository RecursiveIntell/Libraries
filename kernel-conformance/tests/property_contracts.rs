#![allow(clippy::expect_used)]
#![allow(deprecated)]

use constraint_compiler::{compile_batch, CompilerPolicy};
use forge_engine::experiment::{EffectKind, TypedLocatedEffect};
use forge_engine::export::EpisodeExport;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::{CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle, HypothesisStatus};
use forge_memory_bridge::transform_envelope_v3;
use knowledge_runtime::{
    adapters::semantic_memory::SemanticMemoryAdapter, config::ProjectionConfig, KnowledgeRuntime,
    RuntimeConfig, Scope,
};
use proptest::prelude::*;
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use semantic_memory_forge::{
    ExportClaim, ExportEnvelopeV1, ExportEnvelopeV2, ExportEnvelopeV3, ExportRecord,
    EXPORT_ENVELOPE_V1_SCHEMA,
};
use stack_ids::{ClaimId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey};
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
            file: Some(std::path::PathBuf::from(format!("src/property_{index}.rs"))),
            line: Some(10 + index as u32),
            message: format!("property effect {index}"),
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
        created_at: "2026-03-11T00:00:00Z".into(),
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

fn permutation(choice: u8) -> [usize; 3] {
    match choice % 6 {
        0 => [0, 1, 2],
        1 => [0, 2, 1],
        2 => [1, 0, 2],
        3 => [1, 2, 0],
        4 => [2, 0, 1],
        _ => [2, 1, 0],
    }
}

fn base_records() -> Vec<ExportRecord> {
    vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-order-1")),
            claim_version_id: Some(ClaimVersionId::new("claim-version:claim-version-order-1")),
            subject_entity_id: EntityId::new("entity-order-1"),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("result-a"),
            valid_from: None,
            valid_to: None,
            confidence: 1.0,
            content: "first".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-order-2")),
            claim_version_id: Some(ClaimVersionId::new("claim-version:claim-version-order-2")),
            subject_entity_id: EntityId::new("entity-order-2"),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("result-b"),
            valid_from: None,
            valid_to: None,
            confidence: 1.0,
            content: "second".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-order-3")),
            claim_version_id: Some(ClaimVersionId::new("claim-version:claim-version-order-3")),
            subject_entity_id: EntityId::new("entity-order-3"),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("result-c"),
            valid_from: None,
            valid_to: None,
            confidence: 1.0,
            content: "third".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
    ]
}

fn envelope_for_permutation(choice: u8) -> ExportEnvelopeV1 {
    let records = base_records();
    let order = permutation(choice);
    let reordered = order
        .into_iter()
        .map(|idx| records[idx].clone())
        .collect::<Vec<_>>();
    let scope_key = ScopeKey::namespace_only("property-order");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope_key, &reordered).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new(format!("env-order-{}", choice % 6)),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: None,
        exported_at: "2026-03-11T00:00:00Z".into(),
        records: reordered,
    }
}

fn compiled_signature(
    output: &constraint_compiler::CompileOutput,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut nodes = output
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let mut edges = output
        .hyperedges
        .iter()
        .map(|edge| edge.edge_id.clone())
        .collect::<Vec<_>>();
    let mut constraints = output
        .constraints
        .iter()
        .map(|constraint| constraint.constraint_id.as_str().to_string())
        .collect::<Vec<_>>();
    nodes.sort();
    edges.sort();
    constraints.sort();
    (nodes, edges, constraints)
}

proptest! {
    #[test]
    fn compiler_signature_is_stable_under_record_reordering(choice in 0u8..6) {
        let baseline = envelope_for_permutation(0);
        let candidate = envelope_for_permutation(choice);
        let baseline_batch = transform_envelope_v3(
            &ExportEnvelopeV3::try_from(ExportEnvelopeV2::from(baseline)).unwrap(),
        ).unwrap();
        let candidate_batch = transform_envelope_v3(
            &ExportEnvelopeV3::try_from(ExportEnvelopeV2::from(candidate)).unwrap(),
        ).unwrap();

        let policy = CompilerPolicy {
            policy_version: "kernel-conformance.property.order".into(),
            include_hyperedges: true,
        };

        prop_assert_eq!(
            compiled_signature(&compile_batch(&baseline_batch, &policy)),
            compiled_signature(&compile_batch(&candidate_batch, &policy)),
        );
    }

    #[test]
    fn living_memory_thin_export_degradation_is_explicit(effect_count in 1usize..6) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async move {
            let dir = TempDir::new().unwrap();
            let bundle = canonical_bundle("property-thin", effect_count);
            let export = EpisodeExport::from_bundle(&bundle, "property-thin");
            let envelope = export.to_export_envelope_v3(&bundle).unwrap();
            let batch = transform_envelope_v3(&envelope).unwrap();
            let memory = open_store(dir.path());

            let imported = memory.import_projection_batch(&batch).await.unwrap();
            prop_assert_eq!(imported.status, "complete");

            let runtime = runtime_for_store(memory, "property-thin");
            let advisory = runtime
                .latest_inference_advisory(Some(&Scope::new("property-thin")))
                .await
                .unwrap()
                .expect("expected advisory from imported batch");

            prop_assert!(advisory
                .degradation_markers
                .contains(&"thin_export".to_string()));
            Ok(())
        })?;
    }
}
