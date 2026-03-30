use std::path::Path;

use async_trait::async_trait;

use crate::config::ForgeConfig;
use crate::error::{ForgeError, ForgeResult};

pub use check_runner::{
    CheckCommand, CheckKind, CheckResult, CommandOutput, CommandTimings, ExecutionBackendKind,
    LogBundle, ParsedCheckOutput,
};
pub use effect_signature::{EffectSignature, LocatedEffect};
pub use sandbox_workspace::{PatchedWorkspace, Workspace};

/// Execution backend trait.
#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn kind(&self) -> ExecutionBackendKind;

    async fn prepare_workspace(&self, fixture: &Path) -> ForgeResult<Workspace>;

    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout_secs: u64,
    ) -> ForgeResult<CommandOutput>;

    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> ForgeResult<LogBundle>;
}

pub(crate) fn runner_config(config: &ForgeConfig) -> check_runner::BackendConfig {
    check_runner::BackendConfig {
        mode: config.mode.clone(),
        execution_backend_preference: config.execution_backend_preference.clone(),
        container_runtime_preference: config.container_runtime_preference.clone(),
        sealed_allow_host_backend: config.sealed_allow_host_backend,
        rust_image: config.container.rust_image.clone(),
        command_timeout_secs: config.container.command_timeout_secs,
        memory_limit: config.container.memory_limit.clone(),
        cpu_limit: config.container.cpu_limit.clone(),
    }
}

/// Select the best available backend based on config.
///
/// Sealed mode enforcement:
/// - `"host"` explicit: rejected unless `sealed_allow_host_backend = true`.
/// - `"container"` explicit: never falls back to host; errors if container unavailable.
/// - `"auto"`: tries container first; only falls back to host if not sealed or
///   `sealed_allow_host_backend = true`.
pub fn select_backend(config: &ForgeConfig) -> ForgeResult<Box<dyn ExecutionBackend>> {
    let sealed = config.mode == "sealed_local";

    match config.execution_backend_preference.as_str() {
        "host" => {
            if sealed && !config.sealed_allow_host_backend {
                return Err(ForgeError::SealedModeUnsupported {
                    runtime: "host backend requested in sealed_local mode".into(),
                });
            }
            if sealed {
                tracing::warn!("sealed_local mode with host backend — no network isolation");
            }
            Ok(Box::new(crate::exec::host::HostBackend::new(config)))
        }
        "container" => crate::exec::container::ContainerBackend::new(config)
            .map(|backend| Box::new(backend) as Box<dyn ExecutionBackend>)
            .map_err(|error| {
                if sealed {
                    ForgeError::SealedModeUnsupported {
                        runtime: format!("container unavailable in sealed mode: {error}"),
                    }
                } else {
                    error
                }
            }),
        _ => match crate::exec::container::ContainerBackend::new(config) {
            Ok(backend) => Ok(Box::new(backend)),
            Err(_) => {
                if sealed && !config.sealed_allow_host_backend {
                    return Err(ForgeError::SealedModeUnsupported {
                        runtime:
                            "container unavailable and host fallback not allowed in sealed mode"
                                .into(),
                    });
                }
                if sealed {
                    tracing::warn!(
                            "sealed_local: container unavailable, falling back to host — isolation not guaranteed"
                        );
                }
                Ok(Box::new(crate::exec::host::HostBackend::new(config)))
            }
        },
    }
}
