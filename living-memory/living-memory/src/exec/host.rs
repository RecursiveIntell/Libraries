use std::path::Path;

use async_trait::async_trait;
use check_runner::ExecutionBackend as RunnerExecutionBackend;

use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::exec::backend::*;

/// Check if an environment variable key is allowed to pass through.
pub fn is_env_allowed(key: &str) -> bool {
    check_runner::is_env_allowed(key)
}

/// HostBackend — runs commands via tokio::process::Command with kill_on_drop.
pub struct HostBackend {
    inner: check_runner::HostBackend,
}

impl HostBackend {
    pub fn new(config: &ForgeConfig) -> Self {
        Self {
            inner: check_runner::HostBackend::new(&crate::exec::backend::runner_config(config)),
        }
    }
}

#[async_trait]
impl ExecutionBackend for HostBackend {
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

/// Recursively copy a directory (public for container backend).
pub fn copy_dir_recursive_pub(src: &Path, dst: &Path) -> ForgeResult<()> {
    Ok(sandbox_workspace::copy_dir_recursive_pub(src, dst)?)
}
