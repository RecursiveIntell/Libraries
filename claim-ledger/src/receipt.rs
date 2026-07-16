//! Receipt types for claim ledger material operations.
//!
//! Every material operation (bundle export, support admission, contradiction resolution,
//! ledger append) produces a receipt that captures:
//! - The inputs and their digests
//! - The outputs and their digests
//! - The operation metadata
//!
//! This enables deterministic verification of any claim ledger artifact.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids;
use crate::types::{ContradictionResolution, SupportAdmissionMethod, SupportState};

/// A binding receipt for any export operation.
///
/// Captures inputs, outputs, and digests so the output artifact
/// can be independently verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReceipt {
    /// Schema version identifier.
    pub receipt_version: String,
    /// Unique receipt identifier.
    pub export_receipt_id: String,
    /// The operation performed (e.g., "bundle_export", "testify_answer").
    pub operation: String,
    /// Paths/IDs of input artifacts.
    pub input_refs: Vec<String>,
    /// Digest for each input (ref -> digest).
    pub input_digests: std::collections::BTreeMap<String, String>,
    /// Path/ID of the output artifact.
    pub output_ref: Option<String>,
    /// Digest of the output artifact (computed after writing).
    pub output_digest: Option<String>,
    /// When this receipt was recorded.
    pub recorded_time: DateTime<Utc>,
    /// Attempt identifier for traceability.
    pub attempt_id: String,
    /// Final status of the operation.
    pub status: String,
    /// Degradation items encountered during the operation.
    pub degradation: Vec<String>,
    /// Semantics of the digest (e.g., "sha256-output-bytes").
    pub digest_semantics: String,
}

impl ExportReceipt {
    /// Create a new export receipt.
    pub fn new(operation: &str, input_refs: Vec<String>, attempt_id: String) -> Self {
        Self::new_at(operation, input_refs, attempt_id, Utc::now())
    }

    /// Create a receipt with an explicit recorded time for reproducible artifacts.
    pub fn new_at(
        operation: &str,
        input_refs: Vec<String>,
        attempt_id: String,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        let input_refs_str: Vec<&str> = input_refs.iter().map(|s| s.as_str()).collect();
        let export_receipt_id = ids::export_receipt_id(operation, &input_refs_str);
        Self {
            receipt_version: "ExportReceiptV1".to_string(),
            export_receipt_id,
            operation: operation.to_string(),
            input_refs,
            input_digests: std::collections::BTreeMap::new(),
            output_ref: None,
            output_digest: None,
            recorded_time,
            attempt_id,
            status: "pending".to_string(),
            degradation: Vec::new(),
            digest_semantics: "sha256-output-bytes".to_string(),
        }
    }

    /// Bind the output reference and digest to this receipt.
    pub fn bind_output(&mut self, output_ref: String, output_digest: String) {
        self.output_ref = Some(output_ref);
        self.output_digest = Some(output_digest);
    }

    /// Mark this receipt as successful.
    pub fn mark_success(&mut self) {
        self.status = "success".to_string();
    }

    /// Mark this receipt as failed.
    pub fn mark_failed(&mut self, reason: &str) {
        self.status = format!("failed: {}", reason);
    }

    /// Add a degradation item.
    pub fn add_degradation(&mut self, item: &str) {
        self.degradation.push(item.to_string());
    }
}

/// Receipt for a support admission operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportAdmissionReceipt {
    /// Schema version identifier.
    pub receipt_version: String,
    /// Unique receipt identifier.
    pub support_admission_receipt_id: String,
    /// The claim being admitted.
    pub claim_id: String,
    /// The support judgment being superseded.
    pub previous_support_judgment_ref: String,
    /// The new support judgment.
    pub new_support_judgment_ref: String,
    /// The admission method used.
    pub method: SupportAdmissionMethod,
    /// The support state admitted.
    pub admitted_support_state: SupportState,
    /// Rationale for the admission.
    pub rationale: String,
    /// Operator reference (if applicable).
    pub operator_ref: Option<String>,
    /// Proof payload (if applicable).
    pub proof_payload: Option<crate::types::SupportProofPayload>,
    /// Explicit proof debt waiver (if any).
    pub proof_debt_waiver: Option<String>,
    /// When this receipt was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl SupportAdmissionReceipt {
    /// Create a new support admission receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        claim_id: &str,
        previous_ref: &str,
        new_ref: &str,
        method: SupportAdmissionMethod,
        admitted_state: SupportState,
        rationale: &str,
    ) -> Self {
        Self::new_at(
            claim_id,
            previous_ref,
            new_ref,
            method,
            admitted_state,
            rationale,
            Utc::now(),
        )
    }

    /// Create a receipt with an explicit recorded time for reproducible artifacts.
    #[allow(clippy::too_many_arguments)]
    pub fn new_at(
        claim_id: &str,
        previous_ref: &str,
        new_ref: &str,
        method: SupportAdmissionMethod,
        admitted_state: SupportState,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            receipt_version: "SupportAdmissionReceiptV1".to_string(),
            support_admission_receipt_id: ids::support_admission_receipt_id(
                claim_id,
                previous_ref,
                new_ref,
            ),
            claim_id: claim_id.to_string(),
            previous_support_judgment_ref: previous_ref.to_string(),
            new_support_judgment_ref: new_ref.to_string(),
            method,
            admitted_support_state: admitted_state,
            rationale: rationale.to_string(),
            operator_ref: None,
            proof_payload: None,
            proof_debt_waiver: None,
            recorded_time,
        }
    }
}

/// Receipt for a ledger append operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerAppendReceipt {
    /// Schema version identifier.
    pub receipt_version: String,
    /// Unique receipt identifier.
    pub ledger_append_receipt_id: String,
    /// Path to the ledger file.
    pub ledger_ref: String,
    /// Sequence number of this entry.
    pub sequence: u64,
    /// Digest of the previous entry (None for first entry).
    pub previous_entry_digest: Option<String>,
    /// Digest of this entry.
    pub entry_digest: String,
    /// When this receipt was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl LedgerAppendReceipt {
    /// Create a new ledger append receipt.
    pub fn new(
        ledger_ref: &str,
        sequence: u64,
        previous_entry_digest: Option<String>,
        entry_digest: String,
    ) -> Self {
        Self::new_at(
            ledger_ref,
            sequence,
            previous_entry_digest,
            entry_digest,
            Utc::now(),
        )
    }

    /// Create a receipt with an explicit recorded time for reproducible artifacts.
    pub fn new_at(
        ledger_ref: &str,
        sequence: u64,
        previous_entry_digest: Option<String>,
        entry_digest: String,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            receipt_version: "LedgerAppendReceiptV1".to_string(),
            ledger_append_receipt_id: ids::ledger_append_receipt_id(
                ledger_ref,
                sequence,
                &entry_digest,
            ),
            ledger_ref: ledger_ref.to_string(),
            sequence,
            previous_entry_digest,
            entry_digest,
            recorded_time,
        }
    }
}

/// Receipt for a contradiction resolution operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionResolutionReceipt {
    /// Schema version identifier.
    pub receipt_version: String,
    /// Unique receipt identifier.
    pub contradiction_resolution_receipt_id: String,
    /// The contradiction being resolved.
    pub contradiction_id: String,
    /// Status before resolution.
    pub previous_status: String,
    /// The resolution outcome.
    pub resolution: ContradictionResolution,
    /// Rationale for the resolution.
    pub rationale: String,
    /// Support judgment IDs affected.
    pub affected_support_judgment_refs: Vec<String>,
    /// Supersession receipt IDs created.
    pub supersession_receipt_refs: Vec<String>,
    /// Review queue item IDs affected.
    pub review_queue_refs: Vec<String>,
    /// When this receipt was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl ContradictionResolutionReceipt {
    /// Create a new contradiction resolution receipt.
    pub fn new(
        contradiction_id: &str,
        previous_status: &str,
        resolution: ContradictionResolution,
        rationale: &str,
    ) -> Self {
        Self::new_at(
            contradiction_id,
            previous_status,
            resolution,
            rationale,
            Utc::now(),
        )
    }

    /// Create a receipt with an explicit recorded time for reproducible artifacts.
    pub fn new_at(
        contradiction_id: &str,
        previous_status: &str,
        resolution: ContradictionResolution,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            receipt_version: "ContradictionResolutionReceiptV1".to_string(),
            contradiction_resolution_receipt_id: ids::contradiction_resolution_receipt_id(
                contradiction_id,
                previous_status,
                &format!("{:?}", resolution),
            ),
            contradiction_id: contradiction_id.to_string(),
            previous_status: previous_status.to_string(),
            resolution,
            rationale: rationale.to_string(),
            affected_support_judgment_refs: Vec::new(),
            supersession_receipt_refs: Vec::new(),
            review_queue_refs: Vec::new(),
            recorded_time,
        }
    }
}

/// Receipt for a supersession event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersessionReceipt {
    /// Schema version identifier.
    pub receipt_version: String,
    /// Unique receipt identifier.
    pub supersession_receipt_id: String,
    /// The ID that was superseded.
    pub superseded_ref: String,
    /// The ID that superseded it.
    pub superseding_ref: String,
    /// Rationale for the supersession.
    pub rationale: String,
    /// When this receipt was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl SupersessionReceipt {
    /// Create a new supersession receipt.
    pub fn new(superseded_ref: &str, superseding_ref: &str, rationale: &str) -> Self {
        Self::new_at(superseded_ref, superseding_ref, rationale, Utc::now())
    }

    /// Create a receipt with an explicit recorded time for reproducible artifacts.
    pub fn new_at(
        superseded_ref: &str,
        superseding_ref: &str,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            receipt_version: "SupersessionReceiptV1".to_string(),
            supersession_receipt_id: ids::supersession_receipt_id(superseded_ref, superseding_ref),
            superseded_ref: superseded_ref.to_string(),
            superseding_ref: superseding_ref.to_string(),
            rationale: rationale.to_string(),
            recorded_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_receipt_new_has_id() {
        let receipt = ExportReceipt::new(
            "bundle_export",
            vec!["/path/to/bundle.json".to_string()],
            "attempt-001".to_string(),
        );
        assert!(receipt.export_receipt_id.starts_with("xpt_"));
        assert_eq!(receipt.operation, "bundle_export");
        assert_eq!(receipt.status, "pending");
    }

    #[test]
    fn export_receipt_bind_output() {
        let mut receipt = ExportReceipt::new(
            "testify_answer",
            vec!["in.json".to_string()],
            "attempt-001".to_string(),
        );
        receipt.bind_output("/path/to/out.json".to_string(), "abc123".to_string());
        assert_eq!(receipt.output_ref, Some("/path/to/out.json".to_string()));
        assert_eq!(receipt.output_digest, Some("abc123".to_string()));
    }

    #[test]
    fn ledger_append_receipt_sequence() {
        let receipt =
            LedgerAppendReceipt::new("/ledger.jsonl", 1, None, "entryhash123".to_string());
        assert_eq!(receipt.sequence, 1);
        assert!(receipt.previous_entry_digest.is_none());
    }

    #[test]
    fn supersession_receipt_new() {
        let receipt = SupersessionReceipt::new("old-id", "new-id", "superseded by newer judgment");
        assert!(receipt.supersession_receipt_id.starts_with("ssr_"));
        assert_eq!(receipt.superseded_ref, "old-id");
        assert_eq!(receipt.superseding_ref, "new-id");
    }
}
