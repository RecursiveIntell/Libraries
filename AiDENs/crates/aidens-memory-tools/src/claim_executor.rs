//! ClaimToolExecutor — native claim-ledger tool executor for AiDENs.

use aidens_tool_kit::CustomToolExecutor;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::sync::Arc;

/// Executes claim-ledger tools via direct Rust calls.
#[derive(Debug, Clone)]
pub struct ClaimToolExecutor;

impl ClaimToolExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClaimToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CustomToolExecutor for ClaimToolExecutor {
    async fn execute(&self, tool_id: &str, input: Value) -> Result<String> {
        let _ = (tool_id, input);
        Err(anyhow!(
            "claim tools not yet implemented — subagent in progress"
        ))
    }

    fn clone_box(&self) -> Arc<dyn CustomToolExecutor> {
        Arc::new(self.clone())
    }
}
