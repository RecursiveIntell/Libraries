use crate::{
    error::{Error, Result},
    model::RepositorySnapshot,
    v2::SourceSnapshotV2,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        return Err(Error::Command(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn snapshot_repo(cwd: &Path, baseline: &str) -> Result<RepositorySnapshot> {
    let final_sha = git(cwd, &["rev-parse", "HEAD"])?.trim().to_string();
    let status = git(cwd, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    let diff = git(cwd, &["diff", "HEAD", "--binary"])?;
    let diff_stat = git(cwd, &["diff", "HEAD", "--stat"])?;
    let digest = hex::encode(Sha256::digest(diff.as_bytes()));
    Ok(RepositorySnapshot {
        path: cwd.display().to_string(),
        baseline_sha: baseline.to_string(),
        final_sha,
        is_clean: status.trim().is_empty(),
        diff_stat,
        diff,
        status,
        diff_digest: digest,
    })
}

/// Captures one source identity snapshot. Callers must capture both pre- and
/// post-command snapshots; this function never infers a baseline from empty data.
pub fn source_snapshot_v2(cwd: &Path) -> Result<SourceSnapshotV2> {
    let head = git(cwd, &["rev-parse", "HEAD"])?.trim().to_string();
    let tree = git(cwd, &["rev-parse", "HEAD^{tree}"])?.trim().to_string();
    let raw_status = git(cwd, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    // AEW's own untracked event store is a projection, not repository source.
    let status = raw_status
        .lines()
        .filter(|line| !line.get(3..).is_some_and(|path| path.starts_with(".aew/")))
        .collect::<Vec<_>>()
        .join("\n");
    let status = if status.is_empty() {
        status
    } else {
        format!("{status}\n")
    };
    let diff = git(cwd, &["diff", "HEAD", "--binary"])?;
    // Include ignored files too: Git ignore rules are not a source-authority boundary.
    // Only named AEW projections, build output, and Git metadata are excluded.
    let untracked = git(
        cwd,
        &["ls-files", "--others", "--ignored", "--exclude-standard"],
    )?;
    let mut content = diff.as_bytes().to_vec();
    for name in untracked.lines().filter(|name| {
        !name.starts_with(".aew/") && !name.starts_with(".git/") && !name.starts_with("target/")
    }) {
        let path = cwd.join(name);
        if path.is_file() {
            content.extend_from_slice(name.as_bytes());
            content.extend_from_slice(&std::fs::read(path)?);
        }
    }
    Ok(SourceSnapshotV2 {
        repository_path: cwd.display().to_string(),
        head,
        tree,
        is_clean: status.trim().is_empty(),
        status,
        diff_digest: hex::encode(Sha256::digest(diff.as_bytes())),
        workspace_content_digest: hex::encode(Sha256::digest(content)),
        observed_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn non_repository_fails_closed() {
        let directory = tempdir().expect("tempdir");
        assert!(source_snapshot_v2(directory.path()).is_err());
    }
}
