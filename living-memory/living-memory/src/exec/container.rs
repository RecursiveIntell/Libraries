use std::path::Path;

use async_trait::async_trait;
use check_runner::ExecutionBackend as RunnerExecutionBackend;

use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::exec::backend::*;

pub use check_runner::ContainerRuntime;

/// ContainerBackend — runs commands inside containers.
pub struct ContainerBackend {
    inner: check_runner::ContainerBackend,
}

impl ContainerBackend {
    /// Create a new ContainerBackend, auto-detecting the container runtime.
    pub fn new(config: &ForgeConfig) -> ForgeResult<Self> {
        Ok(Self {
            inner: check_runner::ContainerBackend::new(&crate::exec::backend::runner_config(
                config,
            ))?,
        })
    }

    /// Get the detected runtime.
    pub fn runtime(&self) -> ContainerRuntime {
        self.inner.runtime()
    }
}

#[async_trait]
impl ExecutionBackend for ContainerBackend {
    fn kind(&self) -> ExecutionBackendKind {
        self.inner.kind()
    }

    async fn prepare_workspace(&self, fixture: &Path) -> ForgeResult<Workspace> {
        Ok(self.inner.prepare_workspace(fixture).await?)
    }

    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout_secs: u64,
    ) -> ForgeResult<CommandOutput> {
        Ok(self
            .inner
            .run_command(workspace, program, args, env, timeout_secs)
            .await?)
    }

    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> ForgeResult<LogBundle> {
        Ok(self.inner.collect_logs(fmt, clippy, test).await?)
    }
}

/// Detect the available container runtime.
pub fn detect_runtime(preference: &str) -> ForgeResult<ContainerRuntime> {
    Ok(check_runner::detect_runtime(preference)?)
}
