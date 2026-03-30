//! # typed-patch
//!
//! Structured patch schema plus validation and apply helpers.
//!
//! This crate owns the typed patch representation, anchor/range edit model, and
//! deterministic validation/apply surface used by `forge-engine`. It does not
//! own execution backends or archive/promotion policy.

//! # typed-patch
//!
//! Structured patch model plus validation and application helpers.
//!
//! This Tier 3 crate turns edits into typed operations, validates them against
//! forge-policy constraints, and applies them through a workspace-safe
//! filesystem abstraction.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use forge_policy::{validate_forbidden_paths, validate_patch_caps, Violation, ViolationKind};
use sandbox_workspace::PatchFs;

#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("patch validation failed")]
    Validation { violations: Vec<Violation> },

    #[error("anchor resolution failed: {0}")]
    AnchorResolution(String),

    #[error("patch apply failed: {0}")]
    Apply(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("workspace error: {0}")]
    Workspace(#[from] sandbox_workspace::WorkspaceError),

    #[error("policy error: {0}")]
    Policy(#[from] forge_policy::PolicyError),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredPatch {
    pub patch_id: uuid::Uuid,
    pub summary: String,
    pub edits: Vec<FileEdit>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEdit {
    pub path: PathBuf,
    pub ops: Vec<EditOp>,
    pub mode: Option<FileMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileMode {
    Create,
    Delete,
    Modify,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EditOp {
    Insert {
        anchor: Anchor,
        lines: Vec<String>,
    },
    Delete {
        range: LineRange,
    },
    Replace {
        range: LineRange,
        lines: Vec<String>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Anchor {
    AfterLine {
        line: u32,
        context_before: Vec<String>,
        context_after: Vec<String>,
    },
    BeforeLine {
        line: u32,
        context_before: Vec<String>,
        context_after: Vec<String>,
    },
    AfterMatch {
        needle: String,
        occurrence: u32,
    },
    BeforeMatch {
        needle: String,
        occurrence: u32,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LineRange {
    pub start: u32,
    pub end_exclusive: u32,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub ok: bool,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone)]
pub struct PatchPolicy {
    pub forbidden_paths: Vec<String>,
    pub allow_test_modifications: bool,
    pub max_files_changed: usize,
    pub max_total_lines_changed: usize,
    pub max_lines_changed_per_file: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LineAttributionMap {
    pub mappings: BTreeMap<String, Vec<(u32, u32)>>,
    pub resolved_anchors: BTreeMap<(String, usize), u32>,
}

impl EditOp {
    pub fn lines_added(&self) -> usize {
        match self {
            Self::Insert { lines, .. } => lines.len(),
            Self::Delete { .. } => 0,
            Self::Replace { lines, .. } => lines.len(),
        }
    }

    pub fn lines_removed(&self) -> usize {
        match self {
            Self::Insert { .. } => 0,
            Self::Delete { range } | Self::Replace { range, .. } => {
                range.end_exclusive.saturating_sub(range.start) as usize
            }
        }
    }

    pub fn lines_changed(&self) -> usize {
        self.lines_added() + self.lines_removed()
    }

    pub fn kind(&self) -> &str {
        match self {
            Self::Insert { .. } => "Insert",
            Self::Delete { .. } => "Delete",
            Self::Replace { .. } => "Replace",
        }
    }

    pub fn anchor_kind(&self) -> Option<&str> {
        match self {
            Self::Insert {
                anchor: Anchor::AfterLine { .. },
                ..
            } => Some("AfterLine"),
            Self::Insert {
                anchor: Anchor::BeforeLine { .. },
                ..
            } => Some("BeforeLine"),
            Self::Insert {
                anchor: Anchor::AfterMatch { .. },
                ..
            } => Some("AfterMatch"),
            Self::Insert {
                anchor: Anchor::BeforeMatch { .. },
                ..
            } => Some("BeforeMatch"),
            _ => None,
        }
    }

    pub fn context_lines(&self) -> Vec<String> {
        match self {
            Self::Insert {
                anchor:
                    Anchor::AfterLine {
                        context_before,
                        context_after,
                        ..
                    }
                    | Anchor::BeforeLine {
                        context_before,
                        context_after,
                        ..
                    },
                ..
            } => {
                let mut context = context_before.clone();
                context.extend(context_after.iter().cloned());
                context
            }
            _ => Vec::new(),
        }
    }
}

impl FileEdit {
    pub fn total_lines_changed(&self) -> usize {
        self.ops.iter().map(EditOp::lines_changed).sum()
    }
}

impl StructuredPatch {
    pub fn files_changed(&self) -> usize {
        self.edits.len()
    }

    pub fn total_lines_changed(&self) -> usize {
        self.edits.iter().map(FileEdit::total_lines_changed).sum()
    }

    pub fn per_file_lines_changed(&self) -> Vec<usize> {
        self.edits
            .iter()
            .map(FileEdit::total_lines_changed)
            .collect()
    }

    pub fn file_paths(&self) -> Vec<&str> {
        self.edits
            .iter()
            .filter_map(|edit| edit.path.to_str())
            .collect()
    }
}

pub fn validate_patch(patch: &StructuredPatch, policy: &PatchPolicy) -> ValidationResult {
    let mut violations = Vec::new();

    if patch.edits.is_empty() {
        violations.push(Violation {
            kind: ViolationKind::EmptyPatch,
            message: "patch has no file edits".to_string(),
        });
    }

    for edit in &patch.edits {
        if edit.path.is_absolute() {
            violations.push(Violation {
                kind: ViolationKind::ForbiddenPath,
                message: format!(
                    "path '{}' is absolute — must be relative to workspace",
                    edit.path.display()
                ),
            });
        }

        for component in edit.path.components() {
            if component == std::path::Component::ParentDir {
                violations.push(Violation {
                    kind: ViolationKind::ForbiddenPath,
                    message: format!(
                        "path '{}' contains '..' — workspace escape not allowed",
                        edit.path.display()
                    ),
                });
                break;
            }
        }
    }

    let mut seen = BTreeSet::new();
    for edit in &patch.edits {
        let key = edit.path.to_string_lossy().to_string();
        if !seen.insert(key.clone()) {
            violations.push(Violation {
                kind: ViolationKind::DuplicatePath,
                message: format!("duplicate file path in patch: '{key}'"),
            });
        }
    }

    let path_strings: Vec<String> = patch
        .file_paths()
        .into_iter()
        .map(ToString::to_string)
        .collect();
    violations.extend(validate_forbidden_paths(
        &path_strings,
        &policy.forbidden_paths,
        policy.allow_test_modifications,
    ));
    violations.extend(validate_patch_caps(
        patch.files_changed(),
        patch.total_lines_changed(),
        &patch.per_file_lines_changed(),
        policy.max_files_changed,
        policy.max_total_lines_changed,
        policy.max_lines_changed_per_file,
    ));

    for edit in &patch.edits {
        if edit.ops.is_empty() && edit.mode != Some(FileMode::Delete) {
            violations.push(Violation {
                kind: ViolationKind::UselessEdit,
                message: format!(
                    "file '{}' has no ops and is not a deletion",
                    edit.path.display()
                ),
            });
        }

        for op in &edit.ops {
            match op {
                EditOp::Delete { range } | EditOp::Replace { range, .. } => {
                    if range.start < 1 {
                        violations.push(Violation {
                            kind: ViolationKind::DegenerateRange,
                            message: format!(
                                "file '{}': range start must be >= 1, got {}",
                                edit.path.display(),
                                range.start
                            ),
                        });
                    }
                    if range.end_exclusive <= range.start {
                        violations.push(Violation {
                            kind: ViolationKind::DegenerateRange,
                            message: format!(
                                "file '{}': degenerate range [{}, {})",
                                edit.path.display(),
                                range.start,
                                range.end_exclusive
                            ),
                        });
                    }
                }
                EditOp::Insert { anchor, .. } => match anchor {
                    Anchor::AfterMatch { occurrence, .. }
                    | Anchor::BeforeMatch { occurrence, .. } => {
                        if *occurrence == 0 {
                            violations.push(Violation {
                                kind: ViolationKind::InvalidOccurrence,
                                message: format!(
                                    "file '{}': match anchor occurrence is 0 (must be 1-indexed)",
                                    edit.path.display()
                                ),
                            });
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    ValidationResult {
        ok: violations.is_empty(),
        violations,
    }
}

pub fn apply_patch<F: PatchFs>(
    patch: &StructuredPatch,
    fs: &F,
) -> Result<LineAttributionMap, PatchError> {
    let mut snapshots: BTreeMap<String, Option<Vec<String>>> = BTreeMap::new();
    let mut attribution_map = LineAttributionMap::default();

    for edit in &patch.edits {
        let path_key = edit.path.to_string_lossy().to_string();
        snapshots.insert(path_key, fs.snapshot_lines(&edit.path)?);
    }

    for edit in &patch.edits {
        let path_key = edit.path.to_string_lossy().to_string();
        match apply_file_edit(edit, &edit.path, &path_key, &mut attribution_map, fs) {
            Ok(line_map) => {
                attribution_map.mappings.insert(path_key, line_map);
            }
            Err(error) => {
                for (path_key, snapshot) in &snapshots {
                    let relative = Path::new(path_key.as_str());
                    match snapshot {
                        Some(lines) => {
                            fs.write_lines(relative, lines)?;
                        }
                        None => {
                            fs.remove_file(relative)?;
                        }
                    }
                }
                return Err(error);
            }
        }
    }

    Ok(attribution_map)
}

pub async fn render_diff(original_dir: &Path, patched_dir: &Path) -> Result<String, PatchError> {
    let original = original_dir.to_path_buf();
    let patched = patched_dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        if let Ok(diff) = render_diff_git(&original, &patched) {
            return Ok(diff);
        }
        render_diff_internal(&original, &patched)
    })
    .await
    .map_err(|error| PatchError::Other(format!("render_diff join error: {error}")))?
}

fn apply_file_edit<F: PatchFs>(
    edit: &FileEdit,
    path: &Path,
    file_key: &str,
    attribution_map: &mut LineAttributionMap,
    fs: &F,
) -> Result<Vec<(u32, u32)>, PatchError> {
    let mode = edit.mode.as_ref().unwrap_or(&FileMode::Modify);

    match mode {
        FileMode::Delete => {
            fs.remove_file(path)?;
            return Ok(Vec::new());
        }
        FileMode::Create => {
            if fs.exists(path)? {
                return Err(PatchError::Apply(format!(
                    "cannot create {}: file already exists",
                    path.display()
                )));
            }
            fs.create_parent_dirs(path)?;
        }
        FileMode::Modify => {
            if !fs.exists(path)? {
                return Err(PatchError::Apply(format!(
                    "cannot modify {}: file does not exist",
                    path.display()
                )));
            }
        }
    }

    let mut content = if *mode == FileMode::Create {
        Vec::new()
    } else {
        fs.read_lines(path)?
    };

    let original_len = content.len();
    let mut offset: i64 = 0;
    let mut line_mappings = Vec::new();
    for index in 0..original_len {
        line_mappings.push((index as u32 + 1, index as u32 + 1));
    }

    for (op_index, op) in edit.ops.iter().enumerate() {
        match op {
            EditOp::Insert { anchor, lines } => {
                let insert_index =
                    resolve_anchor(anchor, &content).map_err(PatchError::AnchorResolution)?;

                if matches!(
                    anchor,
                    Anchor::AfterMatch { .. } | Anchor::BeforeMatch { .. }
                ) {
                    attribution_map
                        .resolved_anchors
                        .insert((file_key.to_string(), op_index), (insert_index + 1) as u32);
                }

                for (line_offset, line) in lines.iter().enumerate() {
                    content.insert(insert_index + line_offset, line.clone());
                }
                offset += lines.len() as i64;

                for mapping in &mut line_mappings {
                    if mapping.1 as usize > insert_index {
                        mapping.1 += lines.len() as u32;
                    }
                }
            }
            EditOp::Delete { range } => {
                let start = adjusted_index(range.start, offset, content.len())?;
                let end = adjusted_index(range.end_exclusive, offset, content.len())?;
                if start >= content.len() || end > content.len() || start >= end {
                    return Err(PatchError::Apply(format!(
                        "delete range [{}, {}) out of bounds (content has {} lines, offset={offset})",
                        range.start, range.end_exclusive, content.len()
                    )));
                }

                content.drain(start..end);
                let removed = (end - start) as i64;
                offset -= removed;

                for mapping in &mut line_mappings {
                    let mapped = mapping.1 as usize;
                    if mapped > end {
                        mapping.1 -= removed as u32;
                    } else if mapped > start {
                        mapping.1 = start as u32 + 1;
                    }
                }
            }
            EditOp::Replace { range, lines } => {
                let start = adjusted_index(range.start, offset, content.len())?;
                let end = adjusted_index(range.end_exclusive, offset, content.len())?;
                if start >= content.len() || end > content.len() || start >= end {
                    return Err(PatchError::Apply(format!(
                        "replace range [{}, {}) out of bounds (content has {} lines, offset={offset})",
                        range.start, range.end_exclusive, content.len()
                    )));
                }

                let removed = (end - start) as i64;
                let added = lines.len() as i64;

                content.drain(start..end);
                for (line_offset, line) in lines.iter().enumerate() {
                    content.insert(start + line_offset, line.clone());
                }

                let net = added - removed;
                offset += net;

                for mapping in &mut line_mappings {
                    let mapped = mapping.1 as usize;
                    if mapped > end {
                        mapping.1 = (mapped as i64 + net) as u32;
                    } else if mapped > start && mapped <= end {
                        mapping.1 = start as u32 + 1;
                    }
                }
            }
        }
    }

    fs.write_lines(path, &content)?;
    Ok(line_mappings)
}

fn adjusted_index(line: u32, offset: i64, content_len: usize) -> Result<usize, PatchError> {
    let adjusted = line as i64 - 1 + offset;
    if adjusted < 0 {
        return Err(PatchError::Apply(format!(
            "line offset underflow: line={line}, cumulative offset={offset}"
        )));
    }
    Ok((adjusted as usize).min(content_len))
}

fn resolve_anchor(anchor: &Anchor, content: &[String]) -> Result<usize, String> {
    match anchor {
        Anchor::AfterLine {
            line,
            context_before,
            context_after,
        } => {
            let index = *line as usize;
            if index > content.len() {
                return Err(format!(
                    "AfterLine {line} is beyond content length {}",
                    content.len()
                ));
            }
            if !verify_context(content, index, context_before, context_after) {
                return Err(format!("AfterLine {line}: context mismatch"));
            }
            Ok(index)
        }
        Anchor::BeforeLine {
            line,
            context_before,
            context_after,
        } => {
            let index = (*line as usize).saturating_sub(1);
            if index > content.len() {
                return Err(format!(
                    "BeforeLine {line} is beyond content length {}",
                    content.len()
                ));
            }
            if !verify_context(content, index, context_before, context_after) {
                return Err(format!("BeforeLine {line}: context mismatch"));
            }
            Ok(index)
        }
        Anchor::AfterMatch { needle, occurrence } => {
            let positions = find_all_matches(content, needle);
            let occurrence = *occurrence as usize;
            if occurrence == 0 {
                return Err("occurrence must be >= 1".to_string());
            }
            if positions.len() < occurrence {
                return Err(format!(
                    "AfterMatch: only {} matches for '{}', need occurrence {}",
                    positions.len(),
                    needle,
                    occurrence
                ));
            }
            Ok(positions[occurrence - 1] + 1)
        }
        Anchor::BeforeMatch { needle, occurrence } => {
            let positions = find_all_matches(content, needle);
            let occurrence = *occurrence as usize;
            if occurrence == 0 {
                return Err("occurrence must be >= 1".to_string());
            }
            if positions.len() < occurrence {
                return Err(format!(
                    "BeforeMatch: only {} matches for '{}', need occurrence {}",
                    positions.len(),
                    needle,
                    occurrence
                ));
            }
            Ok(positions[occurrence - 1])
        }
    }
}

fn verify_context(
    content: &[String],
    index: usize,
    context_before: &[String],
    context_after: &[String],
) -> bool {
    if context_before.is_empty() && context_after.is_empty() {
        return true;
    }

    for (context_index, line) in context_before.iter().enumerate() {
        let check = index as i64 - context_before.len() as i64 + context_index as i64;
        if check < 0 || check as usize >= content.len() {
            return false;
        }
        if content[check as usize].trim() != line.trim() {
            return false;
        }
    }

    for (context_index, line) in context_after.iter().enumerate() {
        let check = index + context_index;
        if check >= content.len() {
            return false;
        }
        if content[check].trim() != line.trim() {
            return false;
        }
    }

    true
}

fn find_all_matches(content: &[String], needle: &str) -> Vec<usize> {
    content
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(index, _)| index)
        .collect()
}

fn render_diff_git(original_dir: &Path, patched_dir: &Path) -> Result<String, PatchError> {
    let output = std::process::Command::new("git")
        .args(["diff", "--no-index", "--no-color", "--"])
        .arg(original_dir)
        .arg(patched_dir)
        .output()
        .map_err(|error| PatchError::Other(format!("git not available: {error}")))?;

    if output.status.code() == Some(0) {
        return Ok(String::new());
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.is_empty() && !output.stderr.is_empty() {
        return Err(PatchError::Other(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(diff)
}

fn render_diff_internal(original_dir: &Path, patched_dir: &Path) -> Result<String, PatchError> {
    let mut result = String::new();
    let mut files = BTreeSet::new();

    if original_dir.exists() {
        collect_files(original_dir, original_dir, &mut files)?;
    }
    if patched_dir.exists() {
        collect_files(patched_dir, patched_dir, &mut files)?;
    }

    for file in &files {
        let original_path = original_dir.join(file);
        let patched_path = patched_dir.join(file);

        let original_content = if original_path.exists() {
            std::fs::read_to_string(&original_path)?
        } else {
            String::new()
        };
        let patched_content = if patched_path.exists() {
            std::fs::read_to_string(&patched_path)?
        } else {
            String::new()
        };

        if original_content == patched_content {
            continue;
        }

        let diff = similar::TextDiff::from_lines(&original_content, &patched_content);
        let unified = diff
            .unified_diff()
            .context_radius(3)
            .header(&format!("a/{file}"), &format!("b/{file}"))
            .to_string();
        result.push_str(&unified);
    }

    Ok(result)
}

fn collect_files(base: &Path, dir: &Path, files: &mut BTreeSet<String>) -> Result<(), PatchError> {
    for entry in walkdir::WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(relative) = entry.path().strip_prefix(base) {
                files.insert(relative.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sandbox_workspace::LocalPatchFs;
    use tempfile::tempdir;

    #[test]
    fn strategy_helpers_count_lines() {
        let op = EditOp::Replace {
            range: LineRange {
                start: 1,
                end_exclusive: 3,
            },
            lines: vec!["x".to_string()],
        };
        assert_eq!(op.lines_added(), 1);
        assert_eq!(op.lines_removed(), 2);
    }

    fn permissive_policy() -> PatchPolicy {
        PatchPolicy {
            forbidden_paths: vec!["target".into()],
            allow_test_modifications: true,
            max_files_changed: 10,
            max_total_lines_changed: 100,
            max_lines_changed_per_file: 100,
        }
    }

    #[test]
    fn validate_patch_rejects_workspace_escape_and_invalid_occurrence() {
        let patch = StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "bad patch".into(),
            edits: vec![FileEdit {
                path: PathBuf::from("../escape.rs"),
                mode: Some(FileMode::Modify),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterMatch {
                        needle: "fn main".into(),
                        occurrence: 0,
                    },
                    lines: vec!["println!(\"oops\");".into()],
                }],
            }],
            notes: vec![],
        };

        let validation = validate_patch(&patch, &permissive_policy());
        assert!(!validation.ok);
        assert!(validation
            .violations
            .iter()
            .any(|violation| violation.kind == ViolationKind::ForbiddenPath));
        assert!(validation
            .violations
            .iter()
            .any(|violation| violation.kind == ViolationKind::InvalidOccurrence));
    }

    #[test]
    fn apply_patch_rolls_back_when_a_later_edit_fails() {
        let root = tempdir().unwrap();
        let fs = LocalPatchFs::new(root.path());
        let existing = Path::new("src/lib.rs");
        fs.write_lines(existing, &["fn main() {}".into()]).unwrap();

        let patch = StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "mixed patch".into(),
            edits: vec![
                FileEdit {
                    path: existing.into(),
                    mode: Some(FileMode::Modify),
                    ops: vec![EditOp::Replace {
                        range: LineRange {
                            start: 1,
                            end_exclusive: 2,
                        },
                        lines: vec!["fn main() { println!(\"changed\"); }".into()],
                    }],
                },
                FileEdit {
                    path: PathBuf::from("missing/file.rs"),
                    mode: Some(FileMode::Modify),
                    ops: vec![EditOp::Insert {
                        anchor: Anchor::AfterLine {
                            line: 1,
                            context_before: vec![],
                            context_after: vec![],
                        },
                        lines: vec!["let x = 1;".into()],
                    }],
                },
            ],
            notes: vec![],
        };

        let error = apply_patch(&patch, &fs).unwrap_err();
        assert!(matches!(error, PatchError::Apply(_)));
        assert_eq!(
            fs.read_lines(existing).unwrap(),
            vec!["fn main() {}".to_string()]
        );
    }

    #[test]
    fn idempotent_replace_patch_can_be_applied_twice() {
        let root = tempdir().unwrap();
        let fs = LocalPatchFs::new(root.path());
        let existing = Path::new("src/lib.rs");
        fs.write_lines(existing, &["fn main() {}".into()]).unwrap();

        let patch = StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "idempotent replace".into(),
            edits: vec![FileEdit {
                path: existing.into(),
                mode: Some(FileMode::Modify),
                ops: vec![EditOp::Replace {
                    range: LineRange {
                        start: 1,
                        end_exclusive: 2,
                    },
                    lines: vec!["fn main() {}".into()],
                }],
            }],
            notes: vec![],
        };

        apply_patch(&patch, &fs).unwrap();
        let once = fs.read_lines(existing).unwrap();
        apply_patch(&patch, &fs).unwrap();
        let twice = fs.read_lines(existing).unwrap();

        assert_eq!(once, twice);
        assert_eq!(twice, vec!["fn main() {}".to_string()]);
    }

    proptest! {
        #[test]
        fn parent_dir_paths_are_always_rejected(path_tail in "[A-Za-z0-9_./-]{1,24}") {
            let patch = StructuredPatch {
                patch_id: uuid::Uuid::new_v4(),
                summary: "escape".into(),
                edits: vec![FileEdit {
                    path: PathBuf::from(format!("../{path_tail}")),
                    mode: Some(FileMode::Modify),
                    ops: vec![EditOp::Delete {
                        range: LineRange { start: 1, end_exclusive: 2 },
                    }],
                }],
                notes: vec![],
            };

            let validation = validate_patch(&patch, &permissive_policy());
            prop_assert!(!validation.ok);
            prop_assert!(validation
                .violations
                .iter()
                .any(|violation| violation.kind == ViolationKind::ForbiddenPath));
        }

        #[test]
        fn idempotent_replace_patch_remains_stable_for_arbitrary_line(line in "[A-Za-z0-9_ (){};]{1,32}") {
            let root = tempdir().unwrap();
            let fs = LocalPatchFs::new(root.path());
            let existing = Path::new("src/lib.rs");
            fs.write_lines(existing, &["fn main() {}".into()]).unwrap();

            let patch = StructuredPatch {
                patch_id: uuid::Uuid::new_v4(),
                summary: "idempotent replace".into(),
                edits: vec![FileEdit {
                    path: existing.into(),
                    mode: Some(FileMode::Modify),
                    ops: vec![EditOp::Replace {
                        range: LineRange {
                            start: 1,
                            end_exclusive: 2,
                        },
                        lines: vec![line.clone()],
                    }],
                }],
                notes: vec![],
            };

            apply_patch(&patch, &fs).unwrap();
            let once = fs.read_lines(existing).unwrap();
            apply_patch(&patch, &fs).unwrap();
            let twice = fs.read_lines(existing).unwrap();

            prop_assert_eq!(once, twice.clone());
            prop_assert_eq!(twice, vec![line]);
        }
    }
}
