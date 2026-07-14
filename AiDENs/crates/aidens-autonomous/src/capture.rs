//! Result capture — stores execution outputs as facts in semantic memory.
//!
//! The [`ResultCapture`] component takes an [`ExecutionResult`] from the
//! executor, extracts individual factual statements from the model output,
//! checks each for duplicates against existing knowledge, and writes unique
//! statements as candidates in `"autonomous_candidates"`. Each candidate binds
//! the exact immutable model-output byte span and execution configuration.

use crate::executor::ExecutionResult;
use aidens_memory_kit::CanonicalMemoryAdapter;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    /// Evidence-bound candidates corresponding to `fact_ids`.
    #[serde(default)]
    pub candidates: Vec<ClaimCandidateV1>,
}

/// Half-open UTF-8 byte range into the immutable execution output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputByteRangeV1 {
    pub start: usize,
    pub end: usize,
}

/// Immutable provenance binding for one extracted claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpanV1 {
    pub source_job_id: String,
    pub output_byte_range: OutputByteRangeV1,
    pub output_byte_len: usize,
    pub output_digest: String,
    pub model_name: String,
    pub prompt_config_digest: String,
}

/// Minimal claim-ledger-compatible candidate used until canonical claim-ledger
/// types are wired into this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimCandidateV1 {
    pub candidate_fact_id: String,
    pub claim: String,
    pub source_spans: Vec<SourceSpanV1>,
    pub retrieval_evidence: Vec<String>,
    #[serde(default)]
    pub contradicting_fact_ids: Vec<String>,
}

/// Captures execution results into semantic memory.
#[derive(Clone)]
pub struct ResultCapture {
    /// Shared canonical memory adapter for search and fact insertion.
    pub memory: Arc<CanonicalMemoryAdapter>,
    /// Semantic-memory HTTP base URL (reserved for future HTTP-based capture).
    pub http_base_url: String,
    /// Model that generated the immutable output artifact.
    pub model_name: String,
    /// Digest binding the prompt and generation configuration.
    pub prompt_config_digest: String,
}

impl std::fmt::Debug for ResultCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCapture")
            .field("http_base_url", &self.http_base_url)
            .field("model_name", &self.model_name)
            .field("prompt_config_digest", &self.prompt_config_digest)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl ResultCapture {
    /// Create a new result capture with the given memory adapter.
    pub fn new(memory: Arc<CanonicalMemoryAdapter>, http_base_url: impl Into<String>) -> Self {
        let http_base_url = http_base_url.into();
        Self {
            memory,
            prompt_config_digest: sha256_digest(&format!(
                "aidens-autonomous-default-capture\0{http_base_url}"
            )),
            http_base_url,
            model_name: "unspecified-model".to_string(),
        }
    }

    /// Bind capture to the exact model and prompt/config digest used by the
    /// executor.
    pub fn with_source_config(
        mut self,
        model_name: impl Into<String>,
        prompt_config_digest: impl Into<String>,
    ) -> Self {
        self.model_name = model_name.into();
        self.prompt_config_digest = prompt_config_digest.into();
        self
    }

    /// Capture an execution result into semantic memory.
    ///
    /// Instead of capturing the entire model output as one fact, this extracts
    /// individual factual statements:
    /// 1. Splits the output by sentences (". " or newlines).
    /// 2. For each sentence >30 chars, checks for duplicates against the KB.
    /// 3. Adds each unique sentence as a separate fact.
    /// 4. Binds each candidate to the exact output byte span, model, and
    ///    prompt/config digest.
    /// 5. Leaves semantic relation assertion to a reviewed promotion path.
    pub async fn capture(&self, result: &ExecutionResult) -> Result<CaptureOutcome> {
        // Don't capture empty or failed outputs.
        if !result.success || result.output.is_empty() {
            return Ok(CaptureOutcome {
                facts_added: 0,
                facts_skipped_duplicates: 0,
                fact_ids: Vec::new(),
                candidates: Vec::new(),
            });
        }

        let sentences = extract_sentence_spans(&result.output);

        let mut facts_added = 0usize;
        let mut facts_skipped_duplicates = 0usize;
        let mut fact_ids: Vec<String> = Vec::new();
        let mut candidates = Vec::new();

        for extracted in &sentences {
            let sentence = &extracted.text;
            // Skip short sentences.
            if sentence.len() < 30 {
                continue;
            }

            // Search for duplicates in both the autonomous and autonomous_candidates namespaces.
            let existing = self
                .memory
                .search(
                    sentence,
                    Some(&[
                        "autonomous".to_string(),
                        "autonomous_candidates".to_string(),
                    ]),
                    Some(5),
                )
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

            let supporting = self
                .memory
                .search(sentence, Some(&["autonomous".to_string()]), Some(5))
                .await?;
            let retrieval_evidence = supporting
                .iter()
                .map(|result| result.source.result_id())
                .collect::<Vec<_>>();
            let contradicting_fact_ids = supporting
                .iter()
                .filter(|existing| claims_contradict(sentence, &existing.content))
                .map(|existing| existing.source.result_id())
                .collect::<Vec<_>>();

            // Determine confidence based on content quality.
            let confidence = if has_specific_facts(sentence) {
                0.8
            } else {
                0.5
            };

            // AUTO-001 fix: write to quarantine namespace first.
            // Facts are only promoted to "autonomous" after evaluation/audit gates pass.
            // Rejected candidates remain in "autonomous_candidates" and are not searchable
            // in the normal "autonomous" namespace.
            let source_span = SourceSpanV1 {
                source_job_id: result.job_id.clone(),
                output_byte_range: OutputByteRangeV1 {
                    start: extracted.start,
                    end: extracted.end,
                },
                output_byte_len: result.output.len(),
                output_digest: sha256_digest(&result.output),
                model_name: self.model_name.clone(),
                prompt_config_digest: self.prompt_config_digest.clone(),
            };
            let source_binding = serde_json::to_string(&source_span)?;
            let fact_id = self
                .memory
                .add_fact(
                    "autonomous_candidates",
                    sentence,
                    Some(&source_binding),
                    Some(confidence),
                )
                .await?;

            candidates.push(ClaimCandidateV1 {
                candidate_fact_id: fact_id.clone(),
                claim: sentence.clone(),
                source_spans: vec![source_span],
                retrieval_evidence,
                contradicting_fact_ids,
            });

            facts_added += 1;
            fact_ids.push(fact_id);
        }

        Ok(CaptureOutcome {
            facts_added,
            facts_skipped_duplicates,
            fact_ids,
            candidates,
        })
    }

    /// Append a canonical fact only when the candidate has at least one valid
    /// immutable source span. Candidate history remains append-only.
    pub async fn promote_candidate(&self, candidate: &ClaimCandidateV1) -> Result<String> {
        let span = candidate
            .source_spans
            .first()
            .filter(|span| {
                span.output_byte_range.start < span.output_byte_range.end
                    && span.output_byte_range.end <= span.output_byte_len
                    && !span.source_job_id.is_empty()
                    && !span.output_digest.is_empty()
                    && !span.model_name.is_empty()
                    && !span.prompt_config_digest.is_empty()
            })
            .ok_or_else(|| anyhow!("candidate promotion requires an immutable source span"))?;
        let source_binding = serde_json::to_string(span)?;
        Ok(self
            .memory
            .add_fact(
                "autonomous",
                &candidate.claim,
                Some(&source_binding),
                Some(0.8),
            )
            .await?)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Split text into sentences. Splits on ". " and newlines, trims each, and
/// re-attaches the period if it was stripped.
fn extract_sentences(text: &str) -> Vec<String> {
    extract_sentence_spans(text)
        .into_iter()
        .map(|sentence| sentence.text)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExtractedSentence {
    text: String,
    start: usize,
    end: usize,
}

fn extract_sentence_spans(text: &str) -> Vec<ExtractedSentence> {
    let mut sentences = Vec::new();
    let mut line_offset = 0usize;

    for line_with_newline in text.split_inclusive('\n') {
        let line = match line_with_newline.strip_suffix('\n') {
            Some(without_newline) => without_newline,
            None => line_with_newline,
        };
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            line_offset += line_with_newline.len();
            continue;
        }
        let trim_start = line.len() - line.trim_start().len();
        let line_start = line_offset + trim_start;
        let mut part_start = 0usize;
        while part_start < trimmed_line.len() {
            let remaining = &trimmed_line[part_start..];
            let boundary = remaining.find(". ");
            let raw_end = boundary
                .map(|relative| part_start + relative + 1)
                .unwrap_or(trimmed_line.len());
            let raw = &trimmed_line[part_start..raw_end];
            let sentence = raw.trim();
            if sentence.is_empty() {
                part_start = raw_end.saturating_add(1);
                continue;
            }
            let leading = raw.len() - raw.trim_start().len();
            let start = line_start + part_start + leading;
            let end = start + sentence.len();
            sentences.push(ExtractedSentence {
                text: sentence.to_string(),
                start,
                end,
            });
            part_start = match boundary {
                Some(_) => raw_end + 1,
                None => trimmed_line.len(),
            };
        }
        line_offset += line_with_newline.len();
    }

    // If we got nothing from line splitting, return the whole text as one sentence.
    if sentences.is_empty() && !text.trim().is_empty() {
        let trimmed = text.trim();
        let start = text.len() - text.trim_start().len();
        sentences.push(ExtractedSentence {
            text: trimmed.to_string(),
            start,
            end: start + trimmed.len(),
        });
    }

    sentences
}

fn sha256_digest(material: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(material.as_bytes()))
}

fn claims_contradict(left: &str, right: &str) -> bool {
    let left_words = normalized_words(left);
    let right_words = normalized_words(right);
    let left_negated = contains_negation(left);
    let right_negated = contains_negation(right);
    if left_negated == right_negated || left_words.is_empty() || right_words.is_empty() {
        return false;
    }
    let overlap = left_words
        .iter()
        .filter(|word| right_words.contains(*word))
        .count();
    let denominator = left_words.len().min(right_words.len());
    overlap * 4 >= denominator * 3
}

fn contains_negation(value: &str) -> bool {
    value
        .split(|character: char| !character.is_alphanumeric())
        .any(|word| matches!(word.to_ascii_lowercase().as_str(), "not" | "no" | "never"))
}

fn normalized_words(value: &str) -> std::collections::HashSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|word| word.len() > 2 && !matches!(word.as_str(), "not" | "never" | "the" | "and"))
        .collect()
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
    fn contradiction_check_requires_shared_subject_and_opposite_polarity() {
        assert!(claims_contradict(
            "The durable queue does not persist committed entries after restart.",
            "The durable queue persists committed entries after restart."
        ));
        assert!(!claims_contradict(
            "The durable queue persists committed entries after restart.",
            "A parser rejects malformed executable tool calls."
        ));
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
    async fn capture_does_not_assert_graph_edge_before_review() {
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

        // Candidate capture is not authorized to assert a semantic relation.
        let new_fact_id = &outcome.fact_ids[0];
        let new_node = if new_fact_id.starts_with("fact:") {
            new_fact_id.clone()
        } else {
            format!("fact:{new_fact_id}")
        };
        let edges = memory.list_graph_edges(&new_node).await.unwrap();
        assert!(
            edges.is_empty(),
            "candidate capture must not add graph edges"
        );
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

    #[tokio::test]
    async fn captured_candidate_binds_job_and_exact_output_byte_span() {
        let memory = mock_memory();
        let capture = ResultCapture::new(memory, "http://localhost:1738")
            .with_source_config("test-model", "sha256:test-prompt-config");
        let output = "Prefix. Rust preserves exact UTF-8 byte spans for every captured claim.";
        let result = make_result(true, output, "fact:source-abc");

        let outcome = capture.capture(&result).await.expect("capture candidate");
        let candidate = outcome.candidates.first().expect("captured candidate");
        assert_eq!(candidate.source_spans.len(), 1);
        let span = &candidate.source_spans[0];
        assert_eq!(span.source_job_id, "job:test-001");
        assert_eq!(span.model_name, "test-model");
        assert_eq!(span.prompt_config_digest, "sha256:test-prompt-config");
        assert!(span.output_digest.starts_with("sha256:"));
        assert_eq!(
            &output[span.output_byte_range.start..span.output_byte_range.end],
            candidate.claim
        );
    }
}
