// Seed a CEA database with a risky cause->error edge (multiple observations)
// so the predict risk-flag gate (>= 5 effective samples) fires, then print
// the exact signature JSON needed to trigger the risk flag.
use cea_core::{AnchorKind, EditOpKind, EditOpSignature, FileIndex, OpIndex, ScopeTag};
use check_runner::EffectSignature;

fn main() {
    let db = std::env::args().nth(1).expect("db path");
    let path = std::path::Path::new(&db);
    // Fresh DB each run.
    let _ = std::fs::remove_file(path);
    let store = cea_sqlite::SqliteCeaStore::open(path).expect("open store");

    let sig = EditOpSignature {
        op_kind: EditOpKind::Replace,
        anchor_kind: AnchorKind::Range,
        lines_added: 1,
        lines_removed: 0,
        context_hash: "".to_string(),
        file_extension: "rs".to_string(),
        scope_tag: ScopeTag::Unknown,
        op_index: OpIndex(0),
        file_index: FileIndex(0),
    };
    let effect = EffectSignature {
        check_kind: "clippy".to_string(),
        outcome: "error".to_string(),
        severity: "error".to_string(),
        message_class: "risky_edit".to_string(),
        line_offset_from_edit: Some(1),
    };

    // 40 runs with DISTINCT distances so run hashes all differ; each adds a
    // positive observation for the same cause->effect edge. This pushes the
    // sample-growth curve (1-exp(-n/8)) above the 0.65 risk threshold.
    for i in 0..40u32 {
        let triple = cea_core::AttributionTriple {
            cause: sig.clone(),
            effect: effect.clone(),
            distance: (1 + i) as i32,
            weight: 1.0,
        };
        let check = check_runner::CheckResult {
            fmt_pass: true,
            clippy_pass: false,
            test_pass: false,
            fmt_output: check_runner::ParsedCheckOutput::default(),
            clippy_output: check_runner::ParsedCheckOutput::default(),
            test_output: check_runner::ParsedCheckOutput::default(),
            total_duration_ms: 1,
        };
        let run = cea_core::AttributedRunResult::new(vec![triple], check);
        // decay_factor here is the DISTANCE decay constant (forge-engine
        // default is 10.0), not the graph edge decay.
        let result =
            cea_store::update_graph(&store, &run, "eval-seed", "v1", 10.0).expect("update");
        eprintln!("run {i}: {result:?}");
    }

    println!("{}", serde_json::to_string(&sig).unwrap());
}
