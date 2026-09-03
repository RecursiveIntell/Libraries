//! Varied experiment runner — generates DIFFERENT patches for each fixture
//! to produce diverse CEA causal edges (not just one repeated pattern).
//!
//! Usage: cargo run --example run_varied -- <fixture-dir> <forge-db-path>

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
use forge_engine::lab::evaluate::compute_scores;
use forge_engine::lab::suite::load_suite;
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::runtime::patch::types::{
    Anchor, EditOp, FileEdit, FileMode, LineRange, StructuredPatch,
};
use forge_engine::store::ForgeStore;
use std::path::{Path, PathBuf};

fn build_patch_for_task(task_id: &str) -> StructuredPatch {
    match task_id {
        // Strategy 1: Insert comment in submodule file (different file = different CEA node)
        "cea-core-baseline" => StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "insert comment in attribution.rs".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("src/attribution.rs"),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 1,
                        context_before: vec![],
                        context_after: vec![],
                    },
                    lines: vec!["// variant-A: comment in submodule file".into()],
                }],
                mode: Some(FileMode::Modify),
            }],
            notes: vec!["variant: insert-submodule-comment".into()],
        },

        // Strategy 2: Insert an unused import (triggers clippy warning — different effect)
        "cea-sqlite-baseline" => StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "insert unused import (clippy)".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("src/lib.rs"),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 9, // after the doc comment, before use statements
                        context_before: vec![],
                        context_after: vec![],
                    },
                    lines: vec!["use std::collections::HashMap; // variant-B: unused import".into()],
                }],
                mode: Some(FileMode::Modify),
            }],
            notes: vec!["variant: unused-import".into()],
        },

        // Strategy 3: Delete the first comment line (different edit kind: Delete)
        "cea-store-baseline" => StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "delete first comment line".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("src/lib.rs"),
                ops: vec![EditOp::Delete {
                    range: LineRange {
                        start: 1,
                        end_exclusive: 2,
                    },
                }],
                mode: Some(FileMode::Modify),
            }],
            notes: vec!["variant: delete-comment".into()],
        },

        // Strategy 4: Replace a comment line (different edit kind: Replace)
        "typed-patch-baseline" => StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "replace comment line".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("src/lib.rs"),
                ops: vec![EditOp::Replace {
                    range: LineRange {
                        start: 1,
                        end_exclusive: 2,
                    },
                    lines: vec!["//! variant-D: replaced comment for typed-patch".into()],
                }],
                mode: Some(FileMode::Modify),
            }],
            notes: vec!["variant: replace-comment".into()],
        },

        // Fallback for unknown tasks
        _ => StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "trivial comment insertion (fallback)".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("src/lib.rs"),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 1,
                        context_before: vec![],
                        context_after: vec![],
                    },
                    lines: vec!["// forge-varied: generic comment".into()],
                }],
                mode: Some(FileMode::Modify),
            }],
            notes: vec!["variant: fallback-insert".into()],
        },
    }
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
    let mut config = ForgeConfig::default();
    config.sealed_allow_host_backend = true;

    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!("Suite: {} ({} tasks)\n", suite.name, suite.tasks.len());

    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);
    let experiment_config = ExperimentConfig::default();
    let version_id = "v3-varied";

    for task in &suite.tasks {
        let patch = build_patch_for_task(&task.task.task_id);
        println!(
            "=== Task: {} ({}) ===",
            task.task.task_id,
            patch.notes.first().map(|s| s.as_str()).unwrap_or("default")
        );
        println!("  fixture: {}", task.fixture_path.display());
        println!("  patch:   {}", patch.summary);

        let result = runner
            .run(&task.fixture_path, &patch, &experiment_config)
            .await;

        match result {
            Ok(experiment) => {
                let baseline = &experiment.baseline_result;
                let patched = &experiment.patched_result;
                println!(
                    "  baseline: fmt={} clippy={} test={}",
                    baseline.fmt_pass, baseline.clippy_pass, baseline.test_pass
                );
                println!(
                    "  patched:  fmt={} clippy={} test={}",
                    patched.fmt_pass, patched.clippy_pass, patched.test_pass
                );
                println!(
                    "  diff:     regressions={} improvements={}",
                    experiment.diff.regressions, experiment.diff.improvements
                );

                // CEA attribution
                let line_map = LineAttributionMap::default();
                let attributed =
                    attribute_effects(&patch, &experiment.patched_result, &line_map, 12)
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

                // Scores
                match compute_scores(
                    &experiment.patched_result,
                    &patch,
                    &task.task,
                    &store,
                    &task.task.task_id,
                    &config,
                ) {
                    Ok(scores) => println!(
                        "  scores: correctness={:.3} novelty={:.3} stability={:.3}",
                        scores.correctness, scores.novelty, scores.stability
                    ),
                    Err(e) => eprintln!("  score error: {e}"),
                }
            }
            Err(e) => eprintln!("  experiment failed: {e}"),
        }
        println!();
    }

    println!("=== DONE ===");
    println!("forge db: {}", db_path.display());
}
