//! Durable `ToolObservationMemory` implementation backed by `semantic-memory`.
//!
//! Search results are candidates only. Each candidate is re-read by its canonical
//! fact ID and returned only when its owner-stored source and versioned metadata
//! identify it as a tool observation. Search is approximate: the candidate limit
//! applies before filtering, so fewer than `limit` observations may be returned.
//! Similarity search can also collapse identical summaries; it is not a receipt
//! enumeration API. The underlying projection preserves distinct receipt IDs.

use crate::semantic_memory::{
    ToolObservationDerivedCandidateTraceV1, ToolObservationMemory, ToolObservationMemoryRecordV1,
    ToolObservationMemorySearchV1,
};
use crate::{ToolError, ToolErrorClass};
use async_trait::async_trait;
use semantic_memory::{MemoryStore, SearchSource, SearchSourceType};
use serde::Deserialize;

/// Fixed source label stamped onto stored tool observations.
const OBSERVATION_SOURCE: &str = "llm-tool-runtime";
const OBSERVATION_KIND: &str = "tool_observation_v1";

/// Metadata persisted alongside the fact content by this adapter.
#[derive(Debug, Deserialize)]
struct StoredToolObservationMetadataV1 {
    kind: String,
    tool_name: String,
    invocation_id: String,
    session_id: Option<String>,
    status: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    receipt_id: Option<String>,
}

#[async_trait]
impl ToolObservationMemory for MemoryStore {
    async fn store_tool_observation(
        &self,
        record: ToolObservationMemoryRecordV1,
    ) -> Result<(), ToolError> {
        let metadata = serde_json::json!({
            "kind": OBSERVATION_KIND,
            "tool_name": record.tool_name,
            "invocation_id": record.invocation_id,
            "session_id": record.session_id,
            "status": record.status,
            "started_at": record.started_at,
            "completed_at": record.completed_at,
            "receipt_id": record.receipt_id,
        });
        let result = if let Some(key) = record.receipt_id.as_deref() {
            self.insert_derived_fact_once(
                &record.scope,
                OBSERVATION_SOURCE,
                key,
                &record.summary,
                metadata,
            )
            .await
        } else {
            self.add_fact(
                &record.scope,
                &record.summary,
                Some(OBSERVATION_SOURCE),
                Some(metadata),
            )
            .await
            .map(|_| ())
        };
        result.map_err(|e| {
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
        let source_types = [SearchSourceType::Facts];
        let results = self
            .search(query, Some(limit), Some(&namespaces), Some(&source_types))
            .await
            .map_err(|e| {
                ToolError::new(
                    ToolErrorClass::ProviderContract,
                    format!("semantic-memory search failed: {e}"),
                )
            })?;

        let mut observations = Vec::with_capacity(limit.min(results.len()));
        for result in results {
            let SearchSource::Fact { fact_id, namespace } = result.source else {
                continue;
            };
            if namespace != scope {
                continue;
            }

            // The search index is a derived candidate source; fetch the canonical
            // fact so metadata and source are not fabricated from search text.
            let fact = self.get_fact(&fact_id).await.map_err(|e| {
                ToolError::new(
                    ToolErrorClass::ProviderContract,
                    format!("semantic-memory fact read failed: {e}"),
                )
            })?;
            let Some(fact) = fact else {
                // A concurrent delete can invalidate an index candidate.
                continue;
            };
            if fact.namespace != scope || fact.source.as_deref() != Some(OBSERVATION_SOURCE) {
                continue;
            }

            let Some(metadata) = fact.metadata else {
                return Err(ToolError::new(
                    ToolErrorClass::ProviderContract,
                    format!("tool observation fact {fact_id} is missing metadata"),
                ));
            };
            if metadata.get("kind").and_then(serde_json::Value::as_str) != Some(OBSERVATION_KIND) {
                continue;
            }
            let stored: StoredToolObservationMetadataV1 = serde_json::from_value(metadata)
                .map_err(|e| {
                    ToolError::new(
                        ToolErrorClass::ProviderContract,
                        format!("tool observation fact {fact_id} has invalid metadata: {e}"),
                    )
                })?;
            if stored.kind != OBSERVATION_KIND {
                continue;
            }

            observations.push(ToolObservationMemoryRecordV1 {
                tool_name: stored.tool_name,
                invocation_id: stored.invocation_id,
                session_id: stored.session_id,
                scope: fact.namespace,
                summary: fact.content,
                status: stored.status,
                started_at: stored.started_at,
                completed_at: stored.completed_at,
                receipt_id: stored.receipt_id,
            });
            if observations.len() == limit {
                break;
            }
        }

        Ok(ToolObservationMemorySearchV1 {
            observations,
            derived_candidate_receipts: Vec::<ToolObservationDerivedCandidateTraceV1>::new(),
        })
    }
}
