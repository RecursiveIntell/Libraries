use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Real-time progress update from ComfyUI's WebSocket.
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub current_step: u32,
    pub total_steps: u32,
}

/// Richer progress type with node and prompt context.
#[derive(Debug, Clone)]
pub struct ComfyProgress {
    /// Current sampling step.
    pub current_step: u32,
    /// Total sampling steps.
    pub total_steps: u32,
    /// The node currently executing, if known.
    pub node_id: Option<String>,
    /// The prompt this progress belongs to.
    pub prompt_id: String,
}

/// High-level status of a ComfyUI generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComfyStatus {
    /// Prompt is queued but not yet executing.
    Queued,
    /// A specific node is executing.
    Running { node_id: String },
    /// Sampling progress update.
    Progress { step: u32, total: u32 },
    /// Generation completed successfully.
    Completed,
    /// Generation failed with an error.
    Failed { error: String },
}

/// WebSocket connection configuration for recovery and limits.
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Number of reconnection attempts before falling back to polling (default: 3).
    pub reconnect_attempts: u32,
    /// Delay between reconnection attempts (default: 1s).
    pub reconnect_delay: Duration,
    /// Timeout for receiving a WebSocket message (default: 30s).
    pub message_timeout: Duration,
    /// Maximum messages for the tracked prompt before fallback (default: 10,000).
    pub max_messages_per_prompt: usize,
    /// Maximum total messages before fallback (default: 50,000).
    pub max_total_messages: usize,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            reconnect_attempts: 3,
            reconnect_delay: Duration::from_secs(1),
            message_timeout: Duration::from_secs(30),
            max_messages_per_prompt: 10_000,
            max_total_messages: 50_000,
        }
    }
}

/// Limits for image download operations.
#[derive(Debug, Clone)]
pub struct DownloadLimits {
    /// Maximum image size in bytes (default: 100 MB).
    pub max_image_bytes: usize,
    /// Per-image download timeout (default: 60s).
    pub download_timeout: Duration,
}

impl Default for DownloadLimits {
    fn default() -> Self {
        Self {
            max_image_bytes: 100 * 1024 * 1024,
            download_timeout: Duration::from_secs(60),
        }
    }
}

/// Reference to an image stored in ComfyUI's output directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub filename: String,
    pub subfolder: String,
    pub img_type: String,
}

/// Parsed history entry for a completed prompt.
#[derive(Debug, Clone)]
pub struct PromptHistory {
    pub status: String,
    pub completed: bool,
    pub images: Vec<ImageRef>,
}

/// Snapshot of ComfyUI's queue state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub running: u32,
    pub pending: u32,
}

/// Outcome of waiting for a generation to finish.
#[derive(Debug, Clone)]
pub enum GenerationOutcome {
    /// Generation completed successfully with output images.
    Completed { images: Vec<ImageRef> },
    /// ComfyUI reported an execution-level failure.
    Failed { error: String },
    /// Timed out before completion.
    TimedOut,
}
