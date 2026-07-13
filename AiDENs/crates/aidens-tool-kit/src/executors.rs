use crate::dispatcher::ReceiptBearingToolFailure;
use crate::patch::{apply_single_replacement, parse_simple_unified_diff, touched_paths_from_diff};
use crate::sandbox::{
    collect_search_matches, display_sandbox_path, path_is_denied_by_prefix, reject_hardlinked_file,
    resolve_existing_sandboxed_path, resolve_target_sandboxed_path,
};
use aidens_contracts::{
    ArtifactId, CommandRunReportV1, DisplayDigestV1, PatchApplyReportV1, PatchProposalV1,
    RepoListEntryV1, RepoListReportV1, RepoReadReportV1,
};
use anyhow::{anyhow, bail, Context};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) fn repo_read(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("repo-read input requires string field 'path'"))?;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let metadata = std::fs::metadata(&resolved)
        .with_context(|| format!("repo-read cannot stat {}", resolved.display()))?;
    if !metadata.is_file() {
        bail!("repo-read path is not a file: {}", relative)
    }
    reject_hardlinked_file(&resolved, &metadata)?;
    if metadata.len() > 1_048_576 {
        bail!(
            "repo-read refuses files larger than 1048576 bytes: {}",
            relative
        )
    }
    let content = std::fs::read_to_string(&resolved)
        .with_context(|| format!("repo-read failed to read {}", relative))?;
    let display_path = display_sandbox_path(sandbox_root, &resolved);
    let read_receipt = RepoReadReportV1::allowed(
        sandbox_root.display().to_string(),
        relative,
        display_path.clone(),
        metadata.len(),
        &content,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-read:1",
        "path": display_path,
        "bytes": metadata.len(),
        "content_digest": read_receipt.content_digest,
        "receipt": read_receipt,
        "content": content,
    }))
}

pub(crate) fn repo_list(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input.get("path").and_then(Value::as_str).unwrap_or(".");
    let max_entries = input
        .get("max_entries")
        .and_then(Value::as_u64)
        .unwrap_or(200)
        .min(1000) as usize;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    if !resolved.is_dir() {
        bail!("repo-list path is not a directory: {relative}")
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&resolved)
        .with_context(|| format!("repo-list cannot read {}", resolved.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)?;
        if path_is_denied_by_prefix(sandbox_root, &path) {
            continue;
        }
        let file_type = metadata.file_type();
        let entry_kind = if file_type.is_symlink() {
            "symlink"
        } else if metadata.is_dir() {
            "dir"
        } else if metadata.is_file() {
            "file"
        } else {
            "other"
        };
        entries.push(RepoListEntryV1 {
            path: display_sandbox_path(sandbox_root, &path),
            entry_kind: entry_kind.into(),
            bytes: metadata.is_file().then_some(metadata.len()),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let total_entries = entries.len();
    let full_listing = serde_json::to_value(&entries).unwrap_or(serde_json::Value::Null);
    let full_listing_digest = DisplayDigestV1::for_json_value(&full_listing);
    entries.truncate(max_entries);
    let receipt = RepoListReportV1::allowed_with_full_listing(
        sandbox_root.display().to_string(),
        relative,
        entries.clone(),
        total_entries,
        full_listing_digest,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-list:1",
        "path": display_sandbox_path(sandbox_root, &resolved),
        "entries": entries,
        "total_entries": total_entries,
        "returned_entries": receipt.returned_entries,
        "truncated": receipt.truncated,
        "full_listing_digest": receipt.full_listing_digest,
        "receipt": receipt,
    }))
}

pub(crate) fn file_stat(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("file-stat input requires string field 'path'"))?;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let metadata = std::fs::metadata(&resolved)
        .with_context(|| format!("file-stat cannot stat {}", resolved.display()))?;
    reject_hardlinked_file(&resolved, &metadata)?;
    let content_digest = if metadata.is_file() && metadata.len() <= 1_048_576 {
        let content = std::fs::read_to_string(&resolved)
            .with_context(|| format!("file-stat cannot read {}", resolved.display()))?;
        Some(DisplayDigestV1::for_text(&content))
    } else {
        None
    };
    Ok(serde_json::json!({
        "tool_id": "aidens:file-stat:1",
        "path": display_sandbox_path(sandbox_root, &resolved),
        "is_file": metadata.is_file(),
        "is_dir": metadata.is_dir(),
        "bytes": metadata.len(),
        "content_digest": content_digest,
    }))
}

pub(crate) fn repo_search(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let query = input
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("repo-search input requires string field 'query'"))?;
    if query.is_empty() {
        bail!("repo-search query must not be empty")
    }
    let relative = input.get("path").and_then(Value::as_str).unwrap_or(".");
    let max_matches = input
        .get("max_matches")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .min(200) as usize;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let mut matches = Vec::new();
    collect_search_matches(sandbox_root, &resolved, query, max_matches, &mut matches)?;
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-search:1",
        "query": query,
        "path": display_sandbox_path(sandbox_root, &resolved),
        "matches": matches,
    }))
}

pub(crate) fn patch_propose(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let summary = input
        .get("summary")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-propose input requires string field 'summary'"))?;
    let diff = input
        .get("diff")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-propose input requires string field 'diff'"))?;
    let touched_paths = touched_paths_from_diff(diff)?;
    for path in &touched_paths {
        let _ = resolve_target_sandboxed_path(sandbox_root, path)?;
    }
    let proposal = PatchProposalV1::new(summary, diff, touched_paths);
    Ok(serde_json::json!({
        "tool_id": "aidens:patch-propose:1",
        "proposal": proposal,
        "mutates_files": false,
    }))
}

pub(crate) fn patch_apply(
    sandbox_root: &Path,
    input: &Value,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
) -> anyhow::Result<Value> {
    let diff = input
        .get("diff")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-apply input requires string field 'diff'"))?;
    let check_only = input
        .get("check_only")
        .or_else(|| input.get("dry_run"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let replacements = parse_simple_unified_diff(diff).map_err(|error| {
        patch_apply_failure(
            sandbox_root,
            input,
            error.to_string(),
            "invalid-patch",
            permit_grant_id.clone(),
            permit_use_receipt_id.clone(),
            PatchFailureState::pre_write(Vec::new()),
        )
    })?;
    let mut before_digests = BTreeMap::new();
    let mut after_digests = BTreeMap::new();
    let mut touched_paths = Vec::new();
    let mut prepared = Vec::new();

    for replacement in replacements {
        let path = resolve_target_sandboxed_path(sandbox_root, &replacement.path)?;
        let before = match std::fs::read_to_string(&path) {
            Ok(content) => OriginalFileState::Existing(content),
            Err(error)
                if error.kind() == io::ErrorKind::NotFound && replacement.removed.is_empty() =>
            {
                OriginalFileState::Absent
            }
            Err(error) => {
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format!(
                        "failed to read patch target {} before applying: {error}",
                        replacement.path
                    ),
                    "read-patch",
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    PatchFailureState::pre_write(vec![replacement.path.clone()]),
                )
                .into());
            }
        };
        let after = apply_single_replacement(before.content(), &replacement).map_err(|error| {
            let failure_kind = if error.to_string().to_ascii_lowercase().contains("ambiguous") {
                "ambiguous-patch"
            } else {
                "invalid-patch"
            };
            patch_apply_failure(
                sandbox_root,
                input,
                error.to_string(),
                failure_kind,
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                PatchFailureState::pre_write(vec![replacement.path.clone()]),
            )
        })?;
        let display_path = display_sandbox_path(sandbox_root, &path);
        before_digests.insert(
            display_path.clone(),
            DisplayDigestV1::for_text(before.content()),
        );
        after_digests.insert(display_path.clone(), DisplayDigestV1::for_text(&after));
        touched_paths.push(display_path.clone());
        prepared.push((path, before, after, display_path));
    }

    if check_only {
        let receipt = PatchApplyReportV1::checked(
            sandbox_root.display().to_string(),
            input,
            touched_paths.clone(),
            before_digests,
            after_digests,
            permit_grant_id,
            permit_use_receipt_id,
        );
        return Ok(serde_json::json!({
            "tool_id": "aidens:patch-apply:1",
            "applied": false,
            "dry_run_checked": true,
            "changed_files": touched_paths,
            "semantic_status": "exact_check",
            "receipt": receipt,
        }));
    }

    let mut attempted_paths = Vec::new();
    let mut written = Vec::new();
    for (path, before, after, display_path) in &prepared {
        attempted_paths.push(display_path.clone());
        if let Err(error) = write_file_atomically(path, after) {
            if error.target_may_have_changed() {
                written.push((path.clone(), before.clone(), display_path.clone()));
            }
            let rollback = rollback_written_files(&written);
            let failure_kind = if rollback.residual_or_unknown_paths.is_empty() {
                "write-patch"
            } else {
                "rollback-failed"
            };
            return Err(patch_apply_failure(
                sandbox_root,
                input,
                format_rollback_failure(
                    format!("failed to write patched file {display_path}: {error}"),
                    rollback.error,
                ),
                failure_kind,
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                PatchFailureState::after_write(
                    touched_paths.clone(),
                    attempted_paths,
                    rollback.restored_existing_paths,
                    rollback.removed_new_paths,
                    rollback.residual_or_unknown_paths,
                ),
            )
            .into());
        }
        written.push((path.clone(), before.clone(), display_path.clone()));
    }

    for (path, _before, after, display_path) in &prepared {
        match std::fs::read_to_string(path) {
            Ok(actual) if actual == *after => {}
            Ok(_) => {
                let rollback = rollback_written_files(&written);
                let failure_kind = if rollback.residual_or_unknown_paths.is_empty() {
                    "rollback-patch"
                } else {
                    "rollback-failed"
                };
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification failed for {display_path}"),
                        rollback.error,
                    ),
                    failure_kind,
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    PatchFailureState::after_write(
                        touched_paths.clone(),
                        attempted_paths,
                        rollback.restored_existing_paths,
                        rollback.removed_new_paths,
                        rollback.residual_or_unknown_paths,
                    ),
                )
                .into());
            }
            Err(error) => {
                let rollback = rollback_written_files(&written);
                let failure_kind = if rollback.residual_or_unknown_paths.is_empty() {
                    "rollback-patch"
                } else {
                    "rollback-failed"
                };
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification could not read {display_path}: {error}"),
                        rollback.error,
                    ),
                    failure_kind,
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    PatchFailureState::after_write(
                        touched_paths.clone(),
                        attempted_paths,
                        rollback.restored_existing_paths,
                        rollback.removed_new_paths,
                        rollback.residual_or_unknown_paths,
                    ),
                )
                .into());
            }
        }
    }

    let receipt = PatchApplyReportV1::new(
        sandbox_root.display().to_string(),
        input,
        touched_paths.clone(),
        before_digests,
        after_digests,
        permit_grant_id,
        permit_use_receipt_id,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:patch-apply:1",
        "applied": true,
        "dry_run_checked": true,
        "changed_files": touched_paths,
        "semantic_status": "exact_check",
        "touched_paths": touched_paths,
        "receipt": receipt,
    }))
}

pub(crate) fn patch_apply_failure(
    sandbox_root: &Path,
    input: &Value,
    message: String,
    failure_kind: &str,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
    file_state: PatchFailureState,
) -> ReceiptBearingToolFailure {
    let touched_paths = file_state.touched_paths.clone();
    let reason_code = match failure_kind {
        "ambiguous-patch" => "patch-ambiguous-failed-closed",
        "read-patch" => "patch-target-read-failed-closed",
        "write-patch" => "patch-write-failed-restored",
        "rollback-failed" => "patch-rollback-failed-quarantined",
        "rollback-patch" => "patch-post-write-verification-failed-restored",
        _ => "patch-invalid-failed-closed",
    };
    let receipt = PatchApplyReportV1::denied_with_details(
        sandbox_root.display().to_string(),
        input,
        reason_code,
        failure_kind,
        touched_paths.clone(),
        permit_grant_id,
        permit_use_receipt_id,
    );
    let restoration_status = file_state.restoration_status();
    let rollback_advice = file_state.rollback_advice();
    ReceiptBearingToolFailure {
        message,
        reason_code: reason_code.into(),
        output: serde_json::json!({
            "tool_id": "aidens:patch-apply:1",
            "applied": false,
            "dry_run_checked": true,
            "changed_files": touched_paths,
            "semantic_status": if restoration_status == "degraded_quarantined" {
                "degraded_quarantined"
            } else {
                "failed_exact_check"
            },
            "failure_kind": failure_kind,
            "attempted_paths": file_state.attempted_paths,
            "restored_paths": file_state.restored_existing_paths,
            "restored_existing_paths": file_state.restored_existing_paths,
            "removed_new_paths": file_state.removed_new_paths,
            "residual_or_unknown_paths": file_state.residual_or_unknown_paths,
            "restoration_status": restoration_status,
            "rollback_advice": rollback_advice,
            "receipt": receipt,
        }),
    }
}

#[derive(Debug, Default)]
pub(crate) struct PatchFailureState {
    touched_paths: Vec<String>,
    attempted_paths: Vec<String>,
    restored_existing_paths: Vec<String>,
    removed_new_paths: Vec<String>,
    residual_or_unknown_paths: Vec<String>,
}

impl PatchFailureState {
    pub(crate) fn pre_write(touched_paths: Vec<String>) -> Self {
        Self {
            touched_paths,
            ..Self::default()
        }
    }

    pub(crate) fn after_write(
        touched_paths: Vec<String>,
        attempted_paths: Vec<String>,
        restored_existing_paths: Vec<String>,
        removed_new_paths: Vec<String>,
        residual_or_unknown_paths: Vec<String>,
    ) -> Self {
        Self {
            touched_paths,
            attempted_paths,
            restored_existing_paths,
            removed_new_paths,
            residual_or_unknown_paths,
        }
    }

    fn restoration_status(&self) -> &'static str {
        if !self.residual_or_unknown_paths.is_empty() {
            "degraded_quarantined"
        } else if !self.restored_existing_paths.is_empty() && !self.removed_new_paths.is_empty() {
            "restored_existing_and_removed_new"
        } else if !self.restored_existing_paths.is_empty() {
            "restored_existing"
        } else if !self.removed_new_paths.is_empty() {
            "removed_new"
        } else {
            "not_required"
        }
    }

    fn rollback_advice(&self) -> Vec<&'static str> {
        if !self.residual_or_unknown_paths.is_empty() {
            vec![
                "One or more attempted paths may remain modified; quarantine and inspect them before retrying.",
                "Do not treat rollback as successful until residual or unknown paths are verified and restored.",
            ]
        } else if !self.restored_existing_paths.is_empty() && !self.removed_new_paths.is_empty() {
            vec![
                "Existing files were restored to captured pre-patch contents and newly created files were removed.",
                "Inspect the primary failure before retrying the patch.",
            ]
        } else if !self.restored_existing_paths.is_empty() {
            vec![
                "Existing files were restored to their captured pre-patch contents.",
                "Inspect the primary failure before retrying the patch.",
            ]
        } else if !self.removed_new_paths.is_empty() {
            vec![
                "Newly created files were removed to restore their captured pre-patch absence.",
                "Inspect the primary failure before retrying the patch.",
            ]
        } else {
            vec![
                "No files were written by this failed-closed patch attempt.",
                "Regenerate a unified diff with unique removal context before retrying.",
            ]
        }
    }
}

#[derive(Debug)]
pub(crate) struct AtomicWriteError {
    source: io::Error,
    target_may_have_changed: bool,
}

impl AtomicWriteError {
    fn before_rename(source: io::Error) -> Self {
        Self {
            source,
            target_may_have_changed: false,
        }
    }

    fn after_rename(source: io::Error) -> Self {
        Self {
            source,
            target_may_have_changed: true,
        }
    }

    pub(crate) fn target_may_have_changed(&self) -> bool {
        self.target_may_have_changed
    }
}

impl fmt::Display for AtomicWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for AtomicWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub(crate) fn write_file_atomically(path: &Path, body: &str) -> Result<(), AtomicWriteError> {
    write_file_atomically_with_parent_sync(path, body, |parent| {
        File::open(parent).and_then(|dir| dir.sync_all())
    })
}

pub(crate) fn write_file_atomically_with_parent_sync<F>(
    path: &Path,
    body: &str,
    sync_parent: F,
) -> Result<(), AtomicWriteError>
where
    F: FnOnce(&Path) -> io::Result<()>,
{
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let tmp_path = parent.join(format!(
        ".{}.patch-tmp-{}-{suffix}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("target"),
        std::process::id()
    ));
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_path)
            .map_err(AtomicWriteError::before_rename)?;
        if let Err(error) = file
            .write_all(body.as_bytes())
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
        {
            return Err(cleanup_temporary_file(&tmp_path, error));
        }
    }
    if let Err(error) = std::fs::rename(&tmp_path, path) {
        return Err(cleanup_temporary_file(&tmp_path, error));
    }
    sync_parent(parent).map_err(AtomicWriteError::after_rename)?;
    Ok(())
}

fn cleanup_temporary_file(tmp_path: &Path, primary: io::Error) -> AtomicWriteError {
    match std::fs::remove_file(tmp_path) {
        Ok(()) => AtomicWriteError::before_rename(primary),
        Err(cleanup) if cleanup.kind() == io::ErrorKind::NotFound => {
            AtomicWriteError::before_rename(primary)
        }
        Err(cleanup) => AtomicWriteError::before_rename(io::Error::new(
            primary.kind(),
            format!(
                "{primary}; failed to remove temporary file {}: {cleanup}",
                tmp_path.display()
            ),
        )),
    }
}

#[derive(Debug, Default)]
pub(crate) struct RollbackOutcome {
    pub(crate) restored_existing_paths: Vec<String>,
    pub(crate) removed_new_paths: Vec<String>,
    pub(crate) residual_or_unknown_paths: Vec<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum OriginalFileState {
    Existing(String),
    Absent,
}

impl OriginalFileState {
    fn content(&self) -> &str {
        match self {
            Self::Existing(content) => content,
            Self::Absent => "",
        }
    }
}

pub(crate) fn rollback_written_files(
    written: &[(PathBuf, OriginalFileState, String)],
) -> RollbackOutcome {
    rollback_written_files_with(written, write_file_atomically, remove_new_file)
}

fn remove_new_file(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn rollback_written_files_with<W, R, WE, RE>(
    written: &[(PathBuf, OriginalFileState, String)],
    mut restore_existing: W,
    mut remove_new: R,
) -> RollbackOutcome
where
    W: FnMut(&Path, &str) -> Result<(), WE>,
    R: FnMut(&Path) -> Result<(), RE>,
    WE: fmt::Display,
    RE: fmt::Display,
{
    let mut restored_existing_paths = Vec::new();
    let mut removed_new_paths = Vec::new();
    let mut residual_or_unknown_paths = Vec::new();
    let mut failures = Vec::new();
    for (path, before, display_path) in written.iter().rev() {
        let rollback_result = match before {
            OriginalFileState::Existing(content) => restore_existing(path, content)
                .map(|()| &mut restored_existing_paths)
                .map_err(|error| error.to_string()),
            OriginalFileState::Absent => remove_new(path)
                .map(|()| &mut removed_new_paths)
                .map_err(|error| error.to_string()),
        };
        match rollback_result {
            Ok(restored_paths) => restored_paths.push(display_path.clone()),
            Err(error) => {
                residual_or_unknown_paths.push(display_path.clone());
                failures.push(format!("{}: {error}", path.display()));
            }
        }
    }
    restored_existing_paths.sort();
    removed_new_paths.sort();
    residual_or_unknown_paths.sort();
    RollbackOutcome {
        restored_existing_paths,
        removed_new_paths,
        residual_or_unknown_paths,
        error: (!failures.is_empty()).then(|| failures.join("; ")),
    }
}

pub(crate) fn format_rollback_failure(primary: String, rollback_error: Option<String>) -> String {
    match rollback_error {
        Some(rollback_error) => format!("{primary}; rollback failed: {rollback_error}"),
        None => primary,
    }
}

pub(crate) fn run_checks(
    sandbox_root: &Path,
    input: &Value,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
) -> anyhow::Result<Value> {
    let command = command_args_from_input(input)?;
    if !command_is_allowed_check(&command) {
        let receipt = CommandRunReportV1::blocked(
            sandbox_root.display().to_string(),
            command,
            "command-not-allowed-by-policy",
        );
        return Ok(serde_json::json!({
            "tool_id": "aidens:run-checks:1",
            "succeeded": false,
            "receipt": receipt,
        }));
    }
    let timed_output =
        run_command_with_timeout(sandbox_root, &command, Duration::from_secs(120))
            .with_context(|| format!("failed to run check command: {}", command.join(" ")))?;
    let stdout_truncated = timed_output.output.stdout.len() > MAX_COMMAND_OUTPUT_BYTES;
    let stderr_truncated = timed_output.output.stderr.len() > MAX_COMMAND_OUTPUT_BYTES;
    let stdout = capped_utf8_lossy(&timed_output.output.stdout, MAX_COMMAND_OUTPUT_BYTES);
    let stderr = capped_utf8_lossy(&timed_output.output.stderr, MAX_COMMAND_OUTPUT_BYTES);
    let mut receipt = CommandRunReportV1::completed(
        sandbox_root.display().to_string(),
        command.clone(),
        permit_grant_id,
        permit_use_receipt_id,
        timed_output.output.status.code(),
        &stdout,
        &stderr,
    );
    if timed_output.timed_out {
        receipt.timed_out = true;
        receipt.succeeded = false;
        receipt.reason_codes = vec![
            "check-command-timeout".into(),
            "command-output-partial-after-timeout".into(),
        ];
    }
    if timed_output.kill_failed {
        receipt.reason_codes.push("kill-failure".into());
    }
    if stdout_truncated {
        receipt.reason_codes.push("stdout-truncated".into());
    }
    if stderr_truncated {
        receipt.reason_codes.push("stderr-truncated".into());
    }
    receipt.reason_codes.sort();
    receipt.reason_codes.dedup();
    let semantic_status = if timed_output.timed_out {
        "partial_timeout"
    } else if stdout_truncated || stderr_truncated {
        "partial_output_capped"
    } else {
        "exact_check"
    };
    Ok(serde_json::json!({
        "tool_id": "aidens:run-checks:1",
        "command": command,
        "succeeded": receipt.succeeded,
        "exit_code": receipt.exit_code,
        "stdout": stdout,
        "stderr": stderr,
        "semantic_status": semantic_status,
        "receipt": receipt,
    }))
}

pub(crate) const MAX_COMMAND_OUTPUT_BYTES: usize = 65_536;

pub(crate) struct TimedCommandOutput {
    output: std::process::Output,
    pub timed_out: bool,
    kill_failed: bool,
}

pub(crate) fn capped_utf8_lossy(bytes: &[u8], cap: usize) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(cap)]).to_string()
}

pub(crate) fn run_command_with_timeout(
    sandbox_root: &Path,
    command: &[String],
    timeout: Duration,
) -> anyhow::Result<TimedCommandOutput> {
    let executable = resolve_allowed_command_executable(&command[0])?;
    let mut command_proc = Command::new(executable);
    command_proc
        .args(&command[1..])
        .current_dir(sandbox_root)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        command_proc.process_group(0);
    }
    let mut child = command_proc.spawn()?;
    let started = Instant::now();
    let mut wait_interval = Duration::from_millis(5);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(TimedCommandOutput {
                output: child.wait_with_output()?,
                timed_out: false,
                kill_failed: false,
            });
        }
        if started.elapsed() >= timeout {
            let kill_failed = terminate_timed_out_command(&mut child, &command[0]);
            return Ok(TimedCommandOutput {
                output: child.wait_with_output()?,
                timed_out: true,
                kill_failed,
            });
        }
        let elapsed = started.elapsed();
        let remaining = timeout.saturating_sub(elapsed);
        std::thread::sleep(wait_interval.min(remaining).min(Duration::from_millis(250)));
        wait_interval = (wait_interval * 2).min(Duration::from_millis(250));
    }
}

pub(crate) fn terminate_timed_out_command(child: &mut Child, command_label: &str) -> bool {
    // If the child already exited before we could terminate it, report no kill failure.
    if child.try_wait().is_ok_and(|s| s.is_some()) {
        return false;
    }
    #[cfg(unix)]
    {
        if terminate_unix_process_group(child.id()).is_ok() {
            return false;
        }
    }
    // Process-group termination is unavailable or failed.
    // Emit kill-failure degradation and return true so the receipt records it.
    eprintln!("WARNING: kill-failure for timed-out command {command_label}: process-group termination unavailable or failed");
    true
}

#[cfg(unix)]
pub(crate) fn terminate_unix_process_group(child_pid: u32) -> anyhow::Result<()> {
    let process_group = format!("-{child_pid}");
    for kill_path in ["/bin/kill", "/usr/bin/kill"] {
        if !Path::new(kill_path).exists() {
            continue;
        }
        let status = Command::new(kill_path)
            .args(["-KILL", &process_group])
            .env_clear()
            .status()
            .with_context(|| format!("failed to invoke fixed kill executable {kill_path}"))?;
        if status.success() {
            return Ok(());
        }
    }
    bail!("no fixed kill executable could terminate process group {process_group}")
}

pub(crate) fn resolve_allowed_command_executable(command: &str) -> anyhow::Result<PathBuf> {
    let candidates: &[&str] = match command {
        "cargo" => &[
            "/usr/bin/cargo",
            "/usr/local/bin/cargo",
            "/root/.cargo/bin/cargo",
        ],
        "bash" => &["/usr/bin/bash", "/bin/bash"],
        other => bail!("command executable is not in the fixed allowlist: {other}"),
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .ok_or_else(|| anyhow!("allowed command executable not found in fixed paths: {command}"))
}

pub(crate) fn command_args_from_input(input: &Value) -> anyhow::Result<Vec<String>> {
    if let Some(command) = input.get("command").and_then(Value::as_array) {
        let args = command
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| anyhow!("run-checks command entries must be strings"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        if args.is_empty() {
            bail!("run-checks command must not be empty")
        }
        return Ok(args);
    }
    if input.get("command").and_then(Value::as_str).is_some() {
        bail!("run-checks command must be structured argv array; shell/string command parsing is unsupported")
    }
    bail!("run-checks input requires field 'command'")
}

pub(crate) fn command_is_allowed_check(command: &[String]) -> bool {
    const ALLOWED: &[&[&str]] = &[
        &["cargo", "fmt", "--all", "--check"],
        &["cargo", "check", "--workspace"],
        &["cargo", "test", "--workspace"],
        &[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        &["bash", "scripts/verify.sh"],
    ];
    ALLOWED.iter().any(|allowed| {
        command
            .iter()
            .map(String::as_str)
            .eq(allowed.iter().copied())
    })
}
