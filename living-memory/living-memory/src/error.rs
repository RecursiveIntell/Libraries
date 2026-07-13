use std::path::PathBuf;

pub use forge_policy::{Violation, ViolationKind};

/// All errors produced by forge-engine.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("refuse to open database: {reason}")]
    RefuseToOpenDb { reason: String },

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("patch validation failed: {violations:?}")]
    PatchValidation { violations: Vec<Violation> },

    #[error("anchor resolution failed: {0}")]
    AnchorResolution(String),

    #[error("patch apply failed: {0}")]
    PatchApply(String),

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

    #[error("container cleanup failed for {container}: {reason}")]
    ContainerCleanup { container: String, reason: String },

    #[error("sealed mode unsupported for runtime: {runtime}")]
    SealedModeUnsupported { runtime: String },

    #[error("remote model forbidden in sealed mode")]
    RemoteModelForbiddenInSealedMode,

    #[error("no container runtime found")]
    NoContainerRuntime,

    #[error("promotion failed: {criterion} = {value}")]
    PromotionFailed { criterion: String, value: String },

    #[error("golden mindstate mismatch: version={version_id}, input_hash={input_hash}")]
    GoldenMindStateMismatch {
        version_id: String,
        input_hash: String,
    },

    #[error("CEA: raw source detected in node")]
    CeaRawSourceDetected,

    #[error("fixture error: {0}")]
    Fixture(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("workspace path error: {0}")]
    WorkspacePath(PathBuf),

    #[error("experiment failed: {0}")]
    ExperimentFailed(String),

    #[error("limit exceeded: {limit} (value={value}, max={max})")]
    LimitExceeded { limit: String, value: u64, max: u64 },

    #[error("export error: {0}")]
    Export(String),

    #[error("write-through blocked: danger-sm-write feature not enabled")]
    WriteThroughBlocked,

    #[error("pair incomparable: {reasons:?}")]
    PairIncomparable { reasons: Vec<String> },

    #[error("sealed bundle cannot be mutated")]
    SealedBundle,

    #[error("{0}")]
    Other(String),
}

impl ForgeError {
    /// Returns a stable string discriminant for programmatic matching.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::RefuseToOpenDb { .. } => "refuse_to_open_db",
            Self::Database(_) => "database",
            Self::Io(_) => "io",
            Self::Serialization(_) => "serialization",
            Self::PatchValidation { .. } => "patch_validation",
            Self::AnchorResolution(_) => "anchor_resolution",
            Self::PatchApply(_) => "patch_apply",
            Self::CommandTimeout { .. } => "command_timeout",
            Self::CommandFailed { .. } => "command_failed",
            Self::KillProcessGroup { .. } => "kill_process_group",
            Self::ContainerCleanup { .. } => "container_cleanup",
            Self::SealedModeUnsupported { .. } => "sealed_mode_unsupported",
            Self::RemoteModelForbiddenInSealedMode => "remote_model_forbidden",
            Self::NoContainerRuntime => "no_container_runtime",
            Self::PromotionFailed { .. } => "promotion_failed",
            Self::GoldenMindStateMismatch { .. } => "golden_mindstate_mismatch",
            Self::CeaRawSourceDetected => "cea_raw_source",
            Self::Fixture(_) => "fixture",
            Self::Config(_) => "config",
            Self::NotFound(_) => "not_found",
            Self::WorkspacePath(_) => "workspace_path",
            Self::ExperimentFailed(_) => "experiment_failed",
            Self::LimitExceeded { .. } => "limit_exceeded",
            Self::Export(_) => "export",
            Self::WriteThroughBlocked => "write_through_blocked",
            Self::PairIncomparable { .. } => "pair_incomparable",
            Self::SealedBundle => "sealed_bundle",
            Self::Other(_) => "other",
        }
    }
}

pub type ForgeResult<T> = Result<T, ForgeError>;

impl From<sandbox_workspace::WorkspaceError> for ForgeError {
    fn from(error: sandbox_workspace::WorkspaceError) -> Self {
        match error {
            sandbox_workspace::WorkspaceError::Io(error) => Self::Io(error),
            sandbox_workspace::WorkspaceError::Policy(error) => Self::Other(error.to_string()),
        }
    }
}

impl From<mindstate_core::MindStateError> for ForgeError {
    fn from(error: mindstate_core::MindStateError) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<stabilizer_core::StabilizerError> for ForgeError {
    fn from(error: stabilizer_core::StabilizerError) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<typed_patch::PatchError> for ForgeError {
    fn from(error: typed_patch::PatchError) -> Self {
        match error {
            typed_patch::PatchError::Validation { violations } => {
                Self::PatchValidation { violations }
            }
            typed_patch::PatchError::AnchorResolution(message) => Self::AnchorResolution(message),
            typed_patch::PatchError::Apply(message) => Self::PatchApply(message),
            typed_patch::PatchError::Io(error) => Self::Io(error),
            typed_patch::PatchError::Workspace(error) => Self::from(error),
            typed_patch::PatchError::Policy(error) => Self::Other(error.to_string()),
            typed_patch::PatchError::Other(message) => Self::Other(message),
        }
    }
}

impl From<check_runner::RunnerError> for ForgeError {
    fn from(error: check_runner::RunnerError) -> Self {
        match error {
            check_runner::RunnerError::Io(error) => Self::Io(error),
            check_runner::RunnerError::Policy(error) => Self::Other(error.to_string()),
            check_runner::RunnerError::Workspace(error) => Self::from(error),
            check_runner::RunnerError::CommandTimeout {
                command,
                timeout_secs,
            } => Self::CommandTimeout {
                command,
                timeout_secs,
            },
            check_runner::RunnerError::CommandFailed { command, exit_code } => {
                Self::CommandFailed { command, exit_code }
            }
            check_runner::RunnerError::KillProcessGroup { pgid, source } => {
                Self::KillProcessGroup { pgid, source }
            }
            check_runner::RunnerError::ContainerCleanup { container, reason } => {
                Self::ContainerCleanup { container, reason }
            }
            check_runner::RunnerError::SealedModeUnsupported { runtime } => {
                Self::SealedModeUnsupported { runtime }
            }
            check_runner::RunnerError::NoContainerRuntime => Self::NoContainerRuntime,
            check_runner::RunnerError::Other(message) => Self::Other(message),
        }
    }
}

impl From<cea_core::CeaCoreError> for ForgeError {
    fn from(error: cea_core::CeaCoreError) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<cea_store::CeaStoreError> for ForgeError {
    fn from(error: cea_store::CeaStoreError) -> Self {
        match error {
            cea_store::CeaStoreError::Serialization(error) => Self::Serialization(error),
            cea_store::CeaStoreError::Backend(message) => Self::Other(message),
        }
    }
}

impl From<semantic_memory_forge::ExportEnvelopeError> for ForgeError {
    fn from(error: semantic_memory_forge::ExportEnvelopeError) -> Self {
        Self::Export(error.to_string())
    }
}

impl From<forge_memory_bridge::BridgeError> for ForgeError {
    fn from(error: forge_memory_bridge::BridgeError) -> Self {
        Self::Export(error.to_string())
    }
}

impl From<semantic_memory::MemoryError> for ForgeError {
    fn from(error: semantic_memory::MemoryError) -> Self {
        Self::Other(format!("semantic-memory write: {error}"))
    }
}
