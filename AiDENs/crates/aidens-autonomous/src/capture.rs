//! Result capture — stores execution outputs as facts in semantic memory.
//!
//! The [`ResultCapture`] component takes an [`ExecutionResult`] from the
//! executor, extracts individual factual statements from the model output,
//! checks each for duplicates against existing knowledge, and writes unique
//! statements as separate facts in the `"autonomous"` namespace. When a source
//! fact ID is known (the gap that triggered the job), a graph edge is added
//! connecting each new fact to the source, with relation `"fills_gap"`.
//!
//! Confidence is set based on content quality: 0.8 for sentences containing
//! specific factual signals (numbers, dates, proper nouns), 0.5 otherwise.

use crate::executor::ExecutionResult;
use aidens_memory_kit::canonical_stack::AddGraphEdgeParams;
use aidens_memory_kit::CanonicalMemoryAdapter;
use anyhow::Result;
use semantic_memory::types::GraphEdgeType;
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
    pub fn new(memory: Arc<CanonicalMemoryAdapter>, http_base_url: impl Into<String>) -> Self {
        Self {
            memory,
            http_base_url: http_base_url.into(),
        }
    }

    /// Capture an execution result into semantic memory.
    ///
    /// Instead of capturing the entire model output as one fact, this extracts
    /// individual factual statements:
    /// 1. Splits the output by sentences (". " or newlines).
    /// 2. For each sentence >30 chars, checks for duplicates against the KB.
    /// 3. Adds each unique sentence as a separate fact.
    /// 4. Sets confidence based on content quality (0.8 if specific facts,
    ///    0.5 otherwise).
    /// 5. Links each new fact to the source fact with a `fills_gap` edge.
    pub async fn capture(&self, result: &ExecutionResult) -> Result<CaptureOutcome> {
        // Don't capture empty or failed outputs.
        if !result.success || result.output.is_empty() {
            return Ok(CaptureOutcome {
                facts_added: 0,
                facts_skipped_duplicates: 0,
                fact_ids: Vec::new(),
            });
        }

        // Extract individual factual statements.
        let sentences = extract_sentences(&result.output);

        let mut facts_added = 0usize;
        let mut facts_skipped_duplicates = 0usize;
        let mut fact_ids: Vec<String> = Vec::new();
        let source = format!("aidens-autonomous:{}", result.job_id);

        for sentence in &sentences {
            // Skip short sentences.
            if sentence.len() < 30 {
                continue;
            }

            // Search for duplicates in the autonomous namespace.
            let existing = self
                .memory
                .search(sentence, Some(&["autonomous".to_string()]), Some(5))
                .await?;

            let is_duplicate = existing.iter().any(|r| {
                r.content == *sentence
                    || r.content.contains(sentence.as_str())
                    || sentence.contains(&r.content)
            });

            if is_duplicate {
                facts_skipped_duplicates += 1;
                continue;
            }

            // Determine confidence based on content quality.
            let confidence = if has_specific_facts(sentence) {
                0.8
            } else {
                0.5
            };

            // Add the new fact.
            let fact_id = self
                .memory
                .add_fact("autonomous", sentence, Some(&source), Some(confidence))
                .await?;

            // Add graph edge connecting new fact to source fact.
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

                let edge_params = capture_edge_params(
                    source_node,
                    new_fact_node,
                    &result.job_id,
                    &result.gap_type,
                    &result.source_valid_time,
                );

                // Best-effort: don't fail capture if edge creation fails.
                let _ = self.memory.add_graph_edge(edge_params).await;
            }

            facts_added += 1;
            fact_ids.push(fact_id);
        }

        Ok(CaptureOutcome {
            facts_added,
            facts_skipped_duplicates,
            fact_ids,
        })
    }
}

fn capture_edge_params(
    source: String,
    target: String,
    job_id: &str,
    gap_type: &str,
    valid_time: &str,
) -> AddGraphEdgeParams {
    AddGraphEdgeParams {
        source,
        target,
        edge_type: GraphEdgeType::Entity {
            relation: "fills_gap".to_string(),
        },
        weight: 1.0,
        metadata: Some(serde_json::json!({ "job_id": job_id, "gap_type": gap_type })),
        valid_time: Some(valid_time.to_string()),
        recorded_time: None,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Split text into sentences. Splits on ". " and newlines, trims each, and
/// re-attaches the period if it was stripped.
fn extract_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();

    // Split on newlines first, then on ". " within each line.
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split on ". " (sentence boundary).
        let parts: Vec<&str> = line.split(". ").collect();
        for (i, part) in parts.iter().enumerate() {
            let mut s = part.trim().to_string();
            if s.is_empty() {
                continue;
            }
            // Re-attach the period for all but the last part (unless it already
            // ends with one).
            if i < parts.len() - 1 && !s.ends_with('.') {
                s.push('.');
            }
            sentences.push(s);
        }
    }

    // If we got nothing from line splitting, return the whole text as one sentence.
    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim().to_string());
    }

    sentences
}

/// Check if a sentence contains specific factual signals: numbers, dates
/// (20XX), or capitalized words (proper nouns).
fn has_specific_facts(sentence: &str) -> bool {
    // Check for numbers.
    if sentence.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }

    // Check for dates (20XX pattern).
    if sentence.contains("20") {
        let bytes = sentence.as_bytes();
        for i in 0..bytes.len().saturating_sub(3) {
            if bytes[i] == b'2'
                && bytes[i + 1] == b'0'
                && bytes[i + 2].is_ascii_digit()
                && bytes[i + 3].is_ascii_digit()
            {
                return true;
            }
        }
    }

    // Check for proper nouns (capitalized words that aren't at the start).
    let words: Vec<&str> = sentence.split_whitespace().collect();
    if words.len() > 1 {
        for word in &words[1..] {
            let first_char = word.chars().next();
            if let Some(c) = first_char {
                if c.is_uppercase() {
                    return true;
                }
            }
        }
    }

    false
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
        let dir = std::env::temp_dir().join(format!(
            "aidens-autonomous-capture-{name}-{id}-{}",
            std::process::id()
        ));
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
            error: if success {
                None
            } else {
                Some("failed".to_string())
            },
            gap_type: "missing-context".to_string(),
            source_fact_id: fact_id.to_string(),
            source_valid_time: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn extract_sentences_splits_on_period_space() {
        let sentences = extract_sentences("Rust is safe. Rust is fast. Rust is concurrent.");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Rust is safe.");
        assert_eq!(sentences[1], "Rust is fast.");
        assert_eq!(sentences[2], "Rust is concurrent.");
    }

    #[test]
    fn extract_sentences_splits_on_newlines() {
        let sentences = extract_sentences("First sentence here.\nSecond sentence here.\n");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "First sentence here.");
        assert_eq!(sentences[1], "Second sentence here.");
    }

    #[test]
    fn extract_sentences_handles_single_sentence() {
        let sentences = extract_sentences("Only one sentence with no period");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0], "Only one sentence with no period");
    }

    #[test]
    fn extract_sentences_skips_empty_lines() {
        let sentences = extract_sentences("First.\n\n\nSecond.\n");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn has_specific_facts_detects_numbers() {
        assert!(has_specific_facts("The crate has 49 tests passing."));
    }

    #[test]
    fn has_specific_facts_detects_dates() {
        assert!(has_specific_facts("Released in 2024 with new features."));
    }

    #[test]
    fn has_specific_facts_detects_proper_nouns() {
        assert!(has_specific_facts(
            "The Rust language provides memory safety."
        ));
    }

    #[test]
    fn has_specific_facts_returns_false_for_vague() {
        assert!(!has_specific_facts(
            "this is a vague statement about things"
        ));
    }

    #[test]
    fn capture_edge_times_have_explicit_distinct_semantics() {
        let params = capture_edge_params(
            "fact:source".into(),
            "fact:new".into(),
            "job:test",
            "missing-context",
            "2025-01-01T00:00:00Z",
        );
        assert_eq!(params.valid_time.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(params.recorded_time, None);
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
            .add_fact(
                "autonomous",
                "Source fact content here.",
                Some("test"),
                Some(0.5),
            )
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

    #[tokio::test]
    async fn capture_extracts_multiple_sentences() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(
            true,
            "Rust is a systems programming language. It provides memory safety without garbage collection. The borrow checker enforces ownership rules at compile time.",
            "fact:source-abc",
        );

        let outcome = capture.capture(&result).await.unwrap();
        // All three sentences are >30 chars, so all should be captured.
        assert_eq!(outcome.facts_added, 3);
        assert_eq!(outcome.fact_ids.len(), 3);
    }

    #[tokio::test]
    async fn capture_skips_short_sentences() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory.clone(), "http://localhost:1738");

        let result = make_result(
            true,
            "Short. This is a longer sentence that should be captured properly here.",
            "fact:source-abc",
        );

        let outcome = capture.capture(&result).await.unwrap();
        // Only the long sentence should be captured.
        assert_eq!(outcome.facts_added, 1);
    }
}
