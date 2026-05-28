//! Core domain types for the claim ledger.
//!
//! ## Support State Machine
//!
//! Claims move through support states as follows:
//!
//! ```text
//! [UNKNOWN/HEURISTIC_ONLY] ──admit_support──► [SUPPORTED]
//! [UNKNOWN/HEURISTIC_ONLY] ──admit_support──► [PARTIALLY_SUPPORTED]
//! [UNKNOWN/HEURISTIC_ONLY] ──admit_support──► [UNSUPPORTED]
//! [SUPPORTED/PARTIALLY_SUPPORTED] ──contradiction confirmed──► [CONTRADICTED]
//! [CONTRADICTED] ──contradiction rejected──► [HEURISTIC_ONLY or prior state]
//! ```
//!
//! Unsupported is bundle-scoped only — it signals no supporting evidence was found
//! for this specific bundle, not a universal verdict.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids;

/// Support states for a claim within a specific evidence bundle scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportState {
    /// Positive evidence was found in the bundle.
    Supported,
    /// Mixed or partial evidence was found.
    PartiallySupported,
    /// No supporting evidence found in this bundle.
    Unsupported,
    /// A resolved contradiction applies to this claim.
    Contradicted,
    /// Heuristic matching was used; no strong proof exists.
    HeuristicOnly,
    /// No support judgment has been made yet.
    Unknown,
}

impl SupportState {
    /// Returns true for states that constitute a "strong" admission
    /// (i.e., have proof debt waived or a valid proof payload).
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Supported | Self::PartiallySupported)
    }
}

/// Evidence relation types between evidence and claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Mentions,
    Supports,
    PartiallySupports,
    Contradicts,
    SourceSpanAnchorOnly,
    RetrievedWith,
}

/// Method by which a support admission was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportAdmissionMethod {
    /// Operator explicitly admitted the support state.
    OperatorAdmitted,
    /// Admitted via a test fixture.
    TestFixtureAdmitted,
    /// Admitted via an external receipt.
    ExternalReceiptAdmitted,
}

/// Contradiction resolution outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionResolution {
    /// Contradiction confirmed — affected claims move to Contradicted state.
    Confirmed,
    /// Contradiction rejected — affected claims retain or revert to prior state.
    Rejected,
    /// Contradiction superseded by a newer resolved contradiction.
    Superseded,
}

/// Contradiction status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionStatus {
    /// Contradiction candidate detected, not yet reviewed.
    Candidate,
    /// Under active review.
    UnderReview,
    /// Review complete but no resolution reached yet.
    Unresolved,
    /// Resolution confirmed the contradiction.
    Confirmed,
    /// Resolution rejected the contradiction.
    Rejected,
    /// Superseded by another contradiction resolution.
    Superseded,
}

/// Types of proof debt — things that could weaken a support claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofDebt {
    /// Claim lacks a source basis in the input.
    MissingSourceBasis,
    /// Claim lacks a benchmark reference.
    MissingBenchmark,
    /// Claim lacks a reproduction case.
    MissingRepro,
    /// Claim lacks external validation.
    MissingExternalValidation,
    /// No proof debt.
    None,
}

/// A source artifact (document) that claims are extracted from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceArtifact {
    pub source_id: String,
    pub path: String,
    pub digest: String,
    pub kind: String,
    pub bytes_size: u64,
    pub loaded_at: Option<DateTime<Utc>>,
    pub degradation: Vec<String>,
}

/// A span within a source artifact (e.g., a paragraph or section).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub source_id: String,
    pub span_id: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub heading_path: Vec<String>,
    pub text: String,
    pub digest: String,
}

/// A source index mapping sources and spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIndex {
    pub sources: Vec<SourceArtifact>,
    pub spans: Vec<SourceSpan>,
}

/// An evidence link relating a claim to a source span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub relation: EvidenceRelation,
    pub source_id: String,
    pub span_id: String,
    pub quote: String,
    pub digest: String,
    pub support_role: String,
}

/// An evidence bundle attached to a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Unique identifier for this evidence bundle.
    pub evidence_bundle_id: String,
    /// The claim this bundle supports.
    pub claim_id: String,
    /// Evidence links within this bundle.
    pub evidence_links: Vec<EvidenceLink>,
    /// IDs of evidence that was considered but omitted.
    pub omitted_evidence_refs: Vec<String>,
    /// When this bundle was created.
    pub created_recorded_time: DateTime<Utc>,
}

impl EvidenceBundle {
    /// Create a new evidence bundle for a claim.
    pub fn new(claim_id: &str) -> Self {
        Self {
            evidence_bundle_id: ids::evidence_bundle_id(claim_id),
            claim_id: claim_id.to_string(),
            evidence_links: Vec::new(),
            omitted_evidence_refs: Vec::new(),
            created_recorded_time: Utc::now(),
        }
    }
}

/// A support judgment scoped to a claim via an evidence bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportJudgment {
    /// Unique identifier for this support judgment.
    pub support_judgment_id: String,
    /// The claim being judged.
    pub claim_id: String,
    /// The evidence bundle this judgment is scoped to.
    pub evidence_bundle_ref: String,
    /// The support state assigned.
    pub support_state: SupportState,
    /// How this judgment was determined.
    pub method: String,
    /// Human-readable rationale for this judgment.
    pub rationale: String,
    /// Contradiction IDs affecting this claim.
    pub contradiction_refs: Vec<String>,
    /// Proof debts attached to this judgment.
    pub proof_debt: Vec<ProofDebt>,
    /// When this judgment was created.
    pub created_recorded_time: DateTime<Utc>,
}

/// Proof payload attached to a support admission.
///
/// Must contain method-specific fields to be considered valid:
/// - Operator: `operator_ref`
/// - Test fixture: `fixture_ref`, `expected_assertion`, `fixture_digest`
/// - External receipt: `external_receipt_ref`, `external_receipt_digest`, `source_adapter`, `degradation_trust_note`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportProofPayload {
    /// Operator reference for operator-admitted support.
    pub operator_ref: Option<String>,
    /// Test fixture reference for fixture-admitted support.
    pub fixture_ref: Option<String>,
    /// Expected assertion for fixture-based support.
    pub expected_assertion: Option<String>,
    /// Digest of the fixture used.
    pub fixture_digest: Option<String>,
    /// External receipt reference.
    pub external_receipt_ref: Option<String>,
    /// Digest of the external receipt.
    pub external_receipt_digest: Option<String>,
    /// Source adapter name for external receipts.
    pub source_adapter: Option<String>,
    /// Trust note for degraded external receipts.
    pub degradation_trust_note: Option<String>,
}

impl Default for SupportProofPayload {
    fn default() -> Self {
        Self {
            operator_ref: None,
            fixture_ref: None,
            expected_assertion: None,
            fixture_digest: None,
            external_receipt_ref: None,
            external_receipt_digest: None,
            source_adapter: None,
            degradation_trust_note: None,
        }
    }
}

impl SupportProofPayload {
    /// Returns true if this payload has operator proof.
    pub fn has_operator_proof(&self) -> bool {
        self.operator_ref.is_some()
    }

    /// Returns true if this payload has fixture proof.
    pub fn has_fixture_proof(&self) -> bool {
        self.fixture_ref.is_some()
            && self.expected_assertion.is_some()
            && self.fixture_digest.is_some()
    }

    /// Returns true if this payload has external receipt proof.
    pub fn has_external_receipt_proof(&self) -> bool {
        self.external_receipt_ref.is_some()
            && self.external_receipt_digest.is_some()
            && self.source_adapter.is_some()
            && self.degradation_trust_note.is_some()
    }
}

/// An admission record for a support state upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportAdmission {
    /// The claim being admitted.
    pub claim_id: String,
    /// The previous support judgment (superseded).
    pub previous_support_judgment_ref: String,
    /// The new support judgment (superseding).
    pub new_support_judgment_ref: String,
    /// The admission method used.
    pub method: SupportAdmissionMethod,
    /// The support state admitted.
    pub admitted_support_state: SupportState,
    /// Rationale for the admission.
    pub rationale: String,
    /// Operator reference (if operator-admitted).
    pub operator_ref: Option<String>,
    /// Optional proof payload.
    pub proof_payload: Option<SupportProofPayload>,
    /// Optional explicit proof debt waiver.
    pub proof_debt_waiver: Option<String>,
    /// When this admission was recorded.
    pub recorded_time: DateTime<Utc>,
}

/// A contradiction between two or more claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionRecord {
    /// Unique identifier for this contradiction.
    pub contradiction_id: String,
    /// IDs of claims involved in this contradiction.
    pub claim_refs: Vec<String>,
    /// Support judgment IDs involved.
    pub support_judgment_refs: Vec<String>,
    /// Source span IDs involved.
    pub source_span_refs: Vec<String>,
    /// The pattern/kind of contradiction detected.
    pub pattern: String,
    /// Rationale for why this is a contradiction.
    pub rationale: String,
    /// Current lifecycle status.
    pub status: ContradictionStatus,
    /// When this record was created.
    pub created_recorded_time: DateTime<Utc>,
}

impl ContradictionRecord {
    /// Create a new contradiction record between two claims.
    pub fn new(left_claim_id: &str, right_claim_id: &str, pattern: &str, rationale: &str) -> Self {
        let claim_refs = if left_claim_id <= right_claim_id {
            vec![left_claim_id.to_string(), right_claim_id.to_string()]
        } else {
            vec![right_claim_id.to_string(), left_claim_id.to_string()]
        };
        Self {
            contradiction_id: ids::contradiction_id(left_claim_id, right_claim_id, pattern),
            claim_refs,
            support_judgment_refs: Vec::new(),
            source_span_refs: Vec::new(),
            pattern: pattern.to_string(),
            rationale: rationale.to_string(),
            status: ContradictionStatus::Candidate,
            created_recorded_time: Utc::now(),
        }
    }
}

/// Resolution record for a contradiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionResolutionRecord {
    /// The contradiction being resolved.
    pub contradiction_id: String,
    /// Status before resolution.
    pub previous_status: ContradictionStatus,
    /// The resolution outcome.
    pub resolution: ContradictionResolution,
    /// Rationale for the resolution.
    pub rationale: String,
    /// Support judgment IDs affected by this resolution.
    pub affected_support_judgment_refs: Vec<String>,
    /// Supersession receipt IDs created.
    pub supersession_receipt_refs: Vec<String>,
    /// Review queue item IDs affected.
    pub review_queue_refs: Vec<String>,
    /// When this resolution was recorded.
    pub recorded_time: DateTime<Utc>,
}

/// A supersession record linking a previous ID to a new ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supersession {
    /// The ID that was superseded.
    pub superseded_ref: String,
    /// The ID that superseded it.
    pub superseding_ref: String,
    /// Rationale for the supersession.
    pub rationale: String,
    /// When this supersession was recorded.
    pub recorded_time: DateTime<Utc>,
}

/// A claim extracted from source material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// Unique identifier for this claim.
    pub claim_id: String,
    /// Source this claim was extracted from.
    pub source_id: String,
    /// Span within the source this claim was extracted from.
    pub span_id: String,
    /// The raw claim text.
    pub claim_text: String,
    /// Normalized form of the claim text.
    pub normalized_claim: String,
    /// Kind of claim (e.g., "fact", "suggestion", "warning").
    pub claim_type: String,
    /// Processing status.
    pub status: String,
    /// Current support judgment ID for this claim.
    pub support_judgment_ref: String,
    /// Evidence bundle ID for this claim.
    pub evidence_bundle_ref: String,
    /// Proof debts attached to this claim.
    pub proof_debt: Vec<ProofDebt>,
    /// Recommended next action.
    pub recommended_action: String,
    /// Rationale for this claim.
    pub rationale: String,
    /// Confidence score 0.0–1.0.
    pub confidence: f64,
    /// When this claim was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Additional metadata.
    pub metadata: serde_json::Value,
}

impl Claim {
    /// Create a new claim from source and span.
    pub fn new(source_id: &str, span_id: &str, claim_text: &str, claim_type: &str) -> Self {
        let normalized = ids::normalize_text(claim_text);
        Self {
            claim_id: ids::claim_id(source_id, span_id, &normalized),
            source_id: source_id.to_string(),
            span_id: span_id.to_string(),
            claim_text: claim_text.to_string(),
            normalized_claim: normalized,
            claim_type: claim_type.to_string(),
            status: "pending".to_string(),
            support_judgment_ref: String::new(),
            evidence_bundle_ref: String::new(),
            proof_debt: vec![ProofDebt::MissingSourceBasis],
            recommended_action: "await_support_judgment".to_string(),
            rationale: String::new(),
            confidence: 0.0,
            created_at: None,
            metadata: serde_json::Value::Object(Default::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_state_is_strong() {
        assert!(SupportState::Supported.is_strong());
        assert!(SupportState::PartiallySupported.is_strong());
        assert!(!SupportState::Unsupported.is_strong());
        assert!(!SupportState::Contradicted.is_strong());
        assert!(!SupportState::HeuristicOnly.is_strong());
        assert!(!SupportState::Unknown.is_strong());
    }

    #[test]
    fn evidence_bundle_new_has_id_prefix() {
        let bundle = EvidenceBundle::new("clm_abc123");
        assert!(bundle.evidence_bundle_id.starts_with("evb_"));
        assert_eq!(bundle.claim_id, "clm_abc123");
    }

    #[test]
    fn contradiction_record_new_is_ordered() {
        let c1 = ContradictionRecord::new("clm_a", "clm_b", "exact", "texts contradict");
        let c2 = ContradictionRecord::new("clm_b", "clm_a", "exact", "texts contradict");
        // Ordering ensures same result regardless of argument order
        assert_eq!(c1.contradiction_id, c2.contradiction_id);
        assert_eq!(c1.claim_refs, c2.claim_refs);
    }

    #[test]
    fn support_proof_payload_has_methods() {
        let operator = SupportProofPayload {
            operator_ref: Some("op-123".to_string()),
            ..Default::default()
        };
        assert!(operator.has_operator_proof());

        let fixture = SupportProofPayload {
            fixture_ref: Some("fix-456".to_string()),
            expected_assertion: Some("claim is true".to_string()),
            fixture_digest: Some("abc123".to_string()),
            ..Default::default()
        };
        assert!(fixture.has_fixture_proof());
    }

    #[test]
    fn claim_new_has_deterministic_id() {
        let c1 = Claim::new("src1", "sp1", "The sky is blue", "fact");
        let c2 = Claim::new("src1", "sp1", "The sky is blue", "fact");
        assert_eq!(c1.claim_id, c2.claim_id);
    }
}
