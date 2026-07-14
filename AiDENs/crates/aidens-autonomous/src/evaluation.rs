//! Evaluation gate — decides whether a captured fact should be promoted,
//! quarantined, or rejected.
//!
//! Lexical quality is advisory. Promotion requires immutable source binding and
//! retrieval evidence, while contradictions and insufficient evidence force
//! abstention into quarantine.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Disposition of a captured fact after evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactDisposition {
    /// Fact is high-quality and should be promoted to the knowledge base.
    Promote,
    /// Fact is uncertain — hold for review or further verification.
    Quarantine,
    /// Fact should be rejected and not stored.
    Reject,
}

/// Quality summary for an immutable model-output source span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpanQualityV1 {
    pub start: usize,
    pub end: usize,
    pub source_len: usize,
    pub output_digest_present: bool,
    pub model_name_present: bool,
    pub prompt_config_digest_present: bool,
}

impl SourceSpanQualityV1 {
    fn is_complete(&self) -> bool {
        self.start < self.end
            && self.end <= self.source_len
            && self.output_digest_present
            && self.model_name_present
            && self.prompt_config_digest_present
    }
}

/// Evidence-bearing input for evaluating one extracted claim.
#[derive(Debug, Clone)]
pub struct ClaimEvaluationInputV1<'a> {
    pub content: &'a str,
    pub execution_success: bool,
    pub retrieval_evidence: Vec<String>,
    pub contradicting_fact_ids: Vec<String>,
    pub source_span: Option<SourceSpanQualityV1>,
}

/// Inspectable evaluation artifact for one claim candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationReportV1 {
    pub disposition: FactDisposition,
    pub score: f64,
    pub retrieval_evidence_count: usize,
    pub source_span_complete: bool,
    pub contradiction_detected: bool,
    pub abstained: bool,
}

/// Evaluates captured facts to determine their disposition.
#[derive(Debug, Clone, Default)]
pub struct EvaluationGate;

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl EvaluationGate {
    /// Create a new evaluation gate.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a captured fact and return its disposition.
    ///
    /// Legacy content-only entry point. With no evidence or source span, valid
    /// content abstains into quarantine and can never be promoted.
    pub fn evaluate(&self, content: &str, execution_success: bool) -> FactDisposition {
        self.evaluate_claim(&ClaimEvaluationInputV1 {
            content,
            execution_success,
            retrieval_evidence: Vec::new(),
            contradicting_fact_ids: Vec::new(),
            source_span: None,
        })
        .disposition
    }

    /// Evaluate content and return the numeric quality score alongside the
    /// disposition. Useful for debugging and logging.
    pub fn evaluate_with_score(
        &self,
        content: &str,
        execution_success: bool,
    ) -> (FactDisposition, f64) {
        let report = self.evaluate_claim(&ClaimEvaluationInputV1 {
            content,
            execution_success,
            retrieval_evidence: Vec::new(),
            contradicting_fact_ids: Vec::new(),
            source_span: None,
        });
        (report.disposition, report.score)
    }

    /// Evaluate one claim using evidence, contradiction, and source-span
    /// features. Lexical signals contribute at most 25% of the score.
    pub fn evaluate_claim(&self, input: &ClaimEvaluationInputV1<'_>) -> EvaluationReportV1 {
        let invalid = !input.execution_success
            || input.content.len() < 20
            || is_garbled(input.content)
            || !is_coherent(input.content);
        if invalid {
            return EvaluationReportV1 {
                disposition: FactDisposition::Reject,
                score: 0.0,
                retrieval_evidence_count: input.retrieval_evidence.len(),
                source_span_complete: false,
                contradiction_detected: !input.contradicting_fact_ids.is_empty(),
                abstained: false,
            };
        }

        let source_span_complete = input
            .source_span
            .as_ref()
            .is_some_and(SourceSpanQualityV1::is_complete);
        let has_retrieval_evidence = !input.retrieval_evidence.is_empty();
        let contradiction_detected = !input.contradicting_fact_ids.is_empty();
        let mut score = lexical_quality_score(input.content);
        if has_retrieval_evidence {
            score += 0.40;
        }
        if source_span_complete {
            score += 0.35;
        }
        if contradiction_detected {
            score = score.min(0.49);
        }
        let score = score.clamp(0.0, 1.0);
        let insufficient_evidence = !has_retrieval_evidence || !source_span_complete;
        let abstained = insufficient_evidence || contradiction_detected;
        let disposition = if abstained {
            FactDisposition::Quarantine
        } else if score >= 0.75 {
            FactDisposition::Promote
        } else {
            FactDisposition::Quarantine
        };

        EvaluationReportV1 {
            disposition,
            score,
            retrieval_evidence_count: input.retrieval_evidence.len(),
            source_span_complete,
            contradiction_detected,
            abstained,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute a content quality score in [0.0, 1.0] based on factual signals
/// and coherence.
fn lexical_quality_score(content: &str) -> f64 {
    let mut score: f64 = 0.05;

    // Bonus for specific factual signals.
    if has_numbers(content) {
        score += 0.03;
    }
    if has_dates(content) {
        score += 0.02;
    }
    if has_proper_nouns(content) {
        score += 0.03;
    }
    if has_technical_terms(content) {
        score += 0.02;
    }

    // Bonus for coherence (reasonable sentence structure).
    if is_coherent(content) {
        score += 0.10;
    }

    // Penalty for being too short (but >20 chars).
    if content.len() < 40 {
        score -= 0.02;
    }

    // Clamp to [0.0, 1.0].
    score.clamp(0.0, 0.25)
}

/// Check if content contains numeric digits.
fn has_numbers(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

/// Check if content contains a date pattern (20XX).
fn has_dates(s: &str) -> bool {
    let bytes = s.as_bytes();
    for i in 0..bytes.len().saturating_sub(3) {
        if bytes[i] == b'2'
            && bytes[i + 1] == b'0'
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
        {
            return true;
        }
    }
    false
}

/// Check if content contains proper nouns (capitalized words not at the start).
fn has_proper_nouns(s: &str) -> bool {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() <= 1 {
        return false;
    }
    for word in &words[1..] {
        if let Some(c) = word.chars().next() {
            if c.is_uppercase() {
                return true;
            }
        }
    }
    false
}

/// Check if content contains technical terms (common in programming/science).
fn has_technical_terms(s: &str) -> bool {
    let tech_indicators = [
        "API", "crate", "struct", "enum", "trait", "impl", "module", "function", "compiler",
        "runtime", "async", "await", "memory", "buffer", "queue", "kernel", "hash", "token",
        "parse", "schema", "vector", "iterator", "protocol", "endpoint", "request", "response",
        "version", "config",
    ];
    let s_lower = s.to_lowercase();
    tech_indicators
        .iter()
        .any(|t| s_lower.contains(&t.to_lowercase()))
}

/// Check if content is coherent — not garbled, not overly repetitive.
fn is_coherent(s: &str) -> bool {
    let words: Vec<&str> = s.split_whitespace().collect();

    // Too few words is not coherent.
    if words.len() < 3 {
        return false;
    }

    // Check for excessive repetition (same word >50% of content).
    let mut word_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for word in &words {
        *word_counts.entry(word).or_insert(0) += 1;
    }
    let max_count = word_counts.values().copied().max().unwrap_or(0);
    if max_count > words.len() / 2 {
        return false;
    }

    // Check for reasonable average word length (not garbled).
    let total_chars: usize = words.iter().map(|w| w.len()).sum();
    let avg_word_len = total_chars as f64 / words.len() as f64;
    if !(2.0..=20.0).contains(&avg_word_len) {
        return false;
    }

    true
}

/// Check if content is garbled — random characters, excessive symbols, or
/// encoding artifacts.
fn is_garbled(s: &str) -> bool {
    let alphanumeric_count = s.chars().filter(|c| c.is_alphanumeric()).count();
    let total_count = s.chars().count();

    if total_count == 0 {
        return true;
    }

    let ratio = alphanumeric_count as f64 / total_count as f64;
    // If less than 50% alphanumeric, it's likely garbled.
    if ratio < 0.5 {
        return true;
    }

    // Check for excessive repetition of single characters.
    let mut char_counts: std::collections::HashMap<char, usize> = std::collections::HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }
    let max_char_count = char_counts.values().copied().max().unwrap_or(0);
    if max_char_count > total_count / 3 {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_on_failed_execution() {
        let gate = EvaluationGate::new();
        let disposition = gate.evaluate("This is a long enough content string.", false);
        assert_eq!(disposition, FactDisposition::Reject);
    }

    #[test]
    fn reject_on_short_content() {
        let gate = EvaluationGate::new();
        let disposition = gate.evaluate("short", true);
        assert_eq!(disposition, FactDisposition::Reject);
    }

    #[test]
    fn reject_on_empty_content() {
        let gate = EvaluationGate::new();
        let disposition = gate.evaluate("", true);
        assert_eq!(disposition, FactDisposition::Reject);
    }

    #[test]
    fn fluent_content_without_evidence_abstains() {
        let gate = EvaluationGate::new();
        // Contains numbers, proper nouns, and technical terms.
        let content = "Rust 1.76 introduced the PlanActVerifyLoopV1 struct in the aidens-runner crate with async support.";
        let disposition = gate.evaluate(content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
    }

    #[test]
    fn quarantine_on_medium_quality_content() {
        let gate = EvaluationGate::new();
        // Contains some specifics but is not as detailed.
        let content = "The knowledge base has some facts about the project.";
        let (disposition, score) = gate.evaluate_with_score(content, true);
        // Coherent but unsupported content must abstain rather than promote.
        assert_eq!(disposition, FactDisposition::Quarantine);
        assert!(score < 0.5);
    }

    #[test]
    fn reject_on_garbled_content() {
        let gate = EvaluationGate::new();
        let garbled = "asdf123!@#$$$%%%^^^&&&****(((())))----";
        let disposition = gate.evaluate(garbled, true);
        assert_eq!(disposition, FactDisposition::Reject);
    }

    #[test]
    fn reject_on_repetitive_content() {
        let gate = EvaluationGate::new();
        let repetitive = "the the the the the the the the the the the the the the the the";
        let disposition = gate.evaluate(repetitive, true);
        assert_eq!(disposition, FactDisposition::Reject);
    }

    #[test]
    fn dates_without_evidence_do_not_promote() {
        let gate = EvaluationGate::new();
        let content = "The AiDENs autonomous crate was released in 2024 with 49 passing tests.";
        let disposition = gate.evaluate(content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
    }

    #[test]
    fn evaluate_with_score_returns_score() {
        let gate = EvaluationGate::new();
        let content =
            "Rust 1.76 introduced the PlanActVerifyLoopV1 struct in the aidens-runner crate.";
        let (disposition, score) = gate.evaluate_with_score(content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
        assert!(score <= 0.25);
    }

    #[test]
    fn evaluate_with_score_rejects_on_failure() {
        let gate = EvaluationGate::new();
        let (disposition, score) = gate.evaluate_with_score("some content", false);
        assert_eq!(disposition, FactDisposition::Reject);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn has_numbers_detects_digits() {
        assert!(has_numbers("49 tests"));
        assert!(!has_numbers("no numbers here"));
    }

    #[test]
    fn has_dates_detects_year_patterns() {
        assert!(has_dates("Released in 2024"));
        assert!(has_dates("Updated 2025-06-15"));
        assert!(!has_dates("No year mentioned"));
    }

    #[test]
    fn has_proper_nouns_detects_capitalized_words() {
        assert!(has_proper_nouns("the Rust language is great"));
        assert!(!has_proper_nouns("all lowercase words here"));
    }

    #[test]
    fn has_technical_terms_detects_tech_words() {
        assert!(has_technical_terms(
            "The API endpoint returns a JSON response"
        ));
        assert!(has_technical_terms("The crate uses async runtime"));
        assert!(!has_technical_terms("The cat sat on the mat"));
    }

    #[test]
    fn is_coherent_validates_reasonable_text() {
        assert!(is_coherent(
            "Rust is a systems programming language with memory safety."
        ));
        assert!(!is_coherent("aa"));
        assert!(!is_coherent("word word word word word word word word"));
    }

    #[test]
    fn is_garbled_detects_symbol_heavy_content() {
        assert!(is_garbled("!@#$%^&*()!@#$%^&*()"));
        assert!(!is_garbled("This is normal text with some content."));
    }

    #[test]
    fn disposition_serializes_to_lowercase() {
        let json = serde_json::to_string(&FactDisposition::Promote).unwrap();
        assert_eq!(json, "\"promote\"");

        let json = serde_json::to_string(&FactDisposition::Quarantine).unwrap();
        assert_eq!(json, "\"quarantine\"");

        let json = serde_json::to_string(&FactDisposition::Reject).unwrap();
        assert_eq!(json, "\"reject\"");
    }

    #[test]
    fn exactly_20_chars_is_not_rejected_for_length() {
        let gate = EvaluationGate::new();
        let content = "This is exactly 20ch"; // 20 chars
        assert_eq!(content.len(), 20);
        // 20 chars passes the length check but has no promotion evidence.
        let (disposition, score) = gate.evaluate_with_score(content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
        assert!(score <= 0.25);
    }

    #[test]
    fn sourced_claim_scores_above_fluent_unsourced_claim() {
        let gate = EvaluationGate::new();
        let fluent = ClaimEvaluationInputV1 {
            content: "The elegant Runtime API schema shipped in 2026 with 99 reliable endpoints.",
            execution_success: true,
            retrieval_evidence: vec![],
            contradicting_fact_ids: vec![],
            source_span: None,
        };
        let sourced = ClaimEvaluationInputV1 {
            content: "the queue stores entries durably",
            execution_success: true,
            retrieval_evidence: vec!["fact:queue-contract".into()],
            contradicting_fact_ids: vec![],
            source_span: Some(SourceSpanQualityV1 {
                start: 0,
                end: 33,
                source_len: 33,
                output_digest_present: true,
                model_name_present: true,
                prompt_config_digest_present: true,
            }),
        };

        let fluent_report = gate.evaluate_claim(&fluent);
        let sourced_report = gate.evaluate_claim(&sourced);
        assert!(sourced_report.score > fluent_report.score);
        assert_eq!(fluent_report.disposition, FactDisposition::Quarantine);
    }

    #[test]
    fn contradiction_forces_quarantine() {
        let gate = EvaluationGate::new();
        let report = gate.evaluate_claim(&ClaimEvaluationInputV1 {
            content: "The queue stores entries durably with a committed append receipt.",
            execution_success: true,
            retrieval_evidence: vec!["fact:support".into()],
            contradicting_fact_ids: vec!["fact:contradiction".into()],
            source_span: Some(SourceSpanQualityV1 {
                start: 0,
                end: 64,
                source_len: 64,
                output_digest_present: true,
                model_name_present: true,
                prompt_config_digest_present: true,
            }),
        });

        assert_eq!(report.disposition, FactDisposition::Quarantine);
        assert!(report.contradiction_detected);
    }
}
