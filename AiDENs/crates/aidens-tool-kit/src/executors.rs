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
use std::fs::{File, OpenOptions};
use std::io::Write;
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
            Vec::new(),
        )
    })?;
    let mut before_digests = BTreeMap::new();
    let mut after_digests = BTreeMap::new();
    let mut touched_paths = Vec::new();
    let mut prepared = Vec::new();

    for replacement in replacements {
        let path = resolve_target_sandboxed_path(sandbox_root, &replacement.path)?;
        // For new-file patches (no removed lines), the file may not exist yet.
        // Read the before-content as empty string if the file doesn't exist.
        let before = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) if replacement.removed.is_empty() => String::new(),
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
                    vec![replacement.path.clone()],
                )
                .into());
            }
        };
        let after = apply_single_replacement(&before, &replacement).map_err(|error| {
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
                vec![replacement.path.clone()],
            )
        })?;
        let display_path = display_sandbox_path(sandbox_root, &path);
        before_digests.insert(display_path.clone(), DisplayDigestV1::for_text(&before));
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

    let mut written = Vec::new();
    for (path, before, after, display_path) in &prepared {
        if let Err(error) = write_file_atomically(path, after) {
            let rollback_error = rollback_written_files(&written).err();
            return Err(patch_apply_failure(
                sandbox_root,
                input,
                format_rollback_failure(
                    format!("failed to write patched file {display_path}: {error}"),
                    rollback_error,
                ),
                "rollback-failed",
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                touched_paths.clone(),
            )
            .into());
        }
        written.push((path.clone(), before.clone()));
    }

    for (path, _before, after, display_path) in &prepared {
        match std::fs::read_to_string(path) {
            Ok(actual) if actual == *after => {}
            Ok(_) => {
                let rollback_error = rollback_written_files(&written).err();
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification failed for {display_path}"),
                        rollback_error,
                    ),
                    "rollback-failed",
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    touched_paths.clone(),
                )
                .into());
            }
            Err(error) => {
                let rollback_error = rollback_written_files(&written).err();
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification could not read {display_path}: {error}"),
                        rollback_error,
                    ),
                    "rollback-failed",
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    touched_paths.clone(),
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
    touched_paths: Vec<String>,
) -> ReceiptBearingToolFailure {
    let reason_code = match failure_kind {
        "ambiguous-patch" => "patch-ambiguous-failed-closed",
        "read-patch" => "patch-target-read-failed-closed",
        "rollback-failed" => "patch-rollback-failed-quarantined",
        "rollback-patch" => "patch-rollback-quarantined",
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
    ReceiptBearingToolFailure {
        message,
        reason_code: reason_code.into(),
        output: serde_json::json!({
            "tool_id": "aidens:patch-apply:1",
            "applied": false,
            "dry_run_checked": true,
            "changed_files": touched_paths,
            "semantic_status": "failed_exact_check",
            "failure_kind": failure_kind,
            "rollback_advice": [
                "No files were written by this failed-closed patch attempt.",
                "Regenerate a single-file unified diff with unique removal context before retrying."
            ],
            "receipt": receipt,
        }),
    }
}

pub(crate) fn write_file_atomically(path: &Path, body: &str) -> std::io::Result<()> {
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
            .open(&tmp_path)?;
        file.write_all(body.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp_path, path)?;
    let _ = File::open(parent).and_then(|dir| dir.sync_all());
    Ok(())
}

pub(crate) fn rollback_written_files(written: &[(PathBuf, String)]) -> Result<(), String> {
    let mut failures = Vec::new();
    for (path, before) in written.iter().rev() {
        if let Err(error) = write_file_atomically(path, before) {
            failures.push(format!("{}: {error}", path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
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
