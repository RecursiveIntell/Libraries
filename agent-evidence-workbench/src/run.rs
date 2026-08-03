use crate::error::Result;
use crate::model::CheckResult;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use tokio::process::Command;

pub async fn run_command(cmd: &str, args: &[String], cwd: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let output = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let digest = |s: &str| hex::encode(Sha256::digest(s.as_bytes()));
    Ok(CheckResult {
        command: std::iter::once(cmd)
            .chain(args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" "),
        exit_code: output.status.code(),
        stdout_digest: digest(&stdout),
        stderr_digest: digest(&stderr),
        duration_ms: start.elapsed().as_millis(),
        passed: output.status.success(),
        stdout,
        stderr,
    })
}

pub async fn run_agent(cmd: &str, args: &[String], cwd: &Path) -> Result<CheckResult> {
    run_command(cmd, args, cwd).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[tokio::test]
    async fn captures_output() {
        let d = tempdir().expect("tempdir");
        let r = run_command("echo", &["hello".into()], d.path())
            .await
            .expect("run");
        assert!(r.passed);
        assert!(r.stdout.contains("hello"));
    }
}
