//! MemoryToolExecutor — native semantic-memory tool executor for AiDENs.

use aidens_memory_kit::CanonicalMemoryAdapter;
use aidens_tool_kit::CustomToolExecutor;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::sync::Arc;

/// Executes memory and knowledge-runtime tools via direct Rust calls.
///
/// Holds an Arc<CanonicalMemoryAdapter> which wraps MemoryStore + KnowledgeRuntime
/// in-process. No MCP, no HTTP — direct function calls.
#[derive(Clone)]
pub struct MemoryToolExecutor {
    adapter: Arc<CanonicalMemoryAdapter>,
}

impl std::fmt::Debug for MemoryToolExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryToolExecutor").finish_non_exhaustive()
    }
}

impl MemoryToolExecutor {
    pub fn new(adapter: Arc<CanonicalMemoryAdapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait::async_trait]
impl CustomToolExecutor for MemoryToolExecutor {
    async fn execute(&self, tool_id: &str, input: Value) -> Result<String> {
        let _ = (tool_id, input);
        Err(anyhow!("memory tools not yet implemented — subagent in progress"))
    }

    fn clone_box(&self) -> Arc<dyn CustomToolExecutor> {
        Arc::new(self.clone())
    }
}