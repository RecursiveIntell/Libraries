use crate::{error::Result, model::CheckResult, v2::redact_text};
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
    let command = std::iter::once(cmd)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    let command = redact_text(&command);
    let raw_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let raw_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = redact_text(&raw_stdout);
    let stderr = redact_text(&raw_stderr);
    let digest = |text: &str| hex::encode(Sha256::digest(text.as_bytes()));
    Ok(CheckResult {
        command: command.text,
        exit_code: output.status.code(),
        stdout_digest: digest(&stdout.text),
        stderr_digest: digest(&stderr.text),
        duration_ms: start.elapsed().as_millis(),
        passed: output.status.success(),
        stdout: stdout.text,
        stderr: stderr.text,
        redaction_count: command.redaction_count + stdout.redaction_count + stderr.redaction_count,
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
        let directory = tempdir().expect("tempdir");
        let result = run_command("echo", &["hello".into()], directory.path())
            .await
            .expect("run");
        assert!(result.passed);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn redacts_output_before_returning_a_receipt() {
        let directory = tempdir().expect("tempdir");
        let result = run_command(
            "sh",
            &[
                "-c".into(),
                "printf 'Bearer secret-token-123\\n'".into(),
                "--api_key=argv-secret-456".into(),
            ],
            directory.path(),
        )
        .await
        .expect("run");
        assert_eq!(result.redaction_count, 3);
        assert!(!result.stdout.contains("secret-token-123"));
        assert!(!result.command.contains("argv-secret-456"));
    }
}
