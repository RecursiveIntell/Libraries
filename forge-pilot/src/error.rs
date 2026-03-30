use forge_engine::ForgeError;
use forge_memory_bridge::BridgeError;
use knowledge_runtime::RuntimeError;
use semantic_memory::MemoryError;
use thiserror::Error;
use verification_policy::PermitIssuanceError;

#[derive(Debug, Error)]
pub enum PilotError {
    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("memory error: {0}")]
    Memory(#[from] MemoryError),
    #[error("forge error: {0}")]
    Forge(#[from] ForgeError),
    #[error("bridge error: {0}")]
    Bridge(#[from] BridgeError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing kernel payload for scope {scope}")]
    MissingKernelPayload { scope: String },
    #[error("invalid kernel payload for scope {scope}: {reason}")]
    InvalidKernelPayload { scope: String, reason: String },
    #[error("missing compiled kernel context")]
    MissingCompiledContext,
    #[error("missing temporal replay snapshots")]
    MissingTemporalSnapshots,
    #[error("no executable plan available for target {target_key}")]
    NoExecutablePlan { target_key: String },
    #[error("unsupported patch fixture {fixture_path}")]
    UnsupportedPatchFixture { fixture_path: String },
    #[error("control-plane ledger replay failed: {0}")]
    ControlPlaneReplay(String),
    #[error("execution permit error: {0}")]
    ExecutionPermit(#[from] PermitIssuanceError),
    #[error("{0}")]
    Other(String),
}

impl PilotError {
    /// Returns a stable string discriminant for this error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Runtime(..) => "runtime",
            Self::Memory(..) => "memory",
            Self::Forge(..) => "forge",
            Self::Bridge(..) => "bridge",
            Self::Json(..) => "json",
            Self::MissingKernelPayload { .. } => "missing_kernel_payload",
            Self::InvalidKernelPayload { .. } => "invalid_kernel_payload",
            Self::MissingCompiledContext => "missing_compiled_context",
            Self::MissingTemporalSnapshots => "missing_temporal_snapshots",
            Self::NoExecutablePlan { .. } => "no_executable_plan",
            Self::UnsupportedPatchFixture { .. } => "unsupported_patch_fixture",
            Self::ControlPlaneReplay(..) => "control_plane_replay",
            Self::ExecutionPermit(..) => "execution_permit",
            Self::Other(..) => "other",
        }
    }
}
