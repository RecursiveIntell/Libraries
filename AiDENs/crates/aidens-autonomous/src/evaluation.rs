//! Evaluation gate — decides whether a captured fact should be promoted,
//! quarantined, or rejected.
//!
//! The [`EvaluationGate`] applies simple heuristics based on content length
//! and execution success to determine a [`FactDisposition`]. This is a
//! first-pass gate; future versions may integrate governance checks and
//! contradiction detection.

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
    /// 3. If content is substantial (`> 50` chars) and execution succeeded →
    ///    `Promote`.
    /// 4. Otherwise → `Quarantine`.
    pub fn evaluate(&self, content: &str, execution_success: bool) -> FactDisposition {
        if !execution_success {
            return FactDisposition::Reject;
        }
        if content.len() < 20 {
            return FactDisposition::Reject;
        }
        if content.len() > 50 && execution_success {
            return FactDisposition::Promote;
        }
        FactDisposition::Quarantine
    }
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
    fn promote_on_long_content_and_success() {
        let gate = EvaluationGate::new();
        let long_content = "This is a sufficiently long content string that exceeds fifty characters.";
        assert!(long_content.len() > 50);
        let disposition = gate.evaluate(long_content, true);
        assert_eq!(disposition, FactDisposition::Promote);
    }

    #[test]
    fn quarantine_on_medium_content_and_success() {
        let gate = EvaluationGate::new();
        // Content between 20 and 50 chars.
        let medium_content = "This is medium content.";
        assert!(medium_content.len() >= 20);
        assert!(medium_content.len() <= 50);
        let disposition = gate.evaluate(medium_content, true);
        assert_eq!(disposition, FactDisposition::Quarantine);
    }

    #[test]
    fn exactly_20_chars_is_not_rejected() {
        let gate = EvaluationGate::new();
        let content = "This is exactly 20ch"; // 20 chars
        assert_eq!(content.len(), 20);
        let disposition = gate.evaluate(content, true);
        // 20 chars is not < 20, so not rejected. 20 is not > 50, so not promoted.
        assert_eq!(disposition, FactDisposition::Quarantine);
    }

    #[test]
    fn exactly_51_chars_is_promoted() {
        let gate = EvaluationGate::new();
        let content = "This content is exactly fifty-one characters long!!";
        assert_eq!(content.len(), 51);
        let disposition = gate.evaluate(content, true);
        assert_eq!(disposition, FactDisposition::Promote);
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
}