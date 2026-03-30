use constraint_compiler::{compile_batch, CompilerPolicy};
use forge_engine::experiment::{EffectKind, TypedLocatedEffect};
use forge_engine::export::EpisodeExport;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::{CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle, HypothesisStatus};
use forge_memory_bridge::transform_envelope_v3;
use kernel_execution::execute_delta_propagation;
use knowledge_runtime::{
    adapters::semantic_memory::SemanticMemoryAdapter, config::ProjectionConfig, KnowledgeRuntime,
    RuntimeConfig, Scope,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use std::time::Instant;
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
            file: Some(std::path::PathBuf::from(format!("src/perf_{index}.rs"))),
            line: Some(10 + index as u32),
            message: format!("perf effect {index}"),
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

struct SnapshotRow {
    label: &'static str,
    effect_count: usize,
    region_count: usize,
    delta_region_count: usize,
    export_ms: u128,
    transform_ms: u128,
    import_ms: u128,
    compile_ms: u128,
    advisory_ms: u128,
}

async fn capture_row(label: &'static str, effect_count: usize) -> SnapshotRow {
    let dir = TempDir::new().unwrap();
    let bundle = canonical_bundle(label, effect_count);
    let export = EpisodeExport::from_bundle(&bundle, &format!("perf-{label}"));

    let started = Instant::now();
    let envelope = export.to_export_envelope_v3(&bundle).unwrap();
    let export_ms = started.elapsed().as_millis();

    let started = Instant::now();
    let batch = transform_envelope_v3(&envelope).unwrap();
    let transform_ms = started.elapsed().as_millis();

    let memory = open_store(dir.path());
    let started = Instant::now();
    memory.import_projection_batch(&batch).await.unwrap();
    let import_ms = started.elapsed().as_millis();

    let started = Instant::now();
    let compiled = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: format!("perf-{label}"),
            include_hyperedges: true,
        },
    );
    let compile_ms = started.elapsed().as_millis();
    let changed = compiled
        .nodes
        .first()
        .map(|node| vec![node.node_id.clone()])
        .unwrap_or_default();
    let delta = execute_delta_propagation(&compiled, &changed, 3);

    let runtime = runtime_for_store(memory, &format!("perf-{label}"));
    let started = Instant::now();
    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new(format!("perf-{label}"))))
        .await
        .unwrap();
    let advisory_ms = started.elapsed().as_millis();
    assert!(
        advisory.is_some(),
        "canonical perf fixture should emit advisory"
    );

    SnapshotRow {
        label,
        effect_count,
        region_count: compiled.regions.len(),
        delta_region_count: delta.recomputed_region_ids.len(),
        export_ms,
        transform_ms,
        import_ms,
        compile_ms,
        advisory_ms,
    }
}

#[tokio::main]
async fn main() {
    let rows = [
        capture_row("small", 2).await,
        capture_row("large", 11).await,
    ];
    let captured_on = chrono::Utc::now().date_naive();

    println!("# PERFORMANCE_BASELINE.md");
    println!();
    println!(
        "Captured on {} from the canonical V3 lane using `cargo run -p kernel-conformance --example canonical_perf_snapshot`.",
        captured_on
    );
    println!();
    println!("These numbers are regression alarms, not throughput claims. Regenerate them after meaningful changes to export, bridge, import, compile, or advisory behavior.");
    println!();
    println!("| Fixture | Effect records | Regions | Delta regions | Export ms | Transform ms | Import ms | Compile ms | Advisory ms |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for row in rows {
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.label,
            row.effect_count,
            row.region_count,
            row.delta_region_count,
            row.export_ms,
            row.transform_ms,
            row.import_ms,
            row.compile_ms,
            row.advisory_ms
        );
    }
}
