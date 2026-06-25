//! Result capture — stores execution outputs as facts in semantic memory.
//!
//! The [`ResultCapture`] component takes an [`ExecutionResult`] from the
//! executor, checks whether the output is a duplicate of existing knowledge,
//! and if not, writes it as a new fact in the `"autonomous"` namespace. When
//! a source fact ID is known (the gap that triggered the job), a graph edge
//! is added connecting the new fact to the source, with relation
//! `"fills_gap"`.

use crate::executor::ExecutionResult;
use aidens_memory_kit::canonical_stack::AddGraphEdgeParams;
use aidens_memory_kit::CanonicalMemoryAdapter;
use semantic_memory::types::GraphEdgeType;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Outcome of a capture operation — how many facts were added, skipped, and
/// their IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureOutcome {
    /// Number of new facts added to memory.
    pub facts_added: usize,
    /// Number of facts skipped because they duplicated existing content.
    pub facts_skipped_duplicates: usize,
    /// IDs of the newly added facts.
    pub fact_ids: Vec<String>,
}

/// Captures execution results into semantic memory.
#[derive(Clone)]
pub struct ResultCapture {
    /// Shared canonical memory adapter for search and fact insertion.
    pub memory: Arc<CanonicalMemoryAdapter>,
    /// Semantic-memory HTTP base URL (reserved for future HTTP-based capture).
    pub http_base_url: String,
}

impl std::fmt::Debug for ResultCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCapture")
            .field("http_base_url", &self.http_base_url)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl ResultCapture {
    /// Create a new result capture with the given memory adapter.
    pub fn new(
        memory: Arc<CanonicalMemoryAdapter>,
        http_base_url: impl Into<String>,
    ) -> Self {
        Self {
            memory,
            http_base_url: http_base_url.into(),
        }
    }

    /// Capture an execution result into semantic memory.
    ///
    /// 1. Searches memory for the output content to check for duplicates.
    /// 2. If no duplicate: adds a fact in the `"autonomous"` namespace.
    /// 3. If the source fact ID is known: adds a graph edge connecting the
    ///    new fact to the source fact with relation `"fills_gap"`.
    pub async fn capture(&self, result: &ExecutionResult) -> Result<CaptureOutcome> {
        // Don't capture empty or failed outputs.
        if !result.success || result.output.is_empty() {
            return Ok(CaptureOutcome {
                facts_added: 0,
                facts_skipped_duplicates: 0,
                fact_ids: Vec::new(),
            });
        }

        // 1. Search for duplicates.
        let existing = self
            .memory
            .search(&result.output, Some(&["autonomous".to_string()]), Some(5))
            .await?;

        // Check if any existing result has very similar content (exact match
        // or near-exact containment).
        let is_duplicate = existing.iter().any(|r| {
            r.content == result.output
                || r.content.contains(&result.output)
                || result.output.contains(&r.content)
        });

        if is_duplicate {
            return Ok(CaptureOutcome {
                facts_added: 0,
                facts_skipped_duplicates: 1,
                fact_ids: Vec::new(),
            });
        }

        // 2. Add the new fact.
        let source = format!("aidens-autonomous:{}", result.job_id);
        let fact_id = self
            .memory
            .add_fact(
                "autonomous",
                &result.output,
                Some(&source),
                Some(0.6),
            )
            .await?;

        let fact_ids = vec![fact_id.clone()];

        // 3. Add graph edge connecting new fact to source fact.
        if !result.source_fact_id.is_empty() {
            let new_fact_node = if fact_id.starts_with("fact:") {
                fact_id.clone()
            } else {
                format!("fact:{fact_id}")
            };

            let source_node = if result.source_fact_id.starts_with("fact:") {
                result.source_fact_id.clone()
            } else {
                format!("fact:{}", result.source_fact_id)
            };

            let edge_params = AddGraphEdgeParams {
                source: source_node,
                target: new_fact_node,
                edge_type: GraphEdgeType::Entity {
                    relation: "fills_gap".to_string(),
                },
                weight: 1.0,
                metadata: Some(serde_json::json!({
                    "job_id": result.job_id,
                    "gap_type": result.gap_type,
                })),
            };

            // Best-effort: don't fail capture if edge creation fails.
            if let Err(_e) = self.memory.add_graph_edge(edge_params).await {
                // Edge creation is best-effort; the fact is already stored.
            }
        }

        Ok(CaptureOutcome {
            facts_added: 1,
            facts_skipped_duplicates: 0,
            fact_ids,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::ExecutionResult;
    use aidens_memory_kit::{memory_config_for_root, runtime_config_for_namespace};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("aidens-autonomous-capture-{name}-{id}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn mock_memory() -> Arc<CanonicalMemoryAdapter> {
        let dir = temp_dir("memory");
        let config = memory_config_for_root(&dir);
        let runtime = runtime_config_for_namespace("autonomous-test");
        Arc::new(
            CanonicalMemoryAdapter::open_with_mock_embedder(config, runtime)
                .expect("open mock memory"),
        )
    }

    fn make_result(success: bool, output: &str, fact_id: &str) -> ExecutionResult {
        ExecutionResult {
            job_id: "job:test-001".to_string(),
            output: output.to_string(),
            success,
            error: if success { None } else { Some("failed".to_string()) },
            gap_type: "missing-context".to_string(),
            source_fact_id: fact_id.to_string(),
        }
    }

    #[tokio::test]
    async fn capture_adds_new_fact() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(
            true,
            "Rust is a systems programming language with memory safety guarantees.",
            "fact:source-abc",
        );

        let outcome = capture.capture(&result).await.unwrap();
        assert_eq!(outcome.facts_added, 1);
        assert_eq!(outcome.facts_skipped_duplicates, 0);
        assert_eq!(outcome.fact_ids.len(), 1);
    }

    #[tokio::test]
    async fn capture_skips_duplicate() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(
            true,
            "Rust is a systems programming language with memory safety guarantees.",
            "fact:source-abc",
        );

        // First capture adds the fact.
        let outcome1 = capture.capture(&result).await.unwrap();
        assert_eq!(outcome1.facts_added, 1);

        // Second capture should detect the duplicate.
        let outcome2 = capture.capture(&result).await.unwrap();
        assert_eq!(outcome2.facts_added, 0);
        assert_eq!(outcome2.facts_skipped_duplicates, 1);
    }

    #[tokio::test]
    async fn capture_skips_failed_execution() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(false, "partial output", "fact:source-abc");

        let outcome = capture.capture(&result).await.unwrap();
        assert_eq!(outcome.facts_added, 0);
        assert_eq!(outcome.facts_skipped_duplicates, 0);
    }

    #[tokio::test]
    async fn capture_skips_empty_output() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(true, "", "fact:source-abc");

        let outcome = capture.capture(&result).await.unwrap();
        assert_eq!(outcome.facts_added, 0);
    }

    #[tokio::test]
    async fn capture_adds_graph_edge_to_source_fact() {
        let memory = mock_memory();

        // Pre-add a source fact so the graph edge has a real target.
        let source_fact_id = memory
            .add_fact("autonomous", "Source fact content here.", Some("test"), Some(0.5))
            .await
            .unwrap();

        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(
            true,
            "New knowledge that fills the gap in the source fact about Rust.",
            &source_fact_id,
        );

        let outcome = capture.capture(&result).await.unwrap();
        assert_eq!(outcome.facts_added, 1);
        assert_eq!(outcome.fact_ids.len(), 1);

        // Verify graph edge was created — list edges for the new fact.
        let new_fact_id = &outcome.fact_ids[0];
        let new_node = if new_fact_id.starts_with("fact:") {
            new_fact_id.clone()
        } else {
            format!("fact:{new_fact_id}")
        };
        let edges = memory.list_graph_edges(&new_node).await.unwrap();
        assert!(!edges.is_empty(), "graph edge should have been created");
    }
}