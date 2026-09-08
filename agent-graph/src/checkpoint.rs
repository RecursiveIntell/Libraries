use crate::state::StateSnapshot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// In-memory checkpoint value used by the graph compatibility API.
///
/// Durable checkpoint truth is owned by [`crate::checkpoint_store::CheckpointStore`]
/// and its `SqliteCheckpointStore` implementation. This value is intentionally
/// storage-neutral and does not open or mutate a database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub execution_id: String,
    pub timestamp: DateTime<Utc>,
    pub current_node: String,
    pub iteration: usize,
    pub state: StateSnapshot,
    #[serde(default)]
    pub step_number: usize,
    #[serde(default)]
    pub active_nodes: Vec<String>,
}
