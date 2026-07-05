//! Custom tool executor trait for registering external executors.
//!
//! This allows kirsten (or any AiDENs consumer) to register tool executors
//! that call semantic-memory or any other backend, without modifying the
//! AiDENs tool-kit internals.

use serde_json::Value;
use std::sync::Arc;

/// Trait for custom tool executors. Implement this for each kirsten tool.
#[async_trait::async_trait]
pub trait CustomToolExecutor: Send + Sync + std::fmt::Debug {
    /// Execute the tool with the given input. Return the output text.
    async fn execute(&self, tool_id: &str, input: Value) -> anyhow::Result<String>;

    /// Clone into an Arc (for storing in ToolExecutorV1::Custom).
    fn clone_box(&self) -> Arc<dyn CustomToolExecutor>;
}

/// A wrapper that makes CustomToolExecutor cloneable for the enum.
#[derive(Clone)]
pub struct CustomExecutorHandle {
    pub inner: Arc<dyn CustomToolExecutor>,
}

impl std::fmt::Debug for CustomExecutorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomExecutorHandle").finish()
    }
}

impl CustomExecutorHandle {
    pub fn new(exec: Arc<dyn CustomToolExecutor>) -> Self {
        Self { inner: exec }
    }
}
