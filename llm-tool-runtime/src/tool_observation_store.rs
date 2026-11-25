//! Durable `ToolObservationMemory` implementation backed by `semantic-memory`.
//!
//! `llm-tool-runtime` previously defined the trait but shipped only
//! `InMemoryToolObservationMemory` (and the trait module was not even wired
//! into the crate root). This module provides the durable store-backed impl.

use crate::semantic_memory::{
    ToolObservationDerivedCandidateTraceV1, ToolObservationMemory, ToolObservationMemoryRecordV1,
    ToolObservationMemorySearchV1,
};
use crate::{ToolError, ToolErrorClass};
use async_trait::async_trait;
use semantic_memory::MemoryStore;

/// Fixed source label stamped onto stored tool observations.
const OBSERVATION_SOURCE: &str = "llm-tool-runtime";

#[async_trait]
impl ToolObservationMemory for MemoryStore {
    async fn store_tool_observation(
        &self,
        record: ToolObservationMemoryRecordV1,
    ) -> Result<(), ToolError> {
        let metadata = serde_json::json!({
            "kind": "tool_observation_v1",
            "tool_name": record.tool_name,
            "invocation_id": record.invocation_id,
            "session_id": record.session_id,
            "status": record.status,
            "started_at": record.started_at,
            "completed_at": record.completed_at,
            "receipt_id": record.receipt_id,
        });
        self.add_fact(
            &record.scope,
            &record.summary,
            Some(OBSERVATION_SOURCE),
            Some(metadata),
        )
        .await
        .map(|_| ())
        .map_err(|e| {
            ToolError::new(
                ToolErrorClass::ProviderContract,
                format!("semantic-memory store failed: {e}"),
            )
        })
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
        let namespaces = [scope];
        let results = self
            .search(query, Some(limit), Some(&namespaces), None)
            .await
            .map_err(|e| {
                ToolError::new(
                    ToolErrorClass::ProviderContract,
                    format!("semantic-memory search failed: {e}"),
                )
            })?;
        let observations = results
            .into_iter()
            .map(|r| ToolObservationMemoryRecordV1 {
                tool_name: String::new(),
                invocation_id: String::new(),
                session_id: None,
                scope: scope.to_string(),
                summary: r.content,
                status: String::new(),
                started_at: None,
                completed_at: None,
                receipt_id: None,
            })
            .collect();
        Ok(ToolObservationMemorySearchV1 {
            observations,
            derived_candidate_receipts: Vec::<ToolObservationDerivedCandidateTraceV1>::new(),
        })
    }
}
