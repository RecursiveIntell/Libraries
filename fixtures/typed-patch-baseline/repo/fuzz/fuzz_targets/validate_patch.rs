#![no_main]

use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;
use typed_patch::{validate_patch, Anchor, EditOp, FileEdit, FileMode, PatchPolicy, StructuredPatch};

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let lines: Vec<String> = text.lines().take(8).map(|line| line.to_string()).collect();
    let patch = StructuredPatch {
        patch_id: uuid::Uuid::new_v4(),
        summary: text.chars().take(32).collect(),
        edits: vec![FileEdit {
            path: PathBuf::from(text.chars().take(32).collect::<String>()),
            mode: Some(FileMode::Modify),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: Vec::new(),
                    context_after: Vec::new(),
                },
                lines,
            }],
        }],
        notes: vec![],
    };

    let policy = PatchPolicy {
        forbidden_paths: vec!["target".into()],
        allow_test_modifications: true,
        max_files_changed: 8,
        max_total_lines_changed: 256,
        max_lines_changed_per_file: 128,
    };

    let _ = validate_patch(&patch, &policy);
});
