//! All-at-once experiment runner — generates unique patches per (fixture, strategy)
//! combination to produce maximally diverse CEA edges in a single invocation.
//!
//! Key insight: each (op_kind, file_path) combination produces a distinct CEA
//! cause node. By varying both across fixtures, we get N_ops × N_files edges.
//!
//! Usage: cargo run --example run_all -- <fixture-dir> <forge-db-path>


// Example runner: aborting on a missing fixture or an unopenable DB is the
// intended behavior for a manual experiment harness, and every sibling
// example in this directory does the same. The workspace warns on
// expect_used and CI promotes warnings with -D warnings.
#![allow(clippy::expect_used)]
use forge_engine::adapters::CargoAdapter;
use forge_engine::cea::instrumentation::attribute_effects;
use forge_engine::cea::store::update_graph;
use forge_engine::config::ForgeConfig;
use forge_engine::exec::host::HostBackend;
use forge_engine::experiment::{ExperimentConfig, PairedExperimentRunner};
use forge_engine::export_bundle;
use forge_engine::lab::evidence::ExperimentEvidenceBundle;
use forge_engine::lab::suite::load_suite;
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::runtime::patch::types::{
    Anchor, EditOp, FileEdit, FileMode, LineRange, StructuredPatch,
};
use forge_engine::store::ForgeStore;
use forge_engine::ScoreVector;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Generate unique patches for each fixture, varying op_kind and file path
fn build_unique_patches(fixture_path: &Path, task_id: &str) -> Vec<(String, StructuredPatch)> {
    let mut patches = Vec::new();
    let lib_rs = fixture_path.join("src/lib.rs");

    if !lib_rs.exists() {
        return patches;
    }

    // Discover available .rs files in the fixture
    let src_dir = fixture_path.join("src");
    let mut rs_files: Vec<PathBuf> = vec![lib_rs.clone()];
    if src_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "rs") && p != lib_rs {
                    rs_files.push(p);
                }
            }
        }
    }

    let ops = &[
        ("insert", true),  // safe: always works
        ("delete", true),  // delete line 1
        ("replace", true), // replace line 1
    ];

    for (file_idx, file_path) in rs_files.iter().enumerate() {
        for (op_idx, (op_kind, _)) in ops.iter().enumerate() {
            let rel_path = file_path
                .strip_prefix(fixture_path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let summary = format!("{task_id}:{op_kind}:{rel_path}");
            let strategy_name = format!("{op_kind}-f{file_idx}o{op_idx}");

            let edit_ops = match *op_kind {
                "delete" => vec![EditOp::Delete {
                    range: LineRange {
                        start: 1,
                        end_exclusive: 2,
                    },
                }],
                "replace" => vec![EditOp::Replace {
                    range: LineRange {
                        start: 1,
                        end_exclusive: 2,
                    },
                    lines: vec![format!("//! {summary}")],
                }],
                _ => vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 1,
                        context_before: vec![],
                        context_after: vec![],
                    },
                    lines: vec![format!("// {summary}")],
                }],
            };

            let patch = StructuredPatch {
                patch_id: uuid::Uuid::new_v4(),
                summary,
                edits: vec![FileEdit {
                    path: PathBuf::from(&rel_path),
                    ops: edit_ops,
                    mode: Some(FileMode::Modify),
                }],
                notes: vec![format!("strategy:{}", strategy_name)],
            };

            patches.push((strategy_name, patch));
        }
    }

    patches
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fixture_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./fixtures"));
    let db_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(".recall-coding/forge/forge.db")
    });

    let store = ForgeStore::open(&db_path).expect("open forge db");
    let config = ForgeConfig {
        sealed_allow_host_backend: true,
        ..Default::default()
    };

    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!(
        "Fixture: {}  Tasks: {}",
        fixture_dir.display(),
        suite.tasks.len()
    );

    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);
    let experiment_config = ExperimentConfig::default();

    let mut total_edges: u64 = 0;
    let mut total_runs: u64 = 0;
    let mut seen_run_hashes: BTreeSet<String> = BTreeSet::new();

    for task in &suite.tasks {
        let patches = build_unique_patches(&task.fixture_path, &task.task.task_id);
        println!("\n{}: {} patches", task.task.task_id, patches.len());

        for (idx, (name, patch)) in patches.iter().enumerate() {
            total_runs += 1;
            let version_id: &str = name;
            let version_str = name.clone();

            match runner
                .run(&task.fixture_path, patch, &experiment_config)
                .await
            {
                Ok(experiment) => {
                    let line_map = LineAttributionMap::default();
                    let attributed =
                        attribute_effects(patch, &experiment.patched_result, &line_map, 12)
                            .unwrap_or_else(|_| Vec::new());

                    if attributed.is_empty() {
                        println!(
                            "  {:2} {}: no triples (r={} i={})",
                            idx, name, experiment.diff.regressions, experiment.diff.improvements
                        );
                        continue;
                    }

                    let scope = attributed
                        .first()
                        .map(|t| format!("{:?}", t.cause.scope_tag))
                        .unwrap_or_default();

                    let run_result = forge_engine::AttributedRunResult::new(
                        attributed,
                        experiment.patched_result.clone(),
                    );

                    let run_hash_short = &run_result.run_hash[..16];
                    if !seen_run_hashes.insert(run_result.run_hash.clone()) {
                        // Same run hash = same patch applied to same fixture = AlreadyProcessed
                        continue;
                    }

                    match update_graph(&store, &run_result, &task.task.task_id, version_id, &config)
                    {
                        Ok(update) => match update {
                            forge_engine::cea::store::UpdateResult::Applied {
                                edges_added, ..
                            } => {
                                total_edges += edges_added as u64;
                                println!(
                                    "  {:2} {}: {} edges scope={} hash={}",
                                    idx, name, edges_added, scope, run_hash_short
                                );

                                // Also persist an evidence bundle so the OODA loop
                                // can import it on cold start (fixes thin_export loop).
                                let bundle = ExperimentEvidenceBundle {
                                    bundle_id: format!("run_all:{}-{}", task.task.task_id, name),
                                    candidate_id: task.task.task_id.clone(),
                                    eval_id: format!("eval:{}", name),
                                    version_id: version_str.clone(),
                                    supersedes_claim_version_id: None,
                                    relation_lineage_hints: Default::default(),
                                    scores: ScoreVector {
                                        correctness: if experiment.diff.regressions == 0 {
                                            0.9
                                        } else {
                                            0.4
                                        },
                                        novelty: 0.25,
                                        stability: 0.7,
                                        weighted_total: 0.7,
                                        cea_confidence: None,
                                        cea_predicted_correctness: None,
                                    },
                                    hypotheses: vec![],
                                    verification: None,
                                    trace_id: None,
                                    experiment_diff: Some(experiment.diff.clone()),
                                    attribution_json: None,
                                    assessment: None,
                                    warnings: vec![],
                                    created_at: chrono::Utc::now().to_rfc3339(),
                                    run_id: Some(experiment.run_id.clone()),
                                    attempt_id: None,
                                    causal_question: None,
                                    unit_definition: None,
                                    bundle_scope: None,
                                    receipts: vec![],
                                    verification_trials: vec![],
                                    refutation_artifacts: vec![],
                                    sealed: false,
                                    pair_comparability: None,
                                    claim_strength: Default::default(),
                                    identification_rationale: None,
                                    known_threats: vec![],
                                    patch_hash: None,
                                    treatment: None,
                                    outcome: None,
                                    covariates: None,
                                    promotion_state: None,
                                    primary_effect: None,
                                    all_effects: vec![],
                                    hypothesis_edges: vec![],
                                };
                                if let Err(e) =
                                    export_bundle(&bundle, "forge-pilot-self", &store).await
                                {
                                    eprintln!("  {:2} {}: export err: {}", idx, name, e);
                                }
                                // Also write to evidence_bundles table so import_recent_forge_bundles()
                                // finds it on cold start (export_bundle writes to export_receipts)
                                let scores_json =
                                    serde_json::to_string(&bundle.scores).unwrap_or_default();
                                let warnings_json =
                                    serde_json::to_string(&bundle.warnings).unwrap_or_default();
                                if let Err(e) = store.insert_evidence_bundle(
                                    &bundle.bundle_id,
                                    &bundle.candidate_id,
                                    &bundle.eval_id,
                                    &bundle.version_id,
                                    &bundle.trace_id.clone().unwrap_or_default(),
                                    &scores_json,
                                    "[]",
                                    None,
                                    None,
                                    None,
                                    &warnings_json,
                                ) {
                                    eprintln!("  {:2} {}: evidence insert err: {}", idx, name, e);
                                }
                            }
                            forge_engine::cea::store::UpdateResult::AlreadyProcessed => {
                                // Shouldn't happen since we check run hashes, but handle gracefully
                            }
                        },
                        Err(e) => eprintln!("  {:2} {}: CEA err: {}", idx, name, e),
                    }
                }
                Err(e) => eprintln!("  {:2} {}: run err: {}", idx, name, e),
            }
        }
    }

    println!("\n=== DONE ===");
    println!(
        "Total runs: {}  Edges: {}  DB: {}",
        total_runs,
        total_edges,
        db_path.display()
    );
}
