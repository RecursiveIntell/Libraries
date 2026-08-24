//! Semantic-memory-facing contracts for searchable tool observations.

use crate::{ToolError, ToolErrorClass};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Serializable record written to semantic-memory for a completed or failed tool run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolObservationMemoryRecordV1 {
    pub tool_name: String,
    pub invocation_id: String,
    pub session_id: Option<String>,
    pub scope: String,
    pub summary: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub receipt_id: Option<String>,
}

/// Semantic-memory derived-candidate receipt summary surfaced by searches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolObservationDerivedCandidateTraceV1 {
    pub candidate_backend: String,
    pub codec_family: Option<String>,
    pub generation_id: Option<String>,
    pub exact_rerank: bool,
    pub approximate: bool,
    pub fallback: Option<String>,
}

/// Result of a similar-observation query, including candidate provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolObservationMemorySearchV1 {
    pub observations: Vec<ToolObservationMemoryRecordV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_candidate_receipts: Vec<ToolObservationDerivedCandidateTraceV1>,
}

/// Minimal trait for semantic-memory-backed tool observation storage/retrieval.
#[async_trait]
pub trait ToolObservationMemory: Send + Sync {
    async fn store_tool_observation(
        &self,
        record: ToolObservationMemoryRecordV1,
    ) -> Result<(), ToolError>;

    async fn find_similar_tool_observations(
        &self,
        query: &str,
        scope: &str,
        limit: usize,
    ) -> Result<ToolObservationMemorySearchV1, ToolError>;
}

/// Small test/demonstration implementation that mimics scoped retrieval.
#[derive(Debug, Default)]
pub struct InMemoryToolObservationMemory {
    records: Mutex<Vec<ToolObservationMemoryRecordV1>>,
}

impl InMemoryToolObservationMemory {
    pub fn records(&self) -> Vec<ToolObservationMemoryRecordV1> {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

#[async_trait]
impl ToolObservationMemory for InMemoryToolObservationMemory {
    async fn store_tool_observation(
        &self,
        record: ToolObservationMemoryRecordV1,
    ) -> Result<(), ToolError> {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(record);
        Ok(())
    }

    async fn find_similar_tool_observations(
        &self,
        query: &str,
        scope: &str,
        limit: usize,
    ) -> Result<ToolObservationMemorySearchV1, ToolError> {
        if limit == 0 {
            return Err(ToolError::new(
                ToolErrorClass::ProviderContract,
                "limit must be greater than zero for tool observation search",
            ));
        }
        let query_lc = query.to_ascii_lowercase();
        let observations = self
            .records()
            .into_iter()
            .filter(|record| record.scope == scope)
            .filter(|record| {
                query_lc.is_empty() || record.summary.to_ascii_lowercase().contains(&query_lc)
            })
            .take(limit)
            .collect();
        Ok(ToolObservationMemorySearchV1 {
            observations,
            derived_candidate_receipts: Vec::new(),
        })
    }
}
