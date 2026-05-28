//! Error types for agent-guard.
use thiserror::Error;

/// Errors that can occur in agent-guard operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("BPF LSM not available: {0}")]
    BpfNotAvailable(String),

    #[error("cgroup v2 not available: {0}")]
    CgroupNotAvailable(String),

    #[error("Landlock not available: {0}")]
    LandlockNotAvailable(String),

    #[error("seccomp not available: {0}")]
    SeccompNotAvailable(String),

    #[error("eBPF not available: {0}")]
    EbpfNotAvailable(String),

    #[error("Security operation failed: {0}")]
    SecurityOperation(String),

    #[error("Guard not initialized")]
    NotInitialized,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("MCP broker error: {0}")]
    McpBroker(String),
}

/// Result type alias for agent-guard operations.
pub type Result<T> = std::result::Result<T, Error>;
