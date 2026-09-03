//! Minimal forge-engine experiment runner — runs paired experiments on task
//! fixtures and records results in the forge DB + CEA graph.
//!
//! Usage: cargo run --example run_experiment -- <fixture-dir> <forge-db-path>


// Example runner: aborting on a missing fixture or an unopenable DB is the
// intended behavior for a manual experiment harness, and every sibling
// example in this directory does the same. The workspace warns on
// expect_used and CI promotes warnings with -D warnings.
#![allow(clippy::expect_used)]
use forge_engine::adapters::CargoAdapter;
use forge_engine::cea::instrumentation::{attribute_effects, AttributedRunResult};
use forge_engine::cea::store::update_graph;
use forge_engine::config::ForgeConfig;
use forge_engine::exec::host::HostBackend;
use forge_engine::experiment::{ExperimentConfig, PairedExperimentRunner};
use forge_engine::lab::evaluate::compute_scores;
use forge_engine::lab::suite::{load_suite, EvalTask};
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::runtime::patch::types::{Anchor, LineRange};
use forge_engine::runtime::patch::types::{EditOp, FileEdit, FileMode, StructuredPatch};
use forge_engine::store::ForgeStore;
use forge_engine::ForgeConfig as FC;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fixture_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./fixtures"));
    let db_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(".recall/forge/forge.db")
    });

    // Open forge store
    let store = ForgeStore::open(&db_path).expect("open forge db");

    // Config with host backend allowed (not sealed)
    let mut config = ForgeConfig::default();
    config.sealed_allow_host_backend = true;

    // Load evaluation suite
    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!("Suite: {} ({} tasks)", suite.name, suite.tasks.len());

    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);

    // Build a trivial patch (insert a comment — safe, no behavior change)
    let trivial_patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: "trivial comment insertion".to_string(),
        edits: vec![FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// forge-experiment: trivial comment".to_string()],
            }],
            mode: Some(FileMode::Modify),
        }],
        notes: vec![],
    };

    let experiment_config = ExperimentConfig::default();
    let version_id = "v1";

    for task in &suite.tasks {
        println!("\n=== Task: {} ===", task.task.task_id);
        println!("  fixture: {}", task.fixture_path.display());

        // Run paired experiment
        let result = runner
            .run(&task.fixture_path, &trivial_patch, &experiment_config)
            .await;

        match result {
            Ok(experiment) => {
                let fmt_ok =
                    experiment.baseline_result.fmt_pass && experiment.patched_result.fmt_pass;
                let clippy_ok =
                    experiment.baseline_result.clippy_pass && experiment.patched_result.clippy_pass;
                let test_ok =
                    experiment.baseline_result.test_pass && experiment.patched_result.test_pass;
                println!(
                    "  baseline: fmt={} clippy={} test={}",
                    experiment.baseline_result.fmt_pass,
                    experiment.baseline_result.clippy_pass,
                    experiment.baseline_result.test_pass
                );
                println!(
                    "  patched:  fmt={} clippy={} test={}",
                    experiment.patched_result.fmt_pass,
                    experiment.patched_result.clippy_pass,
                    experiment.patched_result.test_pass
                );
                println!(
                    "  diff: regressions={} improvements={}",
                    experiment.diff.regressions, experiment.diff.improvements
                );

                // Attribute effects to the patch, compute run hash, update CEA graph
                let line_map = LineAttributionMap::default();
                let attributed =
                    attribute_effects(&trivial_patch, &experiment.patched_result, &line_map, 12)
                        .unwrap_or_else(|e| {
                            eprintln!("  attribution error: {e}");
                            Vec::new()
                        });
                if !attributed.is_empty() {
                    let run_result = forge_engine::AttributedRunResult::new(
                        attributed,
                        experiment.patched_result.clone(),
                    );
                    match update_graph(&store, &run_result, &task.task.task_id, version_id, &config)
                    {
                        Ok(update) => println!("  CEA: {:?}", update),
                        Err(e) => eprintln!("  CEA update error: {e}"),
                    }
                } else {
                    println!("  CEA: no triples (cold)");
                }

                // Compute scores (handle potential errors gracefully)
                match compute_scores(
                    &experiment.patched_result,
                    &trivial_patch,
                    &task.task,
                    &store,
                    &task.task.task_id,
                    &config,
                ) {
                    Ok(scores) => println!(
                        "  scores: correctness={:.3} novelty={:.3} stability={:.3}",
                        scores.correctness, scores.novelty, scores.stability
                    ),
                    Err(e) => eprintln!("  score computation error: {e}"),
                }
            }
            Err(e) => {
                eprintln!("  experiment failed: {e}");
            }
        }
    }

    println!("\n=== DONE ===");
    println!("forge db: {}", db_path.display());
}
