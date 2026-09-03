//! Multi-strategy experiment runner — produces diverse CEA edges by targeting
//! different scope types (fn, struct, impl, mod) with context-aware patches.
//!
//! Usage: cargo run --example run_multi -- <fixture-dir> <forge-db-path>


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
use forge_engine::runtime::patch::types::{
    Anchor, EditOp, FileEdit, FileMode, LineRange, StructuredPatch,
};
use forge_engine::store::ForgeStore;
use std::path::PathBuf;

/// A patch strategy targeting a specific scope type for CEA diversity
struct Strategy {
    name: &'static str,
    file: &'static str,
    op_kind: &'static str, // "insert" | "delete" | "replace"
    target_line: u32,
    context: &'static str, // code context hinting at scope (e.g., "fn foo() {", "struct Bar {")
}

const STRATEGIES: &[Strategy] = &[
    // Different op kinds on module-level
    Strategy {
        name: "insert-mod",
        file: "src/lib.rs",
        op_kind: "insert",
        target_line: 1,
        context: "//! Module-level doc comment",
    },
    Strategy {
        name: "delete-mod",
        file: "src/lib.rs",
        op_kind: "delete",
        target_line: 1,
        context: "//! Module-level doc comment",
    },
    Strategy {
        name: "replace-mod",
        file: "src/lib.rs",
        op_kind: "replace",
        target_line: 1,
        context: "//! Module-level doc comment",
    },
    // Different file paths (only for cea-core which has submodules)
    Strategy {
        name: "insert-attrib",
        file: "src/attribution.rs",
        op_kind: "insert",
        target_line: 1,
        context: "fn attribute_effects",
    },
    Strategy {
        name: "insert-types",
        file: "src/types.rs",
        op_kind: "insert",
        target_line: 1,
        context: "pub struct EditOpSignature",
    },
    Strategy {
        name: "insert-graph",
        file: "src/graph.rs",
        op_kind: "insert",
        target_line: 1,
        context: "pub struct CausalGraph",
    },
];

fn build_patch(s: &Strategy) -> StructuredPatch {
    let context_lines: Vec<String> = s.context.lines().map(|l| l.to_string()).collect();
    let ops = match s.op_kind {
        "delete" => vec![EditOp::Delete {
            range: LineRange {
                start: s.target_line,
                end_exclusive: s.target_line + 1,
            },
        }],
        "replace" => vec![EditOp::Replace {
            range: LineRange {
                start: s.target_line,
                end_exclusive: s.target_line + 1,
            },
            lines: vec![format!("// replace: {}", s.name)],
        }],
        _ => vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: s.target_line,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec![format!("// multi: {}", s.name)],
        }],
    };

    StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: format!("{}: {}", s.name, s.op_kind),
        edits: vec![FileEdit {
            path: PathBuf::from(s.file),
            ops,
            mode: Some(FileMode::Modify),
        }],
        notes: vec![format!("scope:{}", s.context)],
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

    // Create forge DB if it doesn't exist yet (schema will be created on first write)
    let store = ForgeStore::open(&db_path).expect("open forge db");
    let mut config = ForgeConfig::default();
    config.sealed_allow_host_backend = true;

    let suite = load_suite(&fixture_dir).expect("load fixture suite");
    println!(
        "Suite: {} ({} tasks, {} strategies)\n",
        suite.name,
        suite.tasks.len(),
        STRATEGIES.len()
    );

    let backend = HostBackend::new(&config);
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);
    let experiment_config = ExperimentConfig::default();

    let mut total_edges = 0u64;
    let mut total_no_triples = 0u64;
    let mut total_dups = 0u64;

    for task in &suite.tasks {
        eprintln!(
            "DEBUG: task={} path={}",
            task.task.task_id,
            task.fixture_path.display()
        );
        for s in STRATEGIES {
            let target_file = task.fixture_path.join(s.file);
            if !target_file.exists() {
                continue; // submodule files only in cea-core
            }

            let patch = build_patch(s);
            let version_id = "multi-v1";

            match runner
                .run(&task.fixture_path, &patch, &experiment_config)
                .await
            {
                Ok(experiment) => {
                    let line_map = LineAttributionMap::default();
                    let attributed =
                        attribute_effects(&patch, &experiment.patched_result, &line_map, 12)
                            .unwrap_or_else(|_| Vec::new());

                    if !attributed.is_empty() {
                        let scope = attributed
                            .first()
                            .map(|t| format!("{:?}", t.cause.scope_tag))
                            .unwrap_or_default();
                        let run_result = forge_engine::AttributedRunResult::new(
                            attributed,
                            experiment.patched_result.clone(),
                        );
                        match update_graph(
                            &store,
                            &run_result,
                            &task.task.task_id,
                            version_id,
                            &config,
                        ) {
                            Ok(update) => match update {
                                forge_engine::cea::store::UpdateResult::Applied {
                                    edges_added,
                                    ..
                                } => {
                                    total_edges += edges_added as u64;
                                    println!(
                                        "  ✓ {}:{} (scope={}, edges={})",
                                        task.task.task_id, s.name, scope, edges_added
                                    );
                                }
                                forge_engine::cea::store::UpdateResult::AlreadyProcessed => {
                                    total_dups += 1;
                                }
                            },
                            Err(e) => eprintln!("  ✗ {}:{} CEA: {}", task.task.task_id, s.name, e),
                        }
                    } else {
                        total_no_triples += 1;
                    }
                }
                Err(e) => eprintln!("  ✗ {}:{} run: {}", task.task.task_id, s.name, e),
            }
        }
    }

    println!("\n=== DONE ===");
    println!(
        "edges: {}  no-triples: {}  dups: {}",
        total_edges, total_no_triples, total_dups
    );
}
