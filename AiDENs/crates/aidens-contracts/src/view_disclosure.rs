//! P13 — Multi-view runtime, query provenance, and view disclosure.
//!
//! Every search/query result declares which view answered, whether widening
//! occurred, and what evidence anchors the result.

use serde::{Deserialize, Serialize};

/// Which runtime view produced a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewType {
    /// Semantic similarity search.
    Semantic,
    /// Temporal / as-of query.
    Temporal,
    /// Entity graph traversal.
    Entity,
    /// Causal chain analysis.
    Causal,
    /// Control / governance state.
    Control,
    /// Execution receipt replay.
    Execution,
}

/// Disclosure of which view answered a query and whether widening occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDisclosureV1 {
    /// Which view produced this result.
    pub view_type: ViewType,
    /// Whether the query widened (e.g., temporal fell back to timeless).
    pub widening_occurred: bool,
    /// Why widening occurred (if it did).
    pub widening_reason: Option<String>,
    /// Evidence anchors (receipt IDs, fact IDs) supporting the result.
    pub evidence_anchors: Vec<String>,
    /// Whether the result is verified or advisory.
    pub verification_status: ViewVerificationStatus,
}

/// Verification status of a view result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewVerificationStatus {
    /// Result is from verified/receipt-bearing source.
    Verified,
    /// Result is advisory (not yet verified).
    Advisory,
    /// Result is from a degraded path.
    Degraded,
}

impl ViewDisclosureV1 {
    /// Create a clean disclosure with no widening.
    pub fn clean(view_type: ViewType, evidence_anchors: Vec<String>) -> Self {
        Self {
            view_type,
            widening_occurred: false,
            widening_reason: None,
            evidence_anchors,
            verification_status: ViewVerificationStatus::Verified,
        }
    }

    /// Create a disclosure with widening.
    pub fn widened(view_type: ViewType, reason: &str, evidence_anchors: Vec<String>) -> Self {
        Self {
            view_type,
            widening_occurred: true,
            widening_reason: Some(reason.to_string()),
            evidence_anchors,
            verification_status: ViewVerificationStatus::Advisory,
        }
    }

    /// Create a degraded disclosure.
    pub fn degraded(view_type: ViewType, reason: &str) -> Self {
        Self {
            view_type,
            widening_occurred: true,
            widening_reason: Some(reason.to_string()),
            evidence_anchors: vec![],
            verification_status: ViewVerificationStatus::Degraded,
        }
    }
}

/// Runtime query provenance — tracks how a query was answered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeQueryProvenanceV1 {
    /// The original query.
    pub query: String,
    /// Which view was used.
    pub view_used: ViewType,
    /// Which retrieval stages fired.
    pub stages_fired: Vec<String>,
    /// Number of results returned.
    pub result_count: usize,
    /// View disclosure details.
    pub disclosure: ViewDisclosureV1,
    /// Timestamp (RFC3339).
    pub timestamp: String,
}

impl RuntimeQueryProvenanceV1 {
    /// Create provenance for a clean query.
    pub fn new(query: &str, view_type: ViewType, stages: Vec<String>, result_count: usize) -> Self {
        let disclosure = ViewDisclosureV1::clean(view_type, vec![]);
        Self {
            query: query.to_string(),
            view_used: view_type,
            stages_fired: stages,
            result_count,
            disclosure,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create provenance for a widened query.
    pub fn widened(
        query: &str,
        view_type: ViewType,
        stages: Vec<String>,
        result_count: usize,
        reason: &str,
    ) -> Self {
        let disclosure = ViewDisclosureV1::widened(view_type, reason, vec![]);
        Self {
            query: query.to_string(),
            view_used: view_type,
            stages_fired: stages,
            result_count,
            disclosure,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_disclosure_no_widening() {
        let d = ViewDisclosureV1::clean(ViewType::Semantic, vec!["fact:123".into()]);
        assert!(!d.widening_occurred);
        assert_eq!(d.verification_status, ViewVerificationStatus::Verified);
    }

    #[test]
    fn widened_disclosure_marks_advisory() {
        let d = ViewDisclosureV1::widened(ViewType::Temporal, "fell back to timeless", vec![]);
        assert!(d.widening_occurred);
        assert_eq!(d.verification_status, ViewVerificationStatus::Advisory);
        assert_eq!(d.widening_reason.as_deref(), Some("fell back to timeless"));
    }

    #[test]
    fn degraded_disclosure_marks_degraded() {
        let d = ViewDisclosureV1::degraded(ViewType::Causal, "missing causal chain");
        assert!(d.widening_occurred);
        assert_eq!(d.verification_status, ViewVerificationStatus::Degraded);
    }

    #[test]
    fn provenance_records_stages() {
        let p = RuntimeQueryProvenanceV1::new(
            "turbo-quant",
            ViewType::Semantic,
            vec!["bm25".into(), "vector".into()],
            5,
        );
        assert_eq!(p.stages_fired.len(), 2);
        assert!(!p.disclosure.widening_occurred);
    }

    #[test]
    fn provenance_widened_marks_disclosure() {
        let p = RuntimeQueryProvenanceV1::widened(
            "what did I know in January",
            ViewType::Temporal,
            vec!["bm25".into()],
            3,
            "temporal index incomplete, fell back to timeless",
        );
        assert!(p.disclosure.widening_occurred);
        assert_eq!(
            p.disclosure.verification_status,
            ViewVerificationStatus::Advisory
        );
    }

    #[test]
    fn time_scoped_query_cannot_silently_fall_back() {
        // A time-scoped query that widens to timeless MUST disclose it
        let p = RuntimeQueryProvenanceV1::widened(
            "facts from June 2026",
            ViewType::Temporal,
            vec!["bm25".into()],
            10,
            "no temporal index, fell back to timeless search",
        );
        assert!(p.disclosure.widening_occurred);
        assert!(p.disclosure.widening_reason.is_some());
        assert_ne!(
            p.disclosure.verification_status,
            ViewVerificationStatus::Verified
        );
    }
}
