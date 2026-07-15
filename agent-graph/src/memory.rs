//! Type-only graph-state retrieval contracts for external memory adapters.
//!
//! The core scheduler does not invoke this trait or record memory envelopes.

use crate::{receipt::GraphMemoryGenerationRefV1, AgentGraphError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// One retrieved graph context item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMemoryContextItemV1 {
    pub graph_id: String,
    pub state_ref: String,
    pub summary: String,
}

/// Retrieval result for graph state/history, including semantic-memory provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMemoryRetrievalV1 {
    pub graph_id: String,
    pub query: String,
    pub items: Vec<GraphMemoryContextItemV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_generations: Vec<GraphMemoryGenerationRefV1>,
}

/// Adapter interface not wired into core graph execution.
#[async_trait]
pub trait GraphMemoryRetriever: Send + Sync {
    async fn retrieve_graph_context(
        &self,
        graph_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<GraphMemoryRetrievalV1, AgentGraphError>;
}
