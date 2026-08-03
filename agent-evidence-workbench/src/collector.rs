use crate::{error::Result, model::RepositorySnapshot};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;
fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8_lossy(
        &Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()?
            .stdout,
    )
    .into_owned())
}
pub fn snapshot_repo(cwd: &Path, baseline: &str) -> Result<RepositorySnapshot> {
    let final_sha = git(cwd, &["rev-parse", "HEAD"])?.trim().to_string();
    let status = git(cwd, &["status", "--porcelain"])?;
    let diff = git(cwd, &["diff"])?;
    let diff_stat = git(cwd, &["diff", "--stat"])?;
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
