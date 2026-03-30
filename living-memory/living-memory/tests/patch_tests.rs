//! C. StructuredPatch validation tests (C1–C6)
//! D. StructuredPatch apply tests (D1–D7)
//! E. Diff rendering tests (E1–E4)

use std::path::PathBuf;

use forge_engine::*;
use tempfile::TempDir;
use uuid::Uuid;

fn default_config() -> ForgeConfig {
    ForgeConfig::default()
}

fn make_patch(edits: Vec<FileEdit>) -> StructuredPatch {
    StructuredPatch {
        patch_id: Uuid::new_v4(),
        summary: "test patch".to_string(),
        edits,
        notes: vec![],
    }
}

// === C. Validation tests ===

/// C1: patch_rejects_forbidden_path_tests
#[test]
fn c1_patch_rejects_forbidden_path_tests() {
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("tests/my_test.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["// test".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = validate_patch(&patch, &default_config());
    assert!(!result.ok);
    assert!(result
        .violations
        .iter()
        .any(|v| v.kind == ViolationKind::ForbiddenPath));
}

/// C2: patch_rejects_forbidden_path_snap
#[test]
fn c2_patch_rejects_forbidden_path_snap() {
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/snapshots/foo.snap"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["snapshot".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = validate_patch(&patch, &default_config());
    assert!(!result.ok);
    assert!(result
        .violations
        .iter()
        .any(|v| v.kind == ViolationKind::ForbiddenPath));
}

/// C3: patch_rejects_cap_files
#[test]
fn c3_patch_rejects_cap_files() {
    let edits: Vec<FileEdit> = (0..9)
        .map(|i| FileEdit {
            path: PathBuf::from(format!("src/file_{i}.rs")),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change".to_string()],
            }],
            mode: Some(FileMode::Modify),
        })
        .collect();

    let patch = make_patch(edits);
    let result = validate_patch(&patch, &default_config());
    assert!(!result.ok);
    assert!(result
        .violations
        .iter()
        .any(|v| v.kind == ViolationKind::CapExceeded));
}

/// C4: patch_rejects_cap_total_lines
#[test]
fn c4_patch_rejects_cap_total_lines() {
    // Create a patch with 401 total lines changed
    let lines: Vec<String> = (0..401).map(|i| format!("line {i}")).collect();
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/big.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines,
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = validate_patch(&patch, &default_config());
    assert!(!result.ok);
    assert!(result
        .violations
        .iter()
        .any(|v| { v.kind == ViolationKind::CapExceeded && v.message.contains("total lines") }));
}

/// C5: patch_accepts_well_formed
#[test]
fn c5_patch_accepts_well_formed() {
    let patch = make_patch(vec![
        FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// comment".to_string()],
            }],
            mode: Some(FileMode::Modify),
        },
        FileEdit {
            path: PathBuf::from("src/main.rs"),
            ops: vec![EditOp::Replace {
                range: LineRange {
                    start: 1,
                    end_exclusive: 2,
                },
                lines: vec!["fn main() {}".to_string()],
            }],
            mode: Some(FileMode::Modify),
        },
    ]);

    let result = validate_patch(&patch, &default_config());
    assert!(
        result.ok,
        "Expected valid patch, got: {:?}",
        result.violations
    );
}

/// C6: patch_validation_returns_all_violations (not fail-fast)
#[test]
fn c6_patch_validation_returns_all_violations() {
    // Patch with 3 violations: forbidden path, cap exceeded, degenerate range
    let lines: Vec<String> = (0..201).map(|i| format!("line {i}")).collect();
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("tests/bad.rs"),
        ops: vec![
            EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines,
            },
            EditOp::Delete {
                range: LineRange {
                    start: 5,
                    end_exclusive: 3, // degenerate!
                },
            },
        ],
        mode: Some(FileMode::Modify),
    }]);

    let result = validate_patch(&patch, &default_config());
    assert!(!result.ok);
    // Should have at least 3 violations (forbidden path, per-file cap, degenerate range)
    assert!(
        result.violations.len() >= 3,
        "Expected at least 3 violations, got {}: {:?}",
        result.violations.len(),
        result.violations
    );
}

/// C7: patch_rejects_path_traversal
/// Patch with path containing ".." or absolute path → must fail with ForbiddenPath.
#[test]
fn c7_patch_rejects_path_traversal() {
    // Test 1: path with ".." component
    let patch_dotdot = make_patch(vec![FileEdit {
        path: PathBuf::from("../../../etc/passwd"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["pwned".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);
    let result = validate_patch(&patch_dotdot, &default_config());
    assert!(!result.ok);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::ForbiddenPath && v.message.contains("..")),
        "Expected ForbiddenPath for '..' path: {:?}",
        result.violations
    );

    // Test 2: absolute path
    let patch_abs = make_patch(vec![FileEdit {
        path: PathBuf::from("/etc/shadow"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["pwned".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);
    let result = validate_patch(&patch_abs, &default_config());
    assert!(!result.ok);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::ForbiddenPath && v.message.contains("absolute")),
        "Expected ForbiddenPath for absolute path: {:?}",
        result.violations
    );

    // Test 3: duplicate file paths in same patch
    let patch_dup = make_patch(vec![
        FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change 1".to_string()],
            }],
            mode: Some(FileMode::Modify),
        },
        FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 2,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["// change 2".to_string()],
            }],
            mode: Some(FileMode::Modify),
        },
    ]);
    let result = validate_patch(&patch_dup, &default_config());
    assert!(!result.ok);
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::DuplicatePath),
        "Expected DuplicatePath violation: {:?}",
        result.violations
    );
}

// === D. Apply tests ===

fn setup_workspace(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (name, content) in files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }
    dir
}

/// D1: apply_insert_after_line
#[test]
fn d1_apply_insert_after_line() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\nline 3\nline 4\n")]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 3,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["inserted line A".to_string(), "inserted line B".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_ok(), "Apply failed: {:?}", result.err());

    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[1], "line 2");
    assert_eq!(lines[2], "line 3");
    assert_eq!(lines[3], "inserted line A");
    assert_eq!(lines[4], "inserted line B");
    assert_eq!(lines[5], "line 4");
}

/// D2: apply_replace_range
#[test]
fn d2_apply_replace_range() {
    let ws = setup_workspace(&[(
        "src/lib.rs",
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\n",
    )]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Replace {
            range: LineRange {
                start: 5,
                end_exclusive: 8,
            },
            lines: vec!["replaced A".to_string(), "replaced B".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_ok(), "Apply failed: {:?}", result.err());

    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[3], "line 4");
    assert_eq!(lines[4], "replaced A");
    assert_eq!(lines[5], "replaced B");
    assert_eq!(lines[6], "line 8");
}

/// D3: apply_delete_range
#[test]
fn d3_apply_delete_range() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\nline 3\nline 4\nline 5\n")]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Delete {
            range: LineRange {
                start: 2,
                end_exclusive: 4,
            },
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_ok(), "Apply failed: {:?}", result.err());

    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[1], "line 4");
    assert_eq!(lines[2], "line 5");
}

/// D4: apply_match_anchor
#[test]
fn d4_apply_match_anchor() {
    let ws = setup_workspace(&[(
        "src/lib.rs",
        "fn compute() {}\nfn helper() {}\nfn compute() {}\nfn done() {}\n",
    )]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterMatch {
                needle: "fn compute".to_string(),
                occurrence: 2,
            },
            lines: vec!["// inserted after 2nd fn compute".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_ok(), "Apply failed: {:?}", result.err());

    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // The 2nd "fn compute" is at index 2 (0-indexed), insert after it at index 3
    assert_eq!(lines[3], "// inserted after 2nd fn compute");
}

/// D5: apply_fails_on_ambiguous_context
#[test]
fn d5_apply_fails_on_ambiguous_context() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\nline 3\n")]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 2,
                context_before: vec!["WRONG CONTEXT".to_string()],
                context_after: vec![],
            },
            lines: vec!["should not appear".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_err(), "Expected failure on context mismatch");

    // Workspace should be unchanged
    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "line 1\nline 2\nline 3\n");
}

/// D6: apply_is_atomic
#[test]
fn d6_apply_is_atomic() {
    let ws = setup_workspace(&[
        ("src/lib.rs", "line 1\nline 2\nline 3\n"),
        ("src/other.rs", "other 1\nother 2\n"),
    ]);

    // First edit succeeds, second edit fails (bad anchor on other.rs)
    let patch = make_patch(vec![
        FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Insert {
                anchor: Anchor::AfterLine {
                    line: 1,
                    context_before: vec![],
                    context_after: vec![],
                },
                lines: vec!["added".to_string()],
            }],
            mode: Some(FileMode::Modify),
        },
        FileEdit {
            path: PathBuf::from("src/other.rs"),
            ops: vec![EditOp::Delete {
                range: LineRange {
                    start: 10,
                    end_exclusive: 20,
                },
            }],
            mode: Some(FileMode::Modify),
        },
    ]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_err(), "Expected atomic failure");

    // First file should be restored
    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "line 1\nline 2\nline 3\n");
}

/// D7: apply_returns_line_attribution_map
#[test]
fn d7_apply_returns_line_attribution_map() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\nline 3\n")]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["new".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path()).unwrap();
    assert!(result.mappings.contains_key("src/lib.rs"));
    let map = &result.mappings["src/lib.rs"];
    assert!(!map.is_empty());
}

/// D8: apply_rejects_path_traversal
/// Patch with ".." path must fail in apply_patch.
#[test]
fn d8_apply_rejects_path_traversal() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\n")]);

    // Try to apply a patch that escapes the workspace
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("../../../etc/evil.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 0,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["// evil".to_string()],
        }],
        mode: Some(FileMode::Create),
    }]);

    let result = apply_patch(&patch, ws.path());
    assert!(result.is_err(), "apply_patch must reject path with '..'");
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains(".."),
        "Error message should mention '..': {err}"
    );

    // Verify workspace is unchanged
    let content = std::fs::read_to_string(ws.path().join("src/lib.rs")).unwrap();
    assert_eq!(content, "line 1\nline 2\n");
}

// === E. Diff rendering tests ===

/// E1: diff_render_produces_valid_unified_format
#[tokio::test]
async fn e1_diff_render_produces_valid_unified_format() {
    let original = TempDir::new().unwrap();
    let patched = TempDir::new().unwrap();

    std::fs::write(original.path().join("file.rs"), "line 1\nline 2\n").unwrap();
    std::fs::write(patched.path().join("file.rs"), "line 1\nnew line\nline 2\n").unwrap();

    let diff = render_diff(original.path(), patched.path()).await.unwrap();
    assert!(!diff.is_empty(), "Diff should not be empty");
    // Check for unified diff markers
    assert!(
        diff.contains("---") && diff.contains("+++") && diff.contains("@@"),
        "Diff should be valid unified format: {diff}"
    );
}

/// E2: diff_render_is_stable
#[tokio::test]
async fn e2_diff_render_is_stable() {
    let original = TempDir::new().unwrap();
    let patched = TempDir::new().unwrap();

    std::fs::write(original.path().join("file.rs"), "a\nb\nc\n").unwrap();
    std::fs::write(patched.path().join("file.rs"), "a\nX\nc\n").unwrap();

    let diff1 = render_diff(original.path(), patched.path()).await.unwrap();
    let diff2 = render_diff(original.path(), patched.path()).await.unwrap();
    assert_eq!(diff1, diff2, "Diff should be stable across calls");
}

/// E4: diff_fallback_renders_correctly (test internal diff)
#[tokio::test]
async fn e4_diff_fallback_renders_correctly() {
    // We test the internal diff by using the public render_diff function.
    // Even if git is available, the output should be parseable.
    let original = TempDir::new().unwrap();
    let patched = TempDir::new().unwrap();

    std::fs::write(original.path().join("test.rs"), "fn main() {\n}\n").unwrap();
    std::fs::write(
        patched.path().join("test.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let diff = render_diff(original.path(), patched.path()).await.unwrap();
    assert!(!diff.is_empty());
    // Should contain the added line
    assert!(
        diff.contains("println") || diff.contains("+"),
        "Diff should contain the change"
    );
}

// === Phase 1 regression tests ===

/// Insert after line 1 should NOT shift line 1's mapping.
#[test]
fn insert_after_line_1_does_not_shift_line_1_mapping() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\nline 3\n")]);

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["new A".to_string(), "new B".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path()).unwrap();
    let map = &result.mappings["src/lib.rs"];

    // Original line 1 (mapping.0 == 1) should still map to patched line 1
    let line1_mapping = map.iter().find(|(orig, _)| *orig == 1).unwrap();
    assert_eq!(
        line1_mapping.1, 1,
        "line 1 should NOT be shifted by insert after line 1, got mapped to {}",
        line1_mapping.1
    );

    // Original lines 2 and 3 should be shifted by +2
    let line2_mapping = map.iter().find(|(orig, _)| *orig == 2).unwrap();
    assert_eq!(
        line2_mapping.1, 4,
        "line 2 should shift to 4 (2 + 2 inserted), got {}",
        line2_mapping.1
    );

    let line3_mapping = map.iter().find(|(orig, _)| *orig == 3).unwrap();
    assert_eq!(
        line3_mapping.1, 5,
        "line 3 should shift to 5 (3 + 2 inserted), got {}",
        line3_mapping.1
    );
}

/// Negative offset from cascading deletes should produce an error, not wrap.
#[test]
fn negative_offset_returns_error_not_wrap() {
    let ws = setup_workspace(&[("src/lib.rs", "line 1\nline 2\n")]);

    // Delete lines 1-2 (removes all content), then try to delete line 1 again.
    // The second delete should fail because offset makes the start negative.
    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/lib.rs"),
        ops: vec![
            EditOp::Delete {
                range: LineRange {
                    start: 1,
                    end_exclusive: 3,
                },
            },
            EditOp::Delete {
                range: LineRange {
                    start: 1,
                    end_exclusive: 2,
                },
            },
        ],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, ws.path());
    // Should either succeed gracefully or return an error — never silently corrupt
    // The second delete on an empty file should produce an out-of-bounds error
    assert!(
        result.is_err(),
        "Second delete on empty file should fail: {:?}",
        result
    );
}

// === Phase 2 regression tests ===

/// Patch must reject symlink files in workspace.
#[cfg(unix)]
#[test]
fn patch_rejects_symlink_file_in_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path();
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    // Create a real file at the target so the symlink is "valid"
    let target = dir.path().join("outside.rs");
    std::fs::write(&target, "original content\n").unwrap();
    // Create a symlink: workspace/src/link.rs → outside.rs
    std::os::unix::fs::symlink(&target, workspace.join("src/link.rs")).unwrap();

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("src/link.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["// injected".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, workspace);
    assert!(result.is_err(), "Should reject symlink file");
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("symlink"),
        "Error should mention symlink: {err}"
    );
}

/// Patch must reject symlink directories in workspace.
#[cfg(unix)]
#[test]
fn patch_rejects_symlink_directory_in_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path();
    std::fs::create_dir_all(workspace.join("real_dir")).unwrap();
    // Create a target directory with a file
    let target_dir = dir.path().join("target_dir");
    std::fs::create_dir_all(&target_dir).unwrap();
    std::fs::write(target_dir.join("file.rs"), "content\n").unwrap();
    // Create a symlink directory: workspace/sneaky_dir → target_dir
    std::os::unix::fs::symlink(&target_dir, workspace.join("sneaky_dir")).unwrap();

    let patch = make_patch(vec![FileEdit {
        path: PathBuf::from("sneaky_dir/file.rs"),
        ops: vec![EditOp::Insert {
            anchor: Anchor::AfterLine {
                line: 1,
                context_before: vec![],
                context_after: vec![],
            },
            lines: vec!["// injected".to_string()],
        }],
        mode: Some(FileMode::Modify),
    }]);

    let result = apply_patch(&patch, workspace);
    assert!(result.is_err(), "Should reject symlink directory");
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("symlink") || err.contains("escapes workspace"),
        "Error should mention symlink or escape: {err}"
    );
}
