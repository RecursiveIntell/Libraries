use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentGraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Routing error: {0}")]
    RoutingError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Max iterations exceeded: {current}/{max}")]
    MaxIterationsExceeded { current: usize, max: usize },

    #[error("Cycle detected: {path:?}")]
    CycleDetected { path: Vec<String> },

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    #[error("Checkpoint graph mismatch: expected hash '{expected}', got '{actual}'")]
    CheckpointMismatch { expected: String, actual: String },

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Interrupt at node '{node}'")]
    InterruptError {
        node: String,
        value: Option<serde_json::Value>,
    },

    #[error("Payload error: {0}")]
    PayloadError(String),

    #[error("Cancelled")]
    Cancelled,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[cfg(feature = "checkpointing")]
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("{0}")]
    Other(String),
}

impl AgentGraphError {
    /// Stable string discriminant for structured logging (PRIMITIVES_CONTRACT §2).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NodeNotFound(_) => "node_not_found",
            Self::RoutingError(_) => "routing",
            Self::StateError(_) => "state",
            Self::MaxIterationsExceeded { .. } => "max_iterations",
            Self::CycleDetected { .. } => "cycle_detected",
            Self::CheckpointError(_) => "checkpoint",
            Self::CheckpointMismatch { .. } => "checkpoint_mismatch",
            Self::ExecutionError(_) => "execution",
            Self::InterruptError { .. } => "interrupt",
            Self::PayloadError(_) => "payload",
            Self::Cancelled => "cancelled",
            Self::SerializationError(_) => "serialization",
            #[cfg(feature = "checkpointing")]
            Self::DatabaseError(_) => "database",
            Self::Other(_) => "other",
        }
    }
}

pub type Result<T> = std::result::Result<T, AgentGraphError>;

/// Create an interrupt error that can be returned from within a node.
/// This causes the graph executor to pause execution at the current node.
pub fn interrupt(node: impl Into<String>, value: Option<serde_json::Value>) -> AgentGraphError {
    AgentGraphError::InterruptError {
        node: node.into(),
        value,
    }
}
