use crate::checkpoint::Checkpoint;
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for saving and loading checkpoints.
#[async_trait]
pub trait CheckpointSaver: Send + Sync {
    /// Save a checkpoint
    async fn save(&self, checkpoint: &Checkpoint) -> Result<()>;
    /// Load the most recent checkpoint for a thread
    async fn load(&self, thread_id: &str) -> Result<Option<Checkpoint>>;
    /// Load all checkpoints for a thread (history)
    async fn load_history(&self, thread_id: &str) -> Result<Vec<Checkpoint>>;
    /// Clear all checkpoints for a thread
    async fn clear(&self, thread_id: &str) -> Result<()>;
}

/// In-memory checkpoint storage for tests and lightweight compatibility use.
///
/// This saver is not a durable persistence implementation. Durable checkpoint
/// state must use [`crate::checkpoint_store::CheckpointStore`].
pub struct MemorySaver {
    checkpoints: Arc<RwLock<HashMap<String, Vec<Checkpoint>>>>,
}

impl MemorySaver {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemorySaver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CheckpointSaver for MemorySaver {
    async fn save(&self, checkpoint: &Checkpoint) -> Result<()> {
        let mut store = self.checkpoints.write().await;
        store
            .entry(checkpoint.execution_id.clone())
            .or_default()
            .push(checkpoint.clone());
        Ok(())
    }

    async fn load(&self, thread_id: &str) -> Result<Option<Checkpoint>> {
        let store = self.checkpoints.read().await;
        Ok(store.get(thread_id).and_then(|v| v.last()).cloned())
    }

    async fn load_history(&self, thread_id: &str) -> Result<Vec<Checkpoint>> {
        let store = self.checkpoints.read().await;
        Ok(store.get(thread_id).cloned().unwrap_or_default())
    }

    async fn clear(&self, thread_id: &str) -> Result<()> {
        let mut store = self.checkpoints.write().await;
        store.remove(thread_id);
        Ok(())
    }
}
