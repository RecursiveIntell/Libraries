//! Scope-diverse experiment runner — targets specific code patterns (fn, struct,
//! enum, impl, trait) to produce CEA edges with DIFFERENT scope tags.
//!
//! Uses context_after to guide the CEA scope inference toward the desired tag.
//!
//! Usage: cargo run --example run_scope -- <fixture-dir> <forge-db-path>

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
use forge_engine::lab::suite::load_suite;
use forge_engine::runtime::patch::apply::LineAttributionMap;
use forge_engine::runtime::patch::types::{Anchor, EditOp, FileEdit, FileMode, StructuredPatch};
use forge_engine::store::ForgeStore;
use std::path::PathBuf;

/// A scope-targeting patch specification
struct ScopeTarget {
    name: &'static str,
    file: &'static str,
    target_line: u32,
    /// Context lines AFTER the anchor point — MUST contain the scope keyword
    context: &'static [&'static str],
    op_kind: &'static str, // "insert" | "replace"
}

const TARGETS: &[ScopeTarget] = &[
    // fn scope — edit after a pub fn declaration
    ScopeTarget {
        name: "fn-attrib",
        file: "src/attribution.rs",
        target_line: 16,
        context: &["pub struct AttributionTriple {"],
        op_kind: "insert",
    },
    ScopeTarget {
        name: "fn-predict",
        file: "src/predict.rs",
        target_line: 32,
        context: &["pub fn predict("],
        op_kind: "insert",
    },
    // struct scope
    ScopeTarget {
        name: "struct-graph",
        file: "src/graph.rs",
        target_line: 18,
        context: &["pub struct EdgeStats {"],
        op_kind: "insert",
    },
    // enum scope
    ScopeTarget {
        name: "enum-error",
        file: "src/error.rs",
        target_line: 2,
        context: &["pub enum CeaCoreError {"],
        op_kind: "insert",
    },
    // impl scope
    ScopeTarget {
        name: "impl-stats",
        file: "src/graph.rs",
        target_line: 37,
        context: &["impl EdgeStats {"],
        op_kind: "replace",
    },
    // trait scope (via mod in lib.rs with context)
    ScopeTarget {
        name: "mod-lib",
        file: "src/lib.rs",
        target_line: 12,
        context: &["mod attribution;"],
        op_kind: "insert",
    },
    // Another fn scope with replace
    ScopeTarget {
        name: "fn-cal-replace",
        file: "src/calibration.rs",
        target_line: 53,
        context: &["pub fn advisory_confidence("],
        op_kind: "replace",
    },
    // macro_rules scope
    ScopeTarget {
        name: "macro-delete",
        file: "src/types.rs",
        target_line: 1,
        context: &[],
        op_kind: "delete",
    },
];

fn build_patch(t: &ScopeTarget) -> StructuredPatch {
    let context_after: Vec<String> = t.context.iter().map(|s| s.to_string()).collect();

    let ops = match t.op_kind {
        "replace" => vec![EditOp::Replace {
            range: forge_engine::runtime::patch::types::LineRange {
                start: t.target_line,
                end_exclusive: t.target_line + 1,
            },
            lines: vec![format!("// scope-patch: {}", t.name)],
        }],
        "delete" => vec![EditOp::Delete {
            range: forge_engine::runtime::patch::types::LineRange {
                start: t.target_line,
                end_exclusive: t.target_line + 1,
            },
        }],
        _ => vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: t.target_line,
                context_before: vec![],
                context_after,
            },
            lines: vec![format!("// scope-patch: {}", t.name)],
        }],
    };

    StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: format!("{}:{}", t.name, t.op_kind),
        edits: vec![FileEdit {
            path: PathBuf::from(t.file),
            ops,
            mode: Some(FileMode::Modify),
        }],
        notes: vec![format!("scope:{}:{}", t.name, t.op_kind)],
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
    let config = ForgeConfig {
        sealed_allow_host_backend: true,
        ..Default::default()
    };

    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!(
        "Fixture: {}  Tasks: {}  Targets: {}\n",
        fixture_dir.display(),
        suite.tasks.len(),
        TARGETS.len()
    );

    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);
    let experiment_config = ExperimentConfig::default();

    let mut total_edges: u64 = 0;

    for task in &suite.tasks {
        // Only cea-core has submodule files
        if task.task.task_id != "cea-core-baseline" {
            continue;
        }

        for t in TARGETS {
            let target_file = task.fixture_path.join(t.file);
            if !target_file.exists() {
                println!("  skip {}: file not found", t.name);
                continue;
            }

            let patch = build_patch(t);
            let version_id = format!("scope-{}", t.name);

            match runner
                .run(&task.fixture_path, &patch, &experiment_config)
                .await
            {
                Ok(experiment) => {
                    let line_map = LineAttributionMap::default();
                    let attributed =
                        attribute_effects(&patch, &experiment.patched_result, &line_map, 12)
                            .unwrap_or_else(|_| Vec::new());

                    if attributed.is_empty() {
                        println!(
                            "  {}: no triples (r={} i={})",
                            t.name, experiment.diff.regressions, experiment.diff.improvements
                        );
                        continue;
                    }

                    let scope = attributed
                        .first()
                        .map(|triple| format!("{:?}", triple.cause.scope_tag))
                        .unwrap_or_default();

                    let run_result = forge_engine::AttributedRunResult::new(
                        attributed,
                        experiment.patched_result.clone(),
                    );

                    match update_graph(
                        &store,
                        &run_result,
                        &task.task.task_id,
                        &version_id,
                        &config,
                    ) {
                        Ok(update) => match update {
                            forge_engine::cea::store::UpdateResult::Applied {
                                edges_added, ..
                            } => {
                                total_edges += edges_added as u64;
                                println!("  ✓ {}: scope={} edges={}", t.name, scope, edges_added);
                            }
                            forge_engine::cea::store::UpdateResult::AlreadyProcessed => {
                                println!("  {}: (dup)", t.name);
                            }
                        },
                        Err(e) => eprintln!("  {}: CEA err: {}", t.name, e),
                    }
                }
                Err(e) => println!("  {}: run err: {:.80}", t.name, e),
            }
        }
    }

    println!("\n=== DONE ===  total edges: {}", total_edges);
}
