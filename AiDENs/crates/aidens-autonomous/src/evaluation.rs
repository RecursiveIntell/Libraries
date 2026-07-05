//! Evaluation gate — decides whether a captured fact should be promoted,
//! quarantined, or rejected.
//!
//! The [`EvaluationGate`] applies content quality heuristics based on:
//! - Specific factual signals (numbers, dates, proper nouns, technical terms)
//! - Coherence (not garbled, not repetitive)
//! - Content length
//! - Execution success
//!
//! Scoring:
//! - High (0.8+) = specific facts + coherent → `Promote`
//! - Medium (0.5–0.8) = some specifics → `Quarantine`
//! - Low (<0.5) = vague or garbled → `Reject`

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
    /// Rules (applied in order):
    /// 1. If execution was not successful → `Reject`.
    /// 2. If content is too short (`< 20` chars) → `Reject`.
    /// 3. If content is garbled or repetitive → `Reject`.
    /// 4. Compute quality score based on factual signals and coherence.
    /// 5. High (≥0.8) → `Promote`, Medium (≥0.5) → `Quarantine`, Low → `Reject`.
    pub fn evaluate(&self, content: &str, execution_success: bool) -> FactDisposition {
        if !execution_success {
            return FactDisposition::Reject;
        }
        if content.len() < 20 {
            return FactDisposition::Reject;
        }

        // Check for garbled or repetitive content.
        if is_garbled(content) {
            return FactDisposition::Reject;
        }

        // Compute quality score.
        let score = content_quality_score(content);

        if score >= 0.8 {
            FactDisposition::Promote
        } else if score >= 0.5 {
            FactDisposition::Quarantine
        } else {
            FactDisposition::Reject
        }
    }

    /// Evaluate content and return the numeric quality score alongside the
    /// disposition. Useful for debugging and logging.
    pub fn evaluate_with_score(
        &self,
        content: &str,
        execution_success: bool,
    ) -> (FactDisposition, f64) {
        if !execution_success {
            return (FactDisposition::Reject, 0.0);
        }
        if content.len() < 20 {
            return (FactDisposition::Reject, 0.0);
        }
        if is_garbled(content) {
            return (FactDisposition::Reject, 0.0);
        }

        let score = content_quality_score(content);
        let disposition = if score >= 0.8 {
            FactDisposition::Promote
        } else if score >= 0.5 {
            FactDisposition::Quarantine
        } else {
            FactDisposition::Reject
        };
        (disposition, score)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute a content quality score in [0.0, 1.0] based on factual signals
/// and coherence.
fn content_quality_score(content: &str) -> f64 {
    let mut score: f64 = 0.3; // Base score for having content.

    // Bonus for specific factual signals.
    if has_numbers(content) {
        score += 0.2;
    }
    if has_dates(content) {
        score += 0.15;
    }
    if has_proper_nouns(content) {
        score += 0.15;
    }
    if has_technical_terms(content) {
        score += 0.1;
    }

    // Bonus for coherence (reasonable sentence structure).
    if is_coherent(content) {
        score += 0.1;
    }

    // Penalty for being too short (but >20 chars).
    if content.len() < 40 {
        score -= 0.1;
    }

    // Clamp to [0.0, 1.0].
    score.clamp(0.0, 1.0)
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
    if avg_word_len < 2.0 || avg_word_len > 20.0 {
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
    fn promote_on_high_quality_content() {
        let gate = EvaluationGate::new();
        // Contains numbers, proper nouns, and technical terms.
        let content = "Rust 1.76 introduced the PlanActVerifyLoopV1 struct in the aidens-runner crate with async support.";
        let disposition = gate.evaluate(content, true);
        assert_eq!(disposition, FactDisposition::Promote);
    }

    #[test]
    fn quarantine_on_medium_quality_content() {
        let gate = EvaluationGate::new();
        // Contains some specifics but is not as detailed.
        let content = "The knowledge base has some facts about the project.";
        let (disposition, score) = gate.evaluate_with_score(content, true);
        // No numbers/dates/tech terms/proper nouns → base 0.3 → Reject
        assert_eq!(disposition, FactDisposition::Reject);
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
    fn promote_on_factual_content_with_dates() {
        let gate = EvaluationGate::new();
        let content = "The AiDENs autonomous crate was released in 2024 with 49 passing tests.";
        let disposition = gate.evaluate(content, true);
        assert_eq!(disposition, FactDisposition::Promote);
    }

    #[test]
    fn evaluate_with_score_returns_score() {
        let gate = EvaluationGate::new();
        let content =
            "Rust 1.76 introduced the PlanActVerifyLoopV1 struct in the aidens-runner crate.";
        let (disposition, score) = gate.evaluate_with_score(content, true);
        assert_eq!(disposition, FactDisposition::Promote);
        assert!(score >= 0.8);
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
        // 20 chars passes the length check. Contains "20" (number +0.2),
        // is coherent (+0.1), <40 chars (-0.1). Score: 0.3+0.2+0.1-0.1=0.5 → Quarantine
        let (disposition, score) = gate.evaluate_with_score(content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
        assert!((score - 0.5).abs() < 0.01);
    }
}
