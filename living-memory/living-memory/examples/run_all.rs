//! All-at-once experiment runner — generates unique patches per (fixture, strategy)
//! combination to produce maximally diverse CEA edges in a single invocation.
//!
//! Key insight: each (op_kind, file_path) combination produces a distinct CEA
//! cause node. By varying both across fixtures, we get N_ops × N_files edges.
//!
//! Usage: cargo run --example run_all -- <fixture-dir> <forge-db-path>

use forge_engine::adapters::CargoAdapter;
use forge_engine::config::ForgeConfig;
use forge_engine::exec::host::HostBackend;
use forge_engine::experiment::{ExperimentConfig, PairedExperimentRunner};
use forge_engine::lab::suite::load_suite;
use forge_engine::runtime::patch::types::{
    Anchor, EditOp, FileEdit, FileMode, LineRange, StructuredPatch,
};
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::cea::instrumentation::attribute_effects;
use forge_engine::cea::store::update_graph;
use forge_engine::store::ForgeStore;
use std::path::{Path, PathBuf};
use std::collections::BTreeSet;

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
                    range: LineRange { start: 1, end_exclusive: 2 },
                }],
                "replace" => vec![EditOp::Replace {
                    range: LineRange { start: 1, end_exclusive: 2 },
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
    let fixture_dir = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("./fixtures"));
    let db_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(".recall-coding/forge/forge.db")
    });

    let store = ForgeStore::open(&db_path).expect("open forge db");
    let mut config = ForgeConfig::default();
    config.sealed_allow_host_backend = true;

    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!("Fixture: {}  Tasks: {}", fixture_dir.display(), suite.tasks.len());

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
            let version_id = &name;

            match runner.run(&task.fixture_path, patch, &experiment_config).await {
                Ok(experiment) => {
                    let line_map = LineAttributionMap::default();
                    let attributed = attribute_effects(patch, &experiment.patched_result, &line_map, 12)
                        .unwrap_or_else(|_| Vec::new());

                    if attributed.is_empty() {
                        println!("  {:2} {}: no triples (r={} i={})",
                            idx, name, experiment.diff.regressions, experiment.diff.improvements);
                        continue;
                    }

                    let scope = attributed.first()
                        .map(|t| format!("{:?}", t.cause.scope_tag))
                        .unwrap_or_default();

                    let run_result = forge_engine::AttributedRunResult::new(
                        attributed, experiment.patched_result.clone(),
                    );

                    let run_hash_short = &run_result.run_hash[..16];
                    if !seen_run_hashes.insert(run_result.run_hash.clone()) {
                        // Same run hash = same patch applied to same fixture = AlreadyProcessed
                        continue;
                    }

                    match update_graph(&store, &run_result, &task.task.task_id, version_id, &config) {
                        Ok(update) => match update {
                            forge_engine::cea::store::UpdateResult::Applied { edges_added, .. } => {
                                total_edges += edges_added as u64;
                                println!("  {:2} {}: {} edges scope={} hash={}",
                                    idx, name, edges_added, scope, run_hash_short);
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
    println!("Total runs: {}  Edges: {}  DB: {}", total_runs, total_edges, db_path.display());
}
