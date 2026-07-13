//! # check-runner
//!
//! Host and container execution surface for patch verification.
//!
//! This Tier 3 crate prepares workspaces, runs validation commands, captures
//! effects, and normalizes command output for higher-level scoring and
//! attribution. It is execution infrastructure, not a policy owner.

use std::path::Path;
#[cfg(feature = "container")]
use std::process::Stdio;
use std::time::{Duration, Instant};

use async_trait::async_trait;

pub use effect_signature::{EffectSignature, LocatedEffect};
pub use sandbox_workspace::{PatchedWorkspace, Workspace};

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

    #[error("failed to terminate process group {pgid}: {source}")]
    KillProcessGroup {
        pgid: i32,
        #[source]
        source: std::io::Error,
    },

    #[error("sealed mode unsupported for runtime: {runtime}")]
    SealedModeUnsupported { runtime: String },

    #[error("no container runtime found")]
    NoContainerRuntime,

    #[error("container cleanup failed for {container}: {reason}")]
    ContainerCleanup { container: String, reason: String },

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
            if !is_env_allowed(key) {
                return Err(RunnerError::Other(format!(
                    "caller environment key is not allowed: {key}"
                )));
            }
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
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
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
                let pgid = child_pid.expect("child PID must be Some immediately after spawn");
                let kill_result = check_runner_sys::kill_process_group(pgid);
                tokio::time::timeout(Duration::from_secs(5), &mut wait)
                    .await
                    .map_err(|_| {
                        RunnerError::Other(format!(
                            "timed out reaping process group leader {pgid} after SIGKILL"
                        ))
                    })?
                    .map_err(|error| {
                        RunnerError::Other(format!(
                            "failed to reap process group leader {pgid}: {error}"
                        ))
                    })?;
                kill_result.map_err(|source| RunnerError::KillProcessGroup { pgid, source })?;

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

#[cfg(feature = "container")]
impl ContainerRuntime {
    fn command(&self) -> &str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
            Self::Nerdctl => "nerdctl",
        }
    }

    fn network_flag(&self) -> &str {
        match self {
            Self::Docker | Self::Podman => "--network=none",
            Self::Nerdctl => "--net=none",
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
}

#[cfg(feature = "container")]
impl ContainerBackend {
    pub fn new(config: &BackendConfig) -> Result<Self, RunnerError> {
        let runtime = detect_runtime(&config.container_runtime_preference)?;
        tracing::info!("container backend: detected runtime {:?}", runtime);

        Ok(Self {
            runtime,
            rust_image: config.rust_image.clone(),
            timeout_secs: config.command_timeout_secs,
            sealed: config.mode == "sealed_local",
            memory_limit: config.memory_limit.clone(),
            cpu_limit: config.cpu_limit.clone(),
        })
    }

    pub fn runtime(&self) -> ContainerRuntime {
        self.runtime
    }

    fn build_run_args(
        &self,
        workspace: &Path,
        program: &str,
        command_args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<Vec<String>, RunnerError> {
        if !is_valid_image_reference(&self.rust_image) {
            return Err(RunnerError::Other(format!(
                "container image is invalid or resembles an option: {}",
                self.rust_image
            )));
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

        let output = canonical.join("output");
        if self.sealed {
            std::fs::create_dir_all(&output)?;
        }
        let unique = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let name = format!("check-runner-{unique}");
        let cidfile = std::env::temp_dir().join(format!("{name}.cid"));
        let mut args = vec!["run".to_string(), "--rm".to_string()];
        args.push("--pull=never".to_string());
        args.push("--name".to_string());
        args.push(name);
        args.push("--cidfile".to_string());
        args.push(cidfile.display().to_string());
        args.push("-v".to_string());
        args.push(format!(
            "{}:/workspace:{}",
            canonical.display(),
            if self.sealed { "ro" } else { "rw" }
        ));
        if self.sealed {
            args.push("-v".to_string());
            args.push(format!("{}:/workspace/output:rw", output.display()));
        }
        args.push("-w".to_string());
        args.push("/workspace".to_string());
        args.push(format!("--memory={}", self.memory_limit));
        args.push(format!("--cpus={}", self.cpu_limit));
        if self.sealed {
            args.push(self.runtime.network_flag().to_string());
            args.push("-e".to_string());
            args.push("CARGO_TARGET_DIR=/workspace/output/target".to_string());
        }
        for (key, value) in env {
            if !is_env_allowed(key) {
                return Err(RunnerError::Other(format!(
                    "caller environment key is not allowed: {key}"
                )));
            }
            args.push("-e".to_string());
            args.push(format!("{key}={value}"));
        }
        args.push(self.rust_image.clone());
        args.push(program.to_string());
        args.extend(command_args.iter().map(|arg| (*arg).to_string()));
        Ok(args)
    }
}

#[cfg(feature = "container")]
#[async_trait]
impl ExecutionBackend for ContainerBackend {
    fn kind(&self) -> ExecutionBackendKind {
        ExecutionBackendKind::Container
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
        let container_name = run_args
            .windows(2)
            .find(|pair| pair[0] == "--name")
            .map(|pair| pair[1].clone())
            .expect("build_run_args always assigns a container name");
        let cidfile = run_args
            .windows(2)
            .find(|pair| pair[0] == "--cidfile")
            .map(|pair| pair[1].clone())
            .expect("build_run_args always assigns a cidfile");

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
        if output.is_ok() {
            let _ = std::fs::remove_file(&cidfile);
        }

        match output {
            Ok(Ok(output)) => Ok(CommandOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                duration_ms,
            }),
            Ok(Err(error)) => Err(RunnerError::Other(format!(
                "container command failed: {error}"
            ))),
            Err(_) => {
                let cleanup = cleanup_container(self.runtime, &container_name).await;
                let _ = std::fs::remove_file(cidfile);
                match cleanup {
                    Ok(()) => Err(RunnerError::CommandTimeout {
                        command: full_command,
                        timeout_secs: timeout,
                    }),
                    Err(reason) => Err(RunnerError::ContainerCleanup {
                        container: container_name,
                        reason,
                    }),
                }
            }
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
fn is_valid_image_reference(image: &str) -> bool {
    !image.is_empty()
        && !image.starts_with('-')
        && image.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'-' | b'/' | b':' | b'@' | b'+')
        })
}

#[cfg(feature = "container")]
async fn runtime_output(
    runtime: ContainerRuntime,
    args: &[&str],
) -> std::io::Result<std::process::Output> {
    let mut command = tokio::process::Command::new(runtime.command());
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command.output().await
}

#[cfg(feature = "container")]
async fn cleanup_container(runtime: ContainerRuntime, name: &str) -> Result<(), String> {
    const CLEANUP_BOUND: Duration = Duration::from_secs(10);
    tokio::time::timeout(CLEANUP_BOUND, async {
        // `podman rm -f` may honor its default stop grace period. An explicit
        // kill is immediate and is supported by both Docker and Podman.
        let _ = runtime_output(runtime, &["kill", name]).await;
        let remove = runtime_output(runtime, &["rm", "-f", name])
            .await
            .map_err(|error| format!("could not start force-remove: {error}"))?;

        let inspect = runtime_output(runtime, &["inspect", name])
            .await
            .map_err(|error| format!("could not confirm removal: {error}"))?;
        if inspect.status.success() {
            return Err("force-remove returned but the container still exists".to_string());
        }

        // A failed remove is acceptable only when inspect independently confirms absence.
        let _ = remove;
        Ok(())
    })
    .await
    .map_err(|_| format!("cleanup and removal confirmation exceeded {CLEANUP_BOUND:?}"))?
}

#[cfg(feature = "container")]
pub fn detect_runtime(preference: &str) -> Result<ContainerRuntime, RunnerError> {
    match preference {
        "docker" => probe_runtime("docker").ok_or(RunnerError::NoContainerRuntime),
        "podman" => probe_runtime("podman").ok_or(RunnerError::NoContainerRuntime),
        "nerdctl" => probe_runtime("nerdctl").ok_or(RunnerError::NoContainerRuntime),
        "auto" | "" => {
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
        _ => Err(RunnerError::Other(format!(
            "invalid container runtime preference: {preference}"
        ))),
    }
}

#[cfg(feature = "container")]
fn probe_runtime(name: &str) -> Option<ContainerRuntime> {
    let runtime = match name {
        "docker" => ContainerRuntime::Docker,
        "podman" => ContainerRuntime::Podman,
        "nerdctl" => ContainerRuntime::Nerdctl,
        _ => return None,
    };
    let mut child = std::process::Command::new(name)
        .arg("info")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };

    status.success().then_some(runtime)
}

pub fn select_backend(config: &BackendConfig) -> Result<Box<dyn ExecutionBackend>, RunnerError> {
    let sealed = config.mode == "sealed_local";

    match config.execution_backend_preference.as_str() {
        "host" => {
            if sealed && !config.sealed_allow_host_backend {
                return Err(RunnerError::SealedModeUnsupported {
                    runtime: "host backend requested in sealed_local mode".into(),
                });
            }
            if sealed {
                tracing::warn!("sealed_local mode with host backend — no network isolation");
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

            if sealed && !config.sealed_allow_host_backend {
                return Err(RunnerError::SealedModeUnsupported {
                    runtime: "container unavailable and host fallback not allowed in sealed mode"
                        .into(),
                });
            }
            if sealed {
                tracing::warn!(
                    "sealed_local: container unavailable, falling back to host — isolation not guaranteed"
                );
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

        let backend = select_backend(&cfg).unwrap();
        assert_eq!(backend.kind(), ExecutionBackendKind::Host);
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

        let backend = select_backend(&config).unwrap();
        assert_eq!(backend.kind(), ExecutionBackendKind::Host);
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

        check_runner_sys::set_env("AWS_SECRET_ACCESS_KEY", "forbidden").unwrap();
        let output = runtime
            .block_on(backend.run_command(
                workspace.path(),
                "sh",
                &["-c", "printf '%s' \"${AWS_SECRET_ACCESS_KEY:-missing}\""],
                &[],
                5,
            ))
            .unwrap();
        check_runner_sys::remove_env("AWS_SECRET_ACCESS_KEY").unwrap();

        assert_eq!(output.stdout, "missing");
    }
}

#[cfg(all(test, feature = "container"))]
mod container_hardening_tests {
    use super::*;

    fn backend(sealed: bool) -> ContainerBackend {
        ContainerBackend {
            runtime: ContainerRuntime::Docker,
            rust_image: "rust:test".into(),
            timeout_secs: 1,
            sealed,
            memory_limit: "1g".into(),
            cpu_limit: "1".into(),
        }
    }

    #[test]
    fn structured_argv_is_not_interpreted_by_a_shell() {
        let dir = tempfile::tempdir().unwrap();
        let args = backend(false)
            .build_run_args(dir.path(), "printf", &["%s", "$(touch pwned)", "a b"], &[])
            .unwrap();
        assert!(!args.windows(2).any(|pair| pair == ["sh", "-c"]));
        assert_eq!(
            &args[args.len() - 4..],
            ["printf", "%s", "$(touch pwned)", "a b"]
        );
    }

    #[test]
    fn caller_environment_must_pass_allowlist() {
        let dir = tempfile::tempdir().unwrap();
        let error = backend(false)
            .build_run_args(
                dir.path(),
                "true",
                &[],
                &[("AWS_SECRET_ACCESS_KEY", "secret")],
            )
            .unwrap_err();
        assert!(
            matches!(error, RunnerError::Other(message) if message.contains("environment key"))
        );
    }

    #[test]
    fn runtime_preference_rejects_option_injection() {
        let error = detect_runtime("--help").unwrap_err();
        assert!(
            matches!(error, RunnerError::Other(message) if message.contains("runtime preference"))
        );
    }

    #[test]
    fn image_rejects_option_injection() {
        let dir = tempfile::tempdir().unwrap();
        let mut injected = backend(false);
        injected.rust_image = "--privileged".into();
        let error = injected
            .build_run_args(dir.path(), "true", &[], &[])
            .unwrap_err();
        assert!(matches!(error, RunnerError::Other(message) if message.contains("image")));
    }

    #[test]
    fn image_rejects_whitespace_and_control_characters() {
        let dir = tempfile::tempdir().unwrap();
        for image in ["alpine --privileged", "alpine\n--privileged", ""] {
            let mut injected = backend(false);
            injected.rust_image = image.into();
            assert!(injected
                .build_run_args(dir.path(), "true", &[], &[])
                .is_err());
        }
    }

    #[test]
    fn sealed_mount_has_read_only_source_and_writable_output() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("output")).unwrap();
        let args = backend(true)
            .build_run_args(dir.path(), "true", &[], &[])
            .unwrap();
        let joined = args.join("\n");
        assert!(joined.contains(":/workspace:ro"));
        assert!(joined.contains(":/workspace/output:rw"));
        assert!(joined.contains("CARGO_TARGET_DIR=/workspace/output/target"));
    }

    #[test]
    fn run_args_track_name_and_cidfile_for_timeout_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let args = backend(false)
            .build_run_args(dir.path(), "true", &[], &[])
            .unwrap();
        assert!(args.iter().any(|arg| arg == "--name"));
        assert!(args.iter().any(|arg| arg == "--cidfile"));
    }

    fn live_backend(sealed: bool) -> Option<ContainerBackend> {
        let runtime = match detect_runtime("auto") {
            Ok(runtime) => runtime,
            Err(RunnerError::NoContainerRuntime) => {
                eprintln!("skipping: no Docker/Podman/nerdctl runtime available");
                return None;
            }
            Err(error) => panic!("runtime detection failed: {error}"),
        };
        Some(ContainerBackend {
            runtime,
            rust_image: std::env::var("CHECK_RUNNER_TEST_IMAGE")
                .unwrap_or_else(|_| "alpine:3.20".into()),
            timeout_secs: 30,
            sealed,
            memory_limit: "1g".into(),
            cpu_limit: "1".into(),
        })
    }

    #[test]
    fn live_runtime_enforces_timeout_and_cleans_up() {
        let Some(backend) = live_backend(false) else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let error = runtime
            .block_on(backend.run_command(dir.path(), "sh", &["-c", "sleep 30"], &[], 1))
            .unwrap_err();
        assert!(
            matches!(error, RunnerError::CommandTimeout { .. }),
            "unexpected timeout lifecycle error: {error:?}"
        );
    }

    #[test]
    fn live_runtime_mounts_sealed_workspace_with_writable_target() {
        let Some(backend) = live_backend(true) else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("input.txt"), "mounted").unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let result = runtime
            .block_on(backend.run_command(
                dir.path(),
                "sh",
                &[
                    "-c",
                    "test \"$(cat input.txt)\" = mounted && test \"$CARGO_TARGET_DIR\" = /workspace/output/target && mkdir -p \"$CARGO_TARGET_DIR\" && touch \"$CARGO_TARGET_DIR/smoke\" && ! touch forbidden",
                ],
                &[],
                30,
            ))
            .unwrap();
        assert_eq!(result.exit_code, 0, "stderr: {}", result.stderr);
        assert!(dir.path().join("output/target/smoke").is_file());
        assert!(!dir.path().join("forbidden").exists());
    }
}
