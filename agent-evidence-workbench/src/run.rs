use crate::{
    model::{CheckResult, CommandOutcome, ExecutionPolicy, DEFAULT_EXECUTION_POLICY},
    v2::redact_text,
};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Stdio, time::Instant};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    task::JoinHandle,
    time::{sleep, timeout, Duration},
};

const STREAM_CLOSE_GRACE: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct CapturedStream {
    bytes: Vec<u8>,
    truncated: bool,
    error: Option<String>,
}

async fn read_capped<R: AsyncRead + Unpin>(mut reader: R, cap: usize) -> CapturedStream {
    let mut bytes = Vec::with_capacity(cap);
    let mut chunk = [0_u8; 4_096];
    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => {
                return CapturedStream {
                    bytes,
                    truncated: false,
                    error: None,
                }
            }
            Ok(count) => {
                let remaining = cap.saturating_sub(bytes.len());
                bytes.extend_from_slice(&chunk[..count.min(remaining)]);
                if count > remaining {
                    return CapturedStream {
                        bytes,
                        truncated: true,
                        error: None,
                    };
                }
            }
            Err(error) => {
                return CapturedStream {
                    bytes,
                    truncated: false,
                    error: Some(error.to_string()),
                }
            }
        }
    }
}

async fn finish_reader(handle: &mut Option<JoinHandle<CapturedStream>>) -> CapturedStream {
    let Some(mut handle) = handle.take() else {
        return CapturedStream {
            bytes: Vec::new(),
            truncated: false,
            error: None,
        };
    };
    match timeout(STREAM_CLOSE_GRACE, &mut handle).await {
        Ok(Ok(captured)) => captured,
        Ok(Err(error)) => CapturedStream {
            bytes: Vec::new(),
            truncated: false,
            error: Some(format!("output task failed: {error}")),
        },
        Err(_) => {
            handle.abort();
            CapturedStream {
                bytes: Vec::new(),
                truncated: false,
                error: Some("output stream did not close after direct-child termination".into()),
            }
        }
    }
}

async fn wait_reader(
    handle: &mut Option<JoinHandle<CapturedStream>>,
) -> Result<CapturedStream, tokio::task::JoinError> {
    match handle.as_mut() {
        Some(handle) => handle.await,
        None => std::future::pending().await,
    }
}

fn rendered_stream(stream: &str, captured: CapturedStream, cap: usize) -> String {
    let mut rendered = String::from_utf8_lossy(&captured.bytes).into_owned();
    if captured.truncated {
        rendered.push_str(&format!(
            "\n[AEW {stream} truncated: retained_prefix_bytes={} cap_bytes={cap}]\n",
            captured.bytes.len()
        ));
    }
    if let Some(error) = captured.error {
        rendered.push_str(&format!("\n[AEW {stream} capture incomplete: {error}]\n"));
    }
    rendered
}

struct ReceiptInput<'a> {
    cmd: &'a str,
    args: &'a [String],
    policy: ExecutionPolicy,
    outcome: CommandOutcome,
    exit_code: Option<i32>,
    raw_stdout: String,
    raw_stderr: String,
    duration_ms: u128,
}

fn receipt(input: ReceiptInput<'_>) -> CheckResult {
    let command = std::iter::once(input.cmd)
        .chain(input.args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    let command = redact_text(&command);
    let stdout = redact_text(&input.raw_stdout);
    let stderr = redact_text(&input.raw_stderr);
    let digest = |text: &str| hex::encode(Sha256::digest(text.as_bytes()));
    CheckResult {
        command: command.text,
        outcome: input.outcome,
        policy: input.policy,
        exit_code: input.exit_code,
        stdout_digest: digest(&stdout.text),
        stderr_digest: digest(&stderr.text),
        duration_ms: input.duration_ms,
        passed: matches!(input.outcome, CommandOutcome::Completed),
        stdout: stdout.text,
        stderr: stderr.text,
        redaction_count: command.redaction_count + stdout.redaction_count + stderr.redaction_count,
    }
}

pub async fn run_command_with_policy(
    cmd: &str,
    args: &[String],
    cwd: &Path,
    policy: ExecutionPolicy,
) -> CheckResult {
    let start = Instant::now();
    let mut child = match Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return receipt(ReceiptInput {
                cmd,
                args,
                policy,
                outcome: CommandOutcome::LaunchFailed,
                exit_code: None,
                raw_stdout: String::new(),
                raw_stderr: format!("[AEW launch failed: {error}]"),
                duration_ms: start.elapsed().as_millis(),
            })
        }
    };
    let mut stdout = Some(tokio::spawn(read_capped(
        child.stdout.take().expect("stdout is piped"),
        policy.stdout_cap_bytes,
    )));
    let mut stderr = Some(tokio::spawn(read_capped(
        child.stderr.take().expect("stderr is piped"),
        policy.stderr_cap_bytes,
    )));
    let mut captured_stdout = None;
    let mut captured_stderr = None;

    enum Stop {
        Completed(std::io::Result<std::process::ExitStatus>),
        TimedOut,
        CaptureFailed,
    }

    let stop = {
        let wait = child.wait();
        tokio::pin!(wait);
        let deadline = sleep(Duration::from_millis(policy.deadline_ms));
        tokio::pin!(deadline);
        loop {
            tokio::select! {
                status = &mut wait => break Stop::Completed(status),
                _ = &mut deadline => break Stop::TimedOut,
                result = wait_reader(&mut stdout) => {
                    let captured = result.unwrap_or_else(|error| CapturedStream {
                        bytes: Vec::new(),
                        truncated: false,
                        error: Some(format!("stdout task failed: {error}")),
                    });
                    let truncated = captured.truncated;
                    captured_stdout = Some(captured);
                    stdout = None;
                    if truncated { break Stop::CaptureFailed; }
                }
                result = wait_reader(&mut stderr) => {
                    let captured = result.unwrap_or_else(|error| CapturedStream {
                        bytes: Vec::new(),
                        truncated: false,
                        error: Some(format!("stderr task failed: {error}")),
                    });
                    let truncated = captured.truncated;
                    captured_stderr = Some(captured);
                    stderr = None;
                    if truncated { break Stop::CaptureFailed; }
                }
            }
        }
    };

    if matches!(&stop, Stop::TimedOut | Stop::CaptureFailed) {
        let _ = child.kill().await;
    }
    let exit_code = match &stop {
        Stop::Completed(Ok(status)) => status.code(),
        Stop::TimedOut | Stop::CaptureFailed => {
            child.wait().await.ok().and_then(|status| status.code())
        }
        Stop::Completed(Err(_)) => None,
    };
    let stdout = match captured_stdout {
        Some(captured) => captured,
        None => finish_reader(&mut stdout).await,
    };
    let stderr = match captured_stderr {
        Some(captured) => captured,
        None => finish_reader(&mut stderr).await,
    };
    let capture_failed =
        stdout.truncated || stderr.truncated || stdout.error.is_some() || stderr.error.is_some();
    let raw_stdout = rendered_stream("stdout", stdout, policy.stdout_cap_bytes);
    let raw_stderr = rendered_stream("stderr", stderr, policy.stderr_cap_bytes);
    let outcome = match stop {
        Stop::TimedOut => CommandOutcome::TimedOut,
        Stop::CaptureFailed => CommandOutcome::CaptureFailed,
        Stop::Completed(Ok(_)) if capture_failed => CommandOutcome::CaptureFailed,
        Stop::Completed(Ok(status)) if status.success() => CommandOutcome::Completed,
        Stop::Completed(Ok(_)) => CommandOutcome::Failed,
        Stop::Completed(Err(error)) => {
            return receipt(ReceiptInput {
                cmd,
                args,
                policy,
                outcome: CommandOutcome::CaptureFailed,
                exit_code: None,
                raw_stdout,
                raw_stderr: format!("{raw_stderr}\n[AEW wait failed: {error}]"),
                duration_ms: start.elapsed().as_millis(),
            })
        }
    };
    receipt(ReceiptInput {
        cmd,
        args,
        policy,
        outcome,
        exit_code,
        raw_stdout,
        raw_stderr,
        duration_ms: start.elapsed().as_millis(),
    })
}

pub async fn run_command(cmd: &str, args: &[String], cwd: &Path) -> CheckResult {
    run_command_with_policy(cmd, args, cwd, DEFAULT_EXECUTION_POLICY).await
}

pub async fn run_agent(cmd: &str, args: &[String], cwd: &Path) -> CheckResult {
    run_command(cmd, args, cwd).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn timeout_returns_a_typed_timed_out_outcome() {
        let directory = tempdir().expect("tempdir");
        let policy = ExecutionPolicy {
            deadline_ms: 50,
            stdout_cap_bytes: 1_024,
            stderr_cap_bytes: 1_024,
        };
        let result =
            run_command_with_policy("sleep", &["1".into()], directory.path(), policy).await;

        assert_eq!(result.outcome, CommandOutcome::TimedOut);
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn stdout_and_stderr_caps_return_a_typed_capture_failure() {
        let directory = tempdir().expect("tempdir");
        let policy = ExecutionPolicy {
            deadline_ms: 1_000,
            stdout_cap_bytes: 8,
            stderr_cap_bytes: 8,
        };
        let stdout = run_command_with_policy(
            "sh",
            &["-c".into(), "printf 123456789".into()],
            directory.path(),
            policy,
        )
        .await;
        let stderr = run_command_with_policy(
            "sh",
            &["-c".into(), "printf abcdefghi >&2".into()],
            directory.path(),
            policy,
        )
        .await;

        assert_eq!(stdout.outcome, CommandOutcome::CaptureFailed);
        assert_eq!(stderr.outcome, CommandOutcome::CaptureFailed);
        assert!(!stdout.passed && !stderr.passed);
        assert!(stdout.stdout.contains("truncated"));
        assert!(stderr.stderr.contains("truncated"));
        assert!(!stdout.stdout.contains("123456789"));
        assert!(!stderr.stderr.contains("abcdefghi"));
    }

    #[tokio::test]
    async fn unavailable_executable_returns_a_typed_launch_failure() {
        let directory = tempdir().expect("tempdir");
        let result = run_command_with_policy(
            "aew-test-executable-that-does-not-exist",
            &[],
            directory.path(),
            DEFAULT_EXECUTION_POLICY,
        )
        .await;

        assert_eq!(result.outcome, CommandOutcome::LaunchFailed);
        assert!(!result.passed);
    }

    #[test]
    fn v2_mapping_preserves_timeout_and_maps_boundary_errors_to_error() {
        assert_eq!(
            CommandOutcome::TimedOut.to_v2_outcome(),
            crate::v2::CommandOutcomeV2::TimedOut
        );
        assert_eq!(
            CommandOutcome::LaunchFailed.to_v2_outcome(),
            crate::v2::CommandOutcomeV2::Error
        );
        assert_eq!(
            CommandOutcome::CaptureFailed.to_v2_outcome(),
            crate::v2::CommandOutcomeV2::Error
        );
    }

    #[tokio::test]
    async fn captures_output() {
        let directory = tempdir().expect("tempdir");
        let result = run_command("echo", &["hello".into()], directory.path()).await;
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
        .await;
        assert_eq!(result.redaction_count, 3);
        assert!(!result.stdout.contains("secret-token-123"));
        assert!(!result.command.contains("argv-secret-456"));
    }
}
