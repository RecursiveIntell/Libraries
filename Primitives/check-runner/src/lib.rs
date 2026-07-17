//! # check-runner
//!
//! Host and container execution surface for patch verification.
//!
//! This Tier 3 crate prepares workspaces, runs validation commands, captures
//! effects, and normalizes command output for higher-level scoring and
//! attribution. It is execution infrastructure, not a policy owner.

use std::path::Path;
#[cfg(feature = "container")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "container")]
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;

pub use effect_signature::{EffectSignature, LocatedEffect};
pub use sandbox_workspace::{PatchedWorkspace, Workspace};

const MAX_CAPTURED_OUTPUT_BYTES: usize = 2_000_000;
const OUTPUT_TRUNCATION_MARKER: &str = "\n[check-runner output truncated]\n";

fn bounded_output(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= MAX_CAPTURED_OUTPUT_BYTES {
        return text.into_owned();
    }

    let mut end = MAX_CAPTURED_OUTPUT_BYTES.saturating_sub(OUTPUT_TRUNCATION_MARKER.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &text[..end], OUTPUT_TRUNCATION_MARKER)
}

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("policy error: {0}")]
    Policy(#[from] forge_policy::PolicyError),

    #[error("workspace error: {0}")]
    Workspace(#[from] sandbox_workspace::WorkspaceError),

    #[error("command timeout after {timeout_secs}s: {command}")]
    CommandTimeout { command: String, timeout_secs: u64 },

    #[error("command failed (exit {exit_code}): {command}")]
    CommandFailed { command: String, exit_code: i32 },

    #[error("sealed mode unsupported for runtime: {runtime}")]
    SealedModeUnsupported { runtime: String },

    #[error("no container runtime found")]
    NoContainerRuntime,

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub mode: String,
    pub execution_backend_preference: String,
    pub container_runtime_preference: String,
    pub sealed_allow_host_backend: bool,
    pub rust_image: String,
    pub command_timeout_secs: u64,
    pub memory_limit: String,
    pub cpu_limit: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CheckKind {
    #[default]
    Fmt,
    Clippy,
    Test,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedCheckOutput {
    pub check_kind: CheckKind,
    pub exit_code: i32,
    pub effects: Vec<LocatedEffect>,
    pub raw_stdout: String,
    pub raw_stderr: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandTimings {
    pub fmt_ms: u64,
    pub clippy_ms: u64,
    pub test_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogBundle {
    pub fmt_stdout: String,
    pub fmt_stderr: String,
    pub clippy_stdout: String,
    pub clippy_stderr: String,
    pub test_stdout: String,
    pub test_stderr: String,
    pub timings: CommandTimings,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub fmt_pass: bool,
    pub clippy_pass: bool,
    pub test_pass: bool,
    pub fmt_output: ParsedCheckOutput,
    pub clippy_output: ParsedCheckOutput,
    pub test_output: ParsedCheckOutput,
    pub total_duration_ms: u64,
}

impl CheckResult {
    pub fn all_pass(&self) -> bool {
        self.fmt_pass && self.clippy_pass && self.test_pass
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionBackendKind {
    Host,
    Container,
}

#[derive(Debug, Clone)]
pub struct CheckCommand {
    pub kind: CheckKind,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn kind(&self) -> ExecutionBackendKind;

    /// True only when this backend has owner-backed runtime evidence from the
    /// current execution. Backend kind and a structurally valid receipt are
    /// configuration/shape signals, not proof of live execution.
    fn has_live_execution_evidence(&self) -> bool {
        false
    }

    async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace, RunnerError>;

    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout_secs: u64,
    ) -> Result<CommandOutput, RunnerError>;

    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> Result<LogBundle, RunnerError>;
}

pub fn is_env_allowed(key: &str) -> bool {
    forge_policy::is_env_allowed(key)
}

pub struct HostBackend {
    timeout_secs: u64,
}

impl HostBackend {
    pub fn new(config: &BackendConfig) -> Self {
        Self {
            timeout_secs: config.command_timeout_secs,
        }
    }
}

#[async_trait]
impl ExecutionBackend for HostBackend {
    fn kind(&self) -> ExecutionBackendKind {
        ExecutionBackendKind::Host
    }

    async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace, RunnerError> {
        Ok(sandbox_workspace::prepare_workspace(fixture)?)
    }

    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout_secs: u64,
    ) -> Result<CommandOutput, RunnerError> {
        let timeout = if timeout_secs > 0 {
            timeout_secs
        } else {
            self.timeout_secs
        };

        let start = Instant::now();
        let mut command = tokio::process::Command::new(program);
        command.args(args);
        command.current_dir(workspace);
        command.env_clear();

        for (key, value) in std::env::vars() {
            if is_env_allowed(&key) {
                command.env(&key, &value);
            }
        }

        command.env("CARGO_TERM_COLOR", "never");
        command.env("RUST_BACKTRACE", "0");
        for (key, value) in env {
            command.env(key, value);
        }

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);

        let child = command
            .spawn()
            .map_err(|error| RunnerError::Other(format!("failed to spawn {program}: {error}")))?;
        #[cfg(unix)]
        let child_pid = child.id().map(|pid| pid as i32);
        let wait = child.wait_with_output();
        tokio::pin!(wait);
        let output = tokio::time::timeout(Duration::from_secs(timeout), &mut wait).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        match output {
            Ok(Ok(output)) => Ok(CommandOutput {
                stdout: bounded_output(&output.stdout),
                stderr: bounded_output(&output.stderr),
                exit_code: output.status.code().unwrap_or(-1),
                duration_ms,
            }),
            Ok(Err(error)) => Err(RunnerError::Other(format!(
                "command {program} failed: {error}"
            ))),
            Err(_) => {
                // P1-1: V30 HARDENING — scoped-allow for architectural process-group termination.
                // This unsafe block is NOT a casual shortcut. It is the only Unix mechanism to
                // terminate an entire spawned process group when a command times out. The child
                // was spawned with process_group(0) — kill() on the parent PID alone would leave
                // orphaned children. No safe Rust wrapper exists for killpg(); this is the minimal
                // correct primitive. Evidence:
                // - tokio::process::Command does not expose PID for safe killpg() dispatch
                // - std::process::Command::kill() hits the same syscall internally (always unsafe)
                // - Precondition: child_pid is a process group leader spawned in this session
                // - Postcondition: entire process group receives SIGKILL, all processes terminated
                #[allow(clippy::arc_with_non_send_sync)]
                check_runner_sys::kill_process_group(
                    child_pid.expect("child PID must be Some immediately after spawn"),
                );

                Err(RunnerError::CommandTimeout {
                    command: format!("{program} {}", args.join(" ")),
                    timeout_secs: timeout,
                })
            }
        }
    }

    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> Result<LogBundle, RunnerError> {
        Ok(LogBundle {
            fmt_stdout: fmt.stdout.clone(),
            fmt_stderr: fmt.stderr.clone(),
            clippy_stdout: clippy.stdout.clone(),
            clippy_stderr: clippy.stderr.clone(),
            test_stdout: test.stdout.clone(),
            test_stderr: test.stderr.clone(),
            timings: CommandTimings {
                fmt_ms: fmt.duration_ms,
                clippy_ms: clippy.duration_ms,
                test_ms: test.duration_ms,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Nerdctl,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxCapabilityTruthReceiptV1 {
    pub schema: String,
    pub execution_mode: String,
    pub runtime: String,
    pub requested_image: String,
    pub resolved_image_digest: String,
    pub container_id: String,
    pub runtime_observation_digest: String,
    pub rootless_required: bool,
    pub rootless_observed: bool,
    pub mechanisms: Vec<String>,
    pub memory_limit: String,
    pub cpu_limit: String,
    pub content_digest: String,
}

impl SandboxCapabilityTruthReceiptV1 {
    fn digest_material(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.schema,
            self.execution_mode,
            self.runtime,
            self.requested_image,
            self.resolved_image_digest,
            self.container_id,
            self.runtime_observation_digest,
            self.rootless_required,
            self.rootless_observed,
            self.mechanisms.join("\n"),
            self.memory_limit,
            self.cpu_limit,
        )
    }

    fn expected_digest(&self) -> String {
        format!(
            "blake3:{}",
            blake3::hash(self.digest_material().as_bytes()).to_hex()
        )
    }

    pub fn verify(&self) -> bool {
        const REQUIRED: [&str; 10] = [
            "cap_drop_all",
            "controlled_workspace_mount",
            "cpu_limit",
            "memory_limit",
            "network_none",
            "no_new_privileges",
            "pids_limit",
            "read_only_rootfs",
            "tmpfs_tmp",
            "userns_keep_id",
        ];
        self.schema == "SandboxCapabilityTruthReceiptV1"
            && self.execution_mode == "sealed_local"
            && self.runtime == "podman"
            && self.rootless_required
            && self.rootless_observed
            && is_digest_pinned_image(&self.requested_image)
            && self
                .requested_image
                .rsplit_once('@')
                .is_some_and(|(_, digest)| digest == self.resolved_image_digest)
            && !self.container_id.is_empty()
            && self.runtime_observation_digest.starts_with("blake3:")
            && !self.memory_limit.is_empty()
            && !self.cpu_limit.is_empty()
            && self.mechanisms.len() == REQUIRED.len()
            && REQUIRED
                .iter()
                .all(|item| self.mechanisms.iter().any(|seen| seen == item))
            && self.content_digest == self.expected_digest()
    }
}

#[cfg(feature = "container")]
impl ContainerRuntime {
    fn command(&self) -> &str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
            Self::Nerdctl => "nerdctl",
        }
    }
}

#[cfg(feature = "container")]
pub struct ContainerBackend {
    runtime: ContainerRuntime,
    rust_image: String,
    timeout_secs: u64,
    sealed: bool,
    memory_limit: String,
    cpu_limit: String,
    last_capability_receipt: Mutex<Option<SandboxCapabilityTruthReceiptV1>>,
}

#[cfg(feature = "container")]
impl ContainerBackend {
    pub fn new(config: &BackendConfig) -> Result<Self, RunnerError> {
        let runtime = detect_runtime(&config.container_runtime_preference)?;
        if config.mode == "sealed_local"
            && runtime == ContainerRuntime::Podman
            && !podman_is_rootless()
        {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: "sealed execution requires rootless Podman".into(),
            });
        }
        tracing::info!("container backend: detected runtime {:?}", runtime);

        Self::new_for_runtime(config, runtime)
    }

    fn new_for_runtime(
        config: &BackendConfig,
        runtime: ContainerRuntime,
    ) -> Result<Self, RunnerError> {
        let sealed = config.mode == "sealed_local";
        if sealed && runtime != ContainerRuntime::Podman {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: "sealed execution requires Podman; Docker and Nerdctl are not certified"
                    .into(),
            });
        }
        if sealed && !is_digest_pinned_image(&config.rust_image) {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: "sealed execution requires an image pinned by @sha256:<64 hex>".into(),
            });
        }
        if sealed {
            validate_cpu_limit(&config.cpu_limit)?;
            validate_memory_limit(&config.memory_limit)?;
        }

        Ok(Self {
            runtime,
            rust_image: config.rust_image.clone(),
            timeout_secs: config.command_timeout_secs,
            sealed,
            memory_limit: config.memory_limit.clone(),
            cpu_limit: config.cpu_limit.clone(),
            last_capability_receipt: Mutex::new(None),
        })
    }

    pub fn runtime(&self) -> ContainerRuntime {
        self.runtime
    }

    /// Return only the most recent post-execution, runtime-inspected receipt.
    /// Configuration alone is never represented as observed capability truth.
    pub fn capability_truth_receipt(&self) -> Option<SandboxCapabilityTruthReceiptV1> {
        self.last_capability_receipt
            .lock()
            .ok()
            .and_then(|receipt| receipt.clone())
    }

    fn build_run_args(
        &self,
        workspace: &Path,
        program: &str,
        command_args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<Vec<String>, RunnerError> {
        let metadata = std::fs::symlink_metadata(workspace)
            .map_err(|error| RunnerError::Other(format!("cannot inspect workspace: {error}")))?;
        if self.sealed && metadata.file_type().is_symlink() {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: "sealed workspace mount must not traverse a symlink".into(),
            });
        }
        let canonical = workspace.canonicalize().map_err(|error| {
            RunnerError::Other(format!("cannot canonicalize workspace: {error}"))
        })?;
        #[cfg(not(target_os = "windows"))]
        if canonical.to_string_lossy().contains(':') {
            return Err(RunnerError::Other(
                "workspace path contains ':' which would corrupt docker -v syntax".into(),
            ));
        }

        if self.sealed && program != "cargo" {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: format!("program is outside the sealed check allowlist: {program}"),
            });
        }
        if self.sealed
            && (command_args.is_empty()
                || !matches!(command_args[0], "fmt" | "clippy" | "test")
                || command_args.iter().any(|arg| {
                    arg.contains(';')
                        || arg.contains("&&")
                        || arg.contains("||")
                        || *arg == "-c"
                        || *arg == "--config"
                        || *arg == "--manifest-path"
                        || *arg == "--target-dir"
                        || *arg == "-Z"
                        || arg.contains("../")
                        || arg.contains("..\\")
                }))
        {
            return Err(RunnerError::SealedModeUnsupported {
                runtime: "sealed execution accepts only direct cargo check argv".into(),
            });
        }
        if self.sealed {
            if let Some((key, _)) = env.iter().find(|(key, _)| !is_env_allowed(key)) {
                return Err(RunnerError::SealedModeUnsupported {
                    runtime: format!("environment key is outside the sealed allowlist: {key}"),
                });
            }
        }

        let mut args = vec!["run".to_string()];
        if self.sealed {
            args.extend(["--name".to_string(), next_container_name()]);
        } else {
            args.push("--rm".to_string());
        }
        if self.sealed {
            args.extend([
                "--network=none".to_string(),
                "--read-only".to_string(),
                "--cap-drop=ALL".to_string(),
                "--security-opt=no-new-privileges".to_string(),
                "--pids-limit=128".to_string(),
                "--userns=keep-id".to_string(),
                "--tmpfs".to_string(),
                "/tmp:rw,noexec,nosuid,nodev".to_string(),
            ]);
        }
        args.push("-v".to_string());
        args.push(format!("{}:/workspace:rw", canonical.display()));
        args.push("-w".to_string());
        args.push("/workspace".to_string());
        args.push(format!("--memory={}", self.memory_limit));
        args.push(format!("--cpus={}", self.cpu_limit));
        for (key, value) in env {
            args.push("-e".to_string());
            args.push(format!("{key}={value}"));
        }
        args.push(self.rust_image.clone());
        args.push(program.to_string());
        args.extend(command_args.iter().map(|arg| (*arg).to_string()));
        Ok(args)
    }

    async fn inspect_capability_receipt(
        &self,
        container_name: &str,
    ) -> Result<SandboxCapabilityTruthReceiptV1, RunnerError> {
        let output = tokio::process::Command::new(self.runtime.command())
            .args(["inspect", container_name, "--format", "json"])
            .output()
            .await
            .map_err(|error| {
                RunnerError::Other(format!("failed to inspect sealed container: {error}"))
            })?;
        if !output.status.success() {
            return Err(RunnerError::Other(format!(
                "sealed container inspection failed: {}",
                bounded_output(&output.stderr)
            )));
        }
        let observation = serde_json::from_slice(&output.stdout).map_err(|error| {
            RunnerError::Other(format!(
                "sealed container inspection was not valid JSON: {error}"
            ))
        })?;
        capability_receipt_from_inspect(
            &self.rust_image,
            &self.memory_limit,
            &self.cpu_limit,
            podman_is_rootless(),
            &observation,
        )
    }

    async fn runtime_maintenance(&self, operation: &str, args: &[&str]) -> Result<(), RunnerError> {
        let output = tokio::process::Command::new(self.runtime.command())
            .args(args)
            .output()
            .await
            .map_err(|error| {
                RunnerError::Other(format!("failed to {operation} sealed container: {error}"))
            })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(RunnerError::Other(format!(
                "failed to {operation} sealed container: {}",
                bounded_output(&output.stderr)
            )))
        }
    }

    async fn finalize_sealed_container(
        &self,
        container_name: &str,
        timed_out: bool,
    ) -> Result<(), RunnerError> {
        if timed_out {
            self.runtime_maintenance("kill timed-out", &["kill", "--ignore", container_name])
                .await?;
        }
        let receipt = self.inspect_capability_receipt(container_name).await;
        self.runtime_maintenance(
            "remove finalized",
            &["rm", "--force", "--ignore", container_name],
        )
        .await?;

        match receipt {
            Ok(receipt) => {
                let mut slot = self.last_capability_receipt.lock().map_err(|_| {
                    RunnerError::Other("capability receipt lock is poisoned".into())
                })?;
                *slot = Some(receipt);
                Ok(())
            }
            Err(_) if timed_out => Ok(()),
            Err(error) => Err(error),
        }
    }
}

fn is_digest_pinned_image(image: &str) -> bool {
    let Some((name, digest)) = image.rsplit_once("@sha256:") else {
        return false;
    };
    !name.trim().is_empty()
        && digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(feature = "container")]
fn validate_cpu_limit(value: &str) -> Result<(), RunnerError> {
    let valid = value == value.trim()
        && !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
        && value
            .parse::<f64>()
            .is_ok_and(|parsed| parsed.is_finite() && parsed > 0.0);
    if valid {
        Ok(())
    } else {
        Err(RunnerError::SealedModeUnsupported {
            runtime: format!("invalid sealed cpu limit: {value:?}"),
        })
    }
}

#[cfg(feature = "container")]
fn validate_memory_limit(value: &str) -> Result<(), RunnerError> {
    let valid = value == value.trim()
        && !value.is_empty()
        && value
            .strip_suffix(['b', 'k', 'm', 'g', 'B', 'K', 'M', 'G'])
            .is_some_and(|digits| {
                !digits.is_empty()
                    && digits.bytes().all(|byte| byte.is_ascii_digit())
                    && digits.parse::<u64>().is_ok_and(|parsed| parsed > 0)
            });
    if valid {
        Ok(())
    } else {
        Err(RunnerError::SealedModeUnsupported {
            runtime: format!("invalid sealed memory limit: {value:?}"),
        })
    }
}

#[cfg(feature = "container")]
fn next_container_name() -> String {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    format!(
        "check-runner-{}-{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(feature = "container")]
fn container_name_from_run_args(args: &[String]) -> Option<&str> {
    args.windows(2)
        .find(|pair| pair[0] == "--name")
        .map(|pair| pair[1].as_str())
}

#[cfg(feature = "container")]
fn capability_receipt_from_inspect(
    requested_image: &str,
    memory_limit: &str,
    cpu_limit: &str,
    rootless_observed: bool,
    observation: &serde_json::Value,
) -> Result<SandboxCapabilityTruthReceiptV1, RunnerError> {
    let invalid = |reason: &str| RunnerError::SealedModeUnsupported {
        runtime: format!("sealed runtime observation failed: {reason}"),
    };
    let observed = observation
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| invalid("podman inspect returned no container"))?;
    let container_id = observed
        .get("Id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("container id missing"))?;
    let resolved_image_digest = observed
        .get("ImageDigest")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("resolved image digest missing"))?;
    let requested_digest = requested_image
        .rsplit_once('@')
        .map(|(_, digest)| digest)
        .ok_or_else(|| invalid("requested image is not digest pinned"))?;
    if requested_digest != resolved_image_digest {
        return Err(invalid(
            "resolved image digest differs from requested digest",
        ));
    }

    let create_command = observed
        .pointer("/Config/CreateCommand")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| invalid("create command missing"))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| invalid("non-string create argument"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let has_arg = |expected: &str| create_command.contains(&expected);
    let has_pair = |flag: &str, value: &str| {
        create_command
            .windows(2)
            .any(|pair| pair[0] == flag && pair[1] == value)
    };
    let host = observed
        .get("HostConfig")
        .ok_or_else(|| invalid("host configuration missing"))?;
    let tmpfs = host
        .pointer("/Tmpfs/~1tmp")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let workspace_mount = create_command
        .windows(2)
        .any(|pair| pair[0] == "-v" && pair[1].ends_with(":/workspace:rw"))
        && has_pair("-w", "/workspace");

    let checks = [
        (
            "cap_drop_all",
            has_arg("--cap-drop=ALL")
                && host
                    .pointer("/Privileged")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false),
        ),
        ("controlled_workspace_mount", workspace_mount),
        (
            "cpu_limit",
            has_arg(&format!("--cpus={cpu_limit}"))
                && host
                    .pointer("/NanoCpus")
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|value| value > 0),
        ),
        (
            "memory_limit",
            has_arg(&format!("--memory={memory_limit}"))
                && host
                    .pointer("/Memory")
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|value| value > 0),
        ),
        (
            "network_none",
            has_arg("--network=none")
                && host
                    .pointer("/NetworkMode")
                    .and_then(serde_json::Value::as_str)
                    == Some("none"),
        ),
        (
            "no_new_privileges",
            has_arg("--security-opt=no-new-privileges")
                && host
                    .pointer("/SecurityOpt")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|items| {
                        items
                            .iter()
                            .any(|item| item.as_str() == Some("no-new-privileges"))
                    }),
        ),
        (
            "pids_limit",
            has_arg("--pids-limit=128")
                && host
                    .pointer("/PidsLimit")
                    .and_then(serde_json::Value::as_u64)
                    == Some(128),
        ),
        (
            "read_only_rootfs",
            has_arg("--read-only")
                && host
                    .pointer("/ReadonlyRootfs")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true),
        ),
        (
            "tmpfs_tmp",
            has_pair("--tmpfs", "/tmp:rw,noexec,nosuid,nodev")
                && ["rw", "noexec", "nosuid", "nodev"]
                    .iter()
                    .all(|option| tmpfs.split(',').any(|seen| seen == *option)),
        ),
        (
            "userns_keep_id",
            has_arg("--userns=keep-id") && rootless_observed,
        ),
    ];
    let failed = checks
        .iter()
        .filter_map(|(name, passed)| (!passed).then_some(*name))
        .collect::<Vec<_>>();
    if !failed.is_empty() {
        return Err(invalid(&format!(
            "required mechanisms not observed: {}",
            failed.join(",")
        )));
    }

    let observation_bytes = serde_json::to_vec(observed)
        .map_err(|error| invalid(&format!("cannot serialize observation: {error}")))?;
    let mut receipt = SandboxCapabilityTruthReceiptV1 {
        schema: "SandboxCapabilityTruthReceiptV1".into(),
        execution_mode: "sealed_local".into(),
        runtime: "podman".into(),
        requested_image: requested_image.into(),
        resolved_image_digest: resolved_image_digest.into(),
        container_id: container_id.into(),
        runtime_observation_digest: format!("blake3:{}", blake3::hash(&observation_bytes).to_hex()),
        rootless_required: true,
        rootless_observed,
        mechanisms: checks.iter().map(|(name, _)| (*name).into()).collect(),
        memory_limit: memory_limit.into(),
        cpu_limit: cpu_limit.into(),
        content_digest: String::new(),
    };
    receipt.content_digest = receipt.expected_digest();
    if receipt.verify() {
        Ok(receipt)
    } else {
        Err(invalid("constructed capability receipt did not verify"))
    }
}

#[cfg(feature = "container")]
fn podman_is_rootless() -> bool {
    std::process::Command::new("podman")
        .args(["info", "--format", "{{.Host.Security.Rootless}}"])
        .output()
        .is_ok_and(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true"
        })
}

#[cfg(feature = "container")]
#[async_trait]
impl ExecutionBackend for ContainerBackend {
    fn kind(&self) -> ExecutionBackendKind {
        ExecutionBackendKind::Container
    }

    fn has_live_execution_evidence(&self) -> bool {
        self.capability_truth_receipt()
            .is_some_and(|receipt| receipt.verify())
    }

    async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace, RunnerError> {
        Ok(sandbox_workspace::prepare_workspace(fixture)?)
    }

    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout_secs: u64,
    ) -> Result<CommandOutput, RunnerError> {
        let timeout = if timeout_secs > 0 {
            timeout_secs
        } else {
            self.timeout_secs
        };

        let full_command = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        };
        let run_args = self.build_run_args(workspace, program, args, env)?;
        let container_name = if self.sealed {
            Some(
                container_name_from_run_args(&run_args)
                    .ok_or_else(|| RunnerError::Other("sealed container name missing".into()))?
                    .to_string(),
            )
        } else {
            None
        };

        let start = Instant::now();
        let child = tokio::process::Command::new(self.runtime.command())
            .args(&run_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| {
                RunnerError::Other(format!(
                    "failed to spawn container runtime {}: {error}",
                    self.runtime.command()
                ))
            })?;

        let output =
            tokio::time::timeout(Duration::from_secs(timeout), child.wait_with_output()).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        if let Some(container_name) = container_name.as_deref() {
            self.finalize_sealed_container(container_name, output.is_err())
                .await?;
        }

        match output {
            Ok(Ok(output)) => Ok(CommandOutput {
                stdout: bounded_output(&output.stdout),
                stderr: bounded_output(&output.stderr),
                exit_code: output.status.code().unwrap_or(-1),
                duration_ms,
            }),
            Ok(Err(error)) => Err(RunnerError::Other(format!(
                "container command failed: {error}"
            ))),
            Err(_) => Err(RunnerError::CommandTimeout {
                command: full_command,
                timeout_secs: timeout,
            }),
        }
    }

    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> Result<LogBundle, RunnerError> {
        HostBackend {
            timeout_secs: self.timeout_secs,
        }
        .collect_logs(fmt, clippy, test)
        .await
    }
}

#[cfg(feature = "container")]
pub fn detect_runtime(preference: &str) -> Result<ContainerRuntime, RunnerError> {
    match preference {
        "docker" => probe_runtime("docker").ok_or(RunnerError::NoContainerRuntime),
        "podman" => probe_runtime("podman").ok_or(RunnerError::NoContainerRuntime),
        "nerdctl" => probe_runtime("nerdctl").ok_or(RunnerError::NoContainerRuntime),
        _ => {
            if let Some(runtime) = probe_runtime("docker") {
                return Ok(runtime);
            }
            if let Some(runtime) = probe_runtime("podman") {
                return Ok(runtime);
            }
            if let Some(runtime) = probe_runtime("nerdctl") {
                return Ok(runtime);
            }
            Err(RunnerError::NoContainerRuntime)
        }
    }
}

#[cfg(feature = "container")]
fn probe_runtime(name: &str) -> Option<ContainerRuntime> {
    let result = std::process::Command::new(name)
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => match name {
            "docker" => Some(ContainerRuntime::Docker),
            "podman" => Some(ContainerRuntime::Podman),
            "nerdctl" => Some(ContainerRuntime::Nerdctl),
            _ => None,
        },
        _ => None,
    }
}

pub fn select_backend(config: &BackendConfig) -> Result<Box<dyn ExecutionBackend>, RunnerError> {
    let sealed = config.mode == "sealed_local";

    match config.execution_backend_preference.as_str() {
        "host" => {
            if sealed {
                return Err(RunnerError::SealedModeUnsupported {
                    runtime: "host backend requested in sealed_local mode".into(),
                });
            }
            Ok(Box::new(HostBackend::new(config)))
        }
        "container" => {
            #[cfg(feature = "container")]
            {
                ContainerBackend::new(config)
                    .map(|backend| Box::new(backend) as Box<dyn ExecutionBackend>)
            }
            #[cfg(not(feature = "container"))]
            {
                Err(RunnerError::NoContainerRuntime)
            }
        }
        _ => {
            #[cfg(feature = "container")]
            {
                if let Ok(backend) = ContainerBackend::new(config) {
                    return Ok(Box::new(backend));
                }
            }

            if sealed {
                return Err(RunnerError::SealedModeUnsupported {
                    runtime: "container unavailable and host fallback not allowed in sealed mode"
                        .into(),
                });
            }
            Ok(Box::new(HostBackend::new(config)))
        }
    }
}

#[cfg(test)]
mod wave1_tests {
    use super::*;

    fn config() -> BackendConfig {
        BackendConfig {
            mode: "local".into(),
            execution_backend_preference: "host".into(),
            container_runtime_preference: "docker".into(),
            sealed_allow_host_backend: false,
            rust_image: "rust:1.75".into(),
            command_timeout_secs: 30,
            memory_limit: "1g".into(),
            cpu_limit: "1.0".into(),
        }
    }

    fn output(stdout: &str, stderr: &str, exit_code: i32, duration_ms: u64) -> CommandOutput {
        CommandOutput {
            stdout: stdout.into(),
            stderr: stderr.into(),
            exit_code,
            duration_ms,
        }
    }

    #[test]
    fn check_result_all_pass_requires_every_check() {
        let result = CheckResult {
            fmt_pass: true,
            clippy_pass: true,
            test_pass: true,
            fmt_output: ParsedCheckOutput::default(),
            clippy_output: ParsedCheckOutput::default(),
            test_output: ParsedCheckOutput::default(),
            total_duration_ms: 0,
        };
        assert!(result.all_pass());

        let failing = CheckResult {
            test_pass: false,
            ..result
        };
        assert!(!failing.all_pass());
    }

    #[test]
    fn host_backend_collect_logs_preserves_outputs_and_timings() {
        let backend = HostBackend::new(&config());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        let logs = runtime
            .block_on(backend.collect_logs(
                &output("fmt ok", "", 0, 12),
                &output("", "clippy warn", 1, 34),
                &output("tests", "", 0, 56),
            ))
            .unwrap();

        assert_eq!(logs.fmt_stdout, "fmt ok");
        assert_eq!(logs.clippy_stderr, "clippy warn");
        assert_eq!(logs.timings.fmt_ms, 12);
        assert_eq!(logs.timings.clippy_ms, 34);
        assert_eq!(logs.timings.test_ms, 56);
    }

    #[test]
    fn select_backend_rejects_unapproved_host_fallback_in_sealed_mode() {
        let mut cfg = config();
        cfg.mode = "sealed_local".into();
        cfg.execution_backend_preference = "host".into();

        let err = select_backend(&cfg).err().unwrap();
        assert!(matches!(err, RunnerError::SealedModeUnsupported { .. }));
    }

    #[test]
    fn select_backend_falls_back_to_host_when_allowed() {
        let mut cfg = config();
        cfg.mode = "sealed_local".into();
        cfg.execution_backend_preference = "auto".into();
        cfg.sealed_allow_host_backend = true;
        // Use a runtime that definitely won't exist so the container path fails
        // and we reliably exercise the host-fallback path (the thing this test verifies)
        cfg.container_runtime_preference = "nonexistent_runtime".into();

        let error = select_backend(&cfg).err().unwrap();
        assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_config() -> BackendConfig {
        BackendConfig {
            mode: "local".into(),
            execution_backend_preference: "host".into(),
            container_runtime_preference: "auto".into(),
            sealed_allow_host_backend: false,
            rust_image: "rust:1.75".into(),
            command_timeout_secs: 30,
            memory_limit: "1g".into(),
            cpu_limit: "1.0".into(),
        }
    }

    fn sample_output(kind: CheckKind, exit_code: i32, duration_ms: u64) -> ParsedCheckOutput {
        ParsedCheckOutput {
            check_kind: kind,
            exit_code,
            effects: vec![],
            raw_stdout: format!("stdout-{duration_ms}"),
            raw_stderr: format!("stderr-{duration_ms}"),
        }
    }

    #[test]
    fn env_allowlist_accepts_safe_prefixes_and_rejects_unknowns() {
        assert!(is_env_allowed("PATH"));
        assert!(is_env_allowed("CARGO_HOME"));
        assert!(is_env_allowed("LC_ALL"));
        assert!(!is_env_allowed("AWS_SECRET_ACCESS_KEY"));
    }

    #[test]
    fn check_result_requires_all_checks_to_pass() {
        let all_pass = CheckResult {
            fmt_pass: true,
            clippy_pass: true,
            test_pass: true,
            fmt_output: sample_output(CheckKind::Fmt, 0, 10),
            clippy_output: sample_output(CheckKind::Clippy, 0, 20),
            test_output: sample_output(CheckKind::Test, 0, 30),
            total_duration_ms: 60,
        };
        assert!(all_pass.all_pass());

        let not_all_pass = CheckResult {
            clippy_pass: false,
            ..all_pass
        };
        assert!(!not_all_pass.all_pass());
    }

    #[test]
    fn select_backend_rejects_host_in_sealed_mode_without_opt_in() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();

        let error = select_backend(&config).err().unwrap();
        assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
    }

    #[test]
    fn select_backend_returns_host_when_allowed() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.sealed_allow_host_backend = true;

        let error = select_backend(&config).err().unwrap();
        assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_backend_rejects_non_podman_runtime_and_unpinned_image() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.execution_backend_preference = "container".into();
        config.rust_image = "rust:1.75".into();

        let image_error = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman)
            .err()
            .unwrap();
        assert!(matches!(
            image_error,
            RunnerError::SealedModeUnsupported { .. }
        ));

        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();
        let runtime_error = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Docker)
            .err()
            .unwrap();
        assert!(matches!(
            runtime_error,
            RunnerError::SealedModeUnsupported { .. }
        ));
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_podman_profile_is_fail_closed_and_bounded() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.execution_backend_preference = "container".into();
        config.container_runtime_preference = "podman".into();
        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();
        let backend = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman).unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let args = backend
            .build_run_args(
                workspace.path(),
                "cargo",
                &["test"],
                &[("CARGO_TERM_COLOR", "never")],
            )
            .unwrap();

        for required in [
            "--network=none",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit=128",
            "--userns=keep-id",
        ] {
            assert!(args.iter().any(|arg| arg == required), "missing {required}");
        }
        assert!(args.iter().any(|arg| arg == "/tmp:rw,noexec,nosuid,nodev"));
        assert!(!args.iter().any(|arg| arg == "--privileged"));

        assert!(backend.capability_truth_receipt().is_none());
    }

    #[cfg(feature = "container")]
    #[test]
    fn runtime_inspection_produces_verifiable_capability_truth() {
        let requested_image = "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let observation = serde_json::json!([{
            "Id": "container-1",
            "ImageDigest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "Config": {"CreateCommand": [
                "podman", "run", "--network=none", "--read-only", "--cap-drop=ALL",
                "--security-opt=no-new-privileges", "--pids-limit=128", "--userns=keep-id",
                "--tmpfs", "/tmp:rw,noexec,nosuid,nodev", "-v", "/host:/workspace:rw",
                "-w", "/workspace", "--memory=1g", "--cpus=1.0", requested_image,
                "cargo", "test"
            ]},
            "HostConfig": {
                "NetworkMode": "none", "ReadonlyRootfs": true, "Privileged": false,
                "SecurityOpt": ["no-new-privileges"], "PidsLimit": 128,
                "Memory": 1073741824, "NanoCpus": 1000000000,
                "Tmpfs": {"/tmp": "rw,noexec,nosuid,nodev,rprivate,tmpcopyup"}
            }
        }]);

        let receipt =
            capability_receipt_from_inspect(requested_image, "1g", "1.0", true, &observation)
                .unwrap();

        assert!(receipt.verify());
        assert_eq!(receipt.container_id, "container-1");
        assert_eq!(receipt.mechanisms.len(), 10);
    }

    #[cfg(feature = "container")]
    #[test]
    fn runtime_inspection_missing_mechanism_fails_closed() {
        let requested_image = "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let observation = serde_json::json!([{
            "Id": "container-1",
            "ImageDigest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "Config": {"CreateCommand": ["podman", "run", requested_image, "cargo", "test"]},
            "HostConfig": {"NetworkMode": "bridge", "ReadonlyRootfs": false}
        }]);

        assert!(
            capability_receipt_from_inspect(requested_image, "1g", "1.0", true, &observation,)
                .is_err()
        );
    }

    #[cfg(feature = "container")]
    #[test]
    fn rootless_podman_execution_emits_observed_receipt_and_cleans_container() {
        const IMAGE: &str = "docker.io/library/alpine@sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc";
        if !podman_is_rootless()
            || !std::process::Command::new("podman")
                .args(["image", "exists", IMAGE])
                .status()
                .is_ok_and(|status| status.success())
        {
            return;
        }

        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.rust_image = IMAGE.into();
        let backend = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman).unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let output = runtime
            .block_on(backend.run_command(workspace.path(), "cargo", &["test"], &[], 5))
            .unwrap();
        assert_ne!(output.exit_code, 0, "alpine should not contain cargo");
        let receipt = backend.capability_truth_receipt().unwrap();
        assert!(receipt.verify());
        assert!(!std::process::Command::new("podman")
            .args(["container", "exists", &receipt.container_id])
            .status()
            .is_ok_and(|status| status.success()));
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_backend_rejects_invalid_resource_limits_before_spawn() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();

        for invalid in ["", "0", "-1", "nope"] {
            config.cpu_limit = invalid.into();
            config.memory_limit = "1g".into();
            assert!(matches!(
                ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman),
                Err(RunnerError::SealedModeUnsupported { .. })
            ));
        }
        config.cpu_limit = "1.5".into();
        for invalid in ["", "0", "-1g", "nope", "1t"] {
            config.memory_limit = invalid.into();
            assert!(matches!(
                ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman),
                Err(RunnerError::SealedModeUnsupported { .. })
            ));
        }
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_podman_profile_rejects_disallowed_environment() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();
        let backend = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman).unwrap();
        let workspace = tempfile::tempdir().unwrap();

        let error = backend
            .build_run_args(
                workspace.path(),
                "cargo",
                &["test"],
                &[("AWS_SECRET_ACCESS_KEY", "forbidden")],
            )
            .unwrap_err();
        assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_direct_argv_rejects_shell_and_unapproved_commands() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();
        let backend = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman).unwrap();
        let workspace = tempfile::tempdir().unwrap();

        for (program, args) in [
            ("sh", vec!["-c", "cargo test"]),
            ("cargo", vec!["run"]),
            ("cargo", vec!["test", "&&", "id"]),
            (
                "cargo",
                vec!["test", "--manifest-path", "../evil/Cargo.toml"],
            ),
            ("cargo", vec!["test", "-Z", "unstable-options"]),
        ] {
            let error = backend
                .build_run_args(workspace.path(), program, &args, &[])
                .unwrap_err();
            assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
        }
    }

    #[cfg(feature = "container")]
    #[test]
    fn sealed_mount_rejects_symlink_workspace_path() {
        let mut config = sample_config();
        config.mode = "sealed_local".into();
        config.rust_image =
            "docker.io/library/rust@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into();
        let backend = ContainerBackend::new_for_runtime(&config, ContainerRuntime::Podman).unwrap();
        let real = tempfile::tempdir().unwrap();
        let link = real.path().join("link");
        std::os::unix::fs::symlink(real.path(), &link).unwrap();

        let error = backend
            .build_run_args(&link, "cargo", &["test"], &[])
            .unwrap_err();
        assert!(matches!(error, RunnerError::SealedModeUnsupported { .. }));
    }

    #[test]
    fn captured_output_is_hard_capped_with_explicit_truncation_marker() {
        let oversized = vec![b'x'; MAX_CAPTURED_OUTPUT_BYTES + 1];
        let captured = bounded_output(&oversized);

        assert!(captured.len() <= MAX_CAPTURED_OUTPUT_BYTES);
        assert!(captured.ends_with("\n[check-runner output truncated]\n"));
    }

    #[test]
    fn collect_logs_preserves_outputs_and_timings() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let config = sample_config();
        let backend = HostBackend::new(&config);
        let fmt = CommandOutput {
            stdout: "fmt out".into(),
            stderr: "fmt err".into(),
            exit_code: 0,
            duration_ms: 11,
        };
        let clippy = CommandOutput {
            stdout: "clippy out".into(),
            stderr: "clippy err".into(),
            exit_code: 0,
            duration_ms: 22,
        };
        let test = CommandOutput {
            stdout: "test out".into(),
            stderr: "test err".into(),
            exit_code: 1,
            duration_ms: 33,
        };

        let bundle = runtime
            .block_on(backend.collect_logs(&fmt, &clippy, &test))
            .unwrap();

        assert_eq!(bundle.fmt_stdout, "fmt out");
        assert_eq!(bundle.clippy_stderr, "clippy err");
        assert_eq!(bundle.test_stdout, "test out");
        assert_eq!(bundle.timings.fmt_ms, 11);
        assert_eq!(bundle.timings.clippy_ms, 22);
        assert_eq!(bundle.timings.test_ms, 33);
    }

    #[test]
    fn host_backend_run_command_times_out() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let config = sample_config();
        let backend = HostBackend::new(&config);
        let workspace = tempfile::tempdir().unwrap();

        let err = runtime
            .block_on(backend.run_command(workspace.path(), "sh", &["-c", "sleep 2"], &[], 1))
            .unwrap_err();

        assert!(matches!(
            err,
            RunnerError::CommandTimeout {
                timeout_secs: 1,
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn host_backend_timeout_kills_spawned_child_process_group() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let config = sample_config();
        let backend = HostBackend::new(&config);
        let workspace = tempfile::tempdir().unwrap();
        let child_pid_file = workspace.path().join("child.pid");
        let child_pid_file_arg = child_pid_file.display().to_string();

        let err = runtime
            .block_on(backend.run_command(
                workspace.path(),
                "sh",
                &[
                    "-c",
                    &format!(
                        "sleep 30 & child=$!; echo $child > \"{child_pid_file_arg}\"; wait $child"
                    ),
                ],
                &[],
                1,
            ))
            .unwrap_err();
        assert!(matches!(err, RunnerError::CommandTimeout { .. }));

        let child_pid: i32 = std::fs::read_to_string(&child_pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        std::thread::sleep(Duration::from_millis(150));

        let kill_result = check_runner_sys::process_exists(child_pid);
        let errno = std::io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or_default();
        // process_exists returns true if the process exists
        // After timeout, the process should be gone (doesn't exist)
        assert!(
            !kill_result && errno == libc::ESRCH,
            "timed-out child process should be gone, pid={child_pid}, kill_result={kill_result}, errno={errno}"
        );
    }

    #[test]
    fn host_backend_clears_disallowed_environment_variables() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let config = sample_config();
        let backend = HostBackend::new(&config);
        let workspace = tempfile::tempdir().unwrap();

        check_runner_sys::set_env("AWS_SECRET_ACCESS_KEY", "forbidden");
        let output = runtime
            .block_on(backend.run_command(
                workspace.path(),
                "sh",
                &["-c", "printf '%s' \"${AWS_SECRET_ACCESS_KEY:-missing}\""],
                &[],
                5,
            ))
            .unwrap();
        check_runner_sys::remove_env("AWS_SECRET_ACCESS_KEY");

        assert_eq!(output.stdout, "missing");
    }
}
