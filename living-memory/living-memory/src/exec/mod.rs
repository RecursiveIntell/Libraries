pub mod backend;
pub mod container;
pub mod host;

pub use backend::{
    select_backend, CheckCommand, CheckKind, CheckResult, CommandOutput, CommandTimings,
    EffectSignature, ExecutionBackend, ExecutionBackendKind, LocatedEffect, LogBundle,
    ParsedCheckOutput, PatchedWorkspace, Workspace,
};
pub use container::{ContainerBackend, ContainerRuntime};
pub use host::{is_env_allowed, HostBackend};
