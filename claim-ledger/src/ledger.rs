//! Append-only hash-chained ledger for claim events.
//!
//! The ledger is a newline-delimited JSON (JSONL) file where each entry contains:
//! - A sequence number
//! - A hash of the previous entry (for chain integrity)
//! - The event payload
//! - A hash of the entry itself
//!
//! This structure allows the ledger to be verified by checking the hash chain
//! and makes it possible to detect tampering or corruption.

use serde::{Deserialize, Serialize};

use super::types::SupportState;
use crate::ids::sha256_text;

/// An entry in the claim ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Sequence number (1-indexed).
    pub sequence: u64,
    /// Digest of the previous entry (None for first entry).
    pub previous_entry_digest: Option<String>,
    /// The event payload.
    pub event: LedgerEvent,
    /// Digest of this entry (computed from sequence + previous + event).
    pub entry_digest: String,
}

/// Events that can be appended to the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LedgerEvent {
    /// A claim was added to the bundle.
    ClaimAdded {
        claim_id: String,
        source_id: String,
        span_id: String,
        normalized_claim: String,
    },
    /// A support judgment was assigned to a claim.
    SupportJudgment {
        support_judgment_id: String,
        claim_id: String,
        evidence_bundle_ref: String,
        support_state: SupportState,
        method: String,
    },
    /// A support admission was recorded.
    SupportAdmission {
        support_admission_receipt_id: String,
        claim_id: String,
        previous_support_judgment_ref: String,
        new_support_judgment_ref: String,
        admitted_support_state: SupportState,
    },
    /// A contradiction candidate was recorded.
    ContradictionCandidate {
        contradiction_id: String,
        claim_refs: Vec<String>,
        pattern: String,
        rationale: String,
    },
    /// A contradiction was resolved.
    ContradictionResolved {
        contradiction_resolution_receipt_id: String,
        contradiction_id: String,
        resolution: String,
        affected_claim_refs: Vec<String>,
    },
    /// An evidence bundle was attached to a claim.
    EvidenceAttached {
        evidence_bundle_id: String,
        claim_id: String,
        evidence_link_count: usize,
    },
    /// A bundle was exported.
    BundleExported {
        bundle_id: String,
        export_receipt_id: String,
        output_ref: String,
        output_digest: String,
    },
    /// Proof-debt budget was consumed.
    ProofDebtConsumed {
        budget_id: String,
        debit_id: String,
        amount_micros: u64,
        source: String,
        overdrawn: bool,
    },
    /// Proof-debt budget was replenished (evidence, admission, or waiver).
    ProofDebtReplenished {
        budget_id: String,
        credit_id: String,
        amount_micros: u64,
        source: String,
    },
}

impl LedgerEvent {
    /// Get the event type name as a string.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::ClaimAdded { .. } => "claim_added",
            Self::SupportJudgment { .. } => "support_judgment",
            Self::SupportAdmission { .. } => "support_admission",
            Self::ContradictionCandidate { .. } => "contradiction_candidate",
            Self::ContradictionResolved { .. } => "contradiction_resolved",
            Self::EvidenceAttached { .. } => "evidence_attached",
            Self::BundleExported { .. } => "bundle_exported",
            Self::ProofDebtConsumed { .. } => "proof_debt_consumed",
            Self::ProofDebtReplenished { .. } => "proof_debt_replenished",
        }
    }
}

/// Result of verifying a ledger.
#[derive(Debug, Clone)]
pub struct LedgerVerification {
    /// Whether the ledger is valid.
    pub valid: bool,
    /// Sequence of the last valid entry.
    pub last_sequence: u64,
    /// Last entry digest in the chain.
    pub last_entry_digest: Option<String>,
    /// Errors encountered during verification.
    pub errors: Vec<String>,
}

impl Default for LedgerVerification {
    fn default() -> Self {
        Self {
            valid: true,
            last_sequence: 0,
            last_entry_digest: None,
            errors: Vec::new(),
        }
    }
}

/// A builder for ledger entries with hash chain support.
pub struct LedgerEntryBuilder {
    sequence: u64,
    previous_entry_digest: Option<String>,
}

impl LedgerEntryBuilder {
    /// Create a new builder for the next entry.
    pub fn new(sequence: u64, previous_entry_digest: Option<String>) -> Self {
        Self {
            sequence,
            previous_entry_digest,
        }
    }

    /// Append a claim to the ledger.
    pub fn add_claim(
        self,
        claim_id: &str,
        source_id: &str,
        span_id: &str,
        normalized_claim: &str,
    ) -> LedgerEntry {
        let event = LedgerEvent::ClaimAdded {
            claim_id: claim_id.to_string(),
            source_id: source_id.to_string(),
            span_id: span_id.to_string(),
            normalized_claim: normalized_claim.to_string(),
        };
        self._build_entry(event)
    }

    /// Add a support judgment event.
    pub fn add_support_judgment(
        self,
        support_judgment_id: &str,
        claim_id: &str,
        evidence_bundle_ref: &str,
        support_state: SupportState,
        method: &str,
    ) -> LedgerEntry {
        let event = LedgerEvent::SupportJudgment {
            support_judgment_id: support_judgment_id.to_string(),
            claim_id: claim_id.to_string(),
            evidence_bundle_ref: evidence_bundle_ref.to_string(),
            support_state,
            method: method.to_string(),
        };
        self._build_entry(event)
    }

    /// Add a support admission event.
    pub fn add_support_admission(
        self,
        support_admission_receipt_id: &str,
        claim_id: &str,
        previous_ref: &str,
        new_ref: &str,
        admitted_state: SupportState,
    ) -> LedgerEntry {
        let event = LedgerEvent::SupportAdmission {
            support_admission_receipt_id: support_admission_receipt_id.to_string(),
            claim_id: claim_id.to_string(),
            previous_support_judgment_ref: previous_ref.to_string(),
            new_support_judgment_ref: new_ref.to_string(),
            admitted_support_state: admitted_state,
        };
        self._build_entry(event)
    }

    /// Add a contradiction candidate event.
    pub fn add_contradiction_candidate(
        self,
        contradiction_id: &str,
        claim_refs: Vec<String>,
        pattern: &str,
        rationale: &str,
    ) -> LedgerEntry {
        let event = LedgerEvent::ContradictionCandidate {
            contradiction_id: contradiction_id.to_string(),
            claim_refs,
            pattern: pattern.to_string(),
            rationale: rationale.to_string(),
        };
        self._build_entry(event)
    }

    /// Add a contradiction resolution event.
    pub fn add_contradiction_resolved(
        self,
        receipt_id: &str,
        contradiction_id: &str,
        resolution: &str,
        affected_claim_refs: Vec<String>,
    ) -> LedgerEntry {
        let event = LedgerEvent::ContradictionResolved {
            contradiction_resolution_receipt_id: receipt_id.to_string(),
            contradiction_id: contradiction_id.to_string(),
            resolution: resolution.to_string(),
            affected_claim_refs,
        };
        self._build_entry(event)
    }

    /// Add a proof-debt consumption event.
    pub fn add_proof_debt_consumed(
        self,
        budget_id: &str,
        debit_id: &str,
        amount_micros: u64,
        source: &str,
        overdrawn: bool,
    ) -> LedgerEntry {
        let event = LedgerEvent::ProofDebtConsumed {
            budget_id: budget_id.to_string(),
            debit_id: debit_id.to_string(),
            amount_micros,
            source: source.to_string(),
            overdrawn,
        };
        self._build_entry(event)
    }

    /// Add a proof-debt replenishment event.
    pub fn add_proof_debt_replenished(
        self,
        budget_id: &str,
        credit_id: &str,
        amount_micros: u64,
        source: &str,
    ) -> LedgerEntry {
        let event = LedgerEvent::ProofDebtReplenished {
            budget_id: budget_id.to_string(),
            credit_id: credit_id.to_string(),
            amount_micros,
            source: source.to_string(),
        };
        self._build_entry(event)
    }

    fn _build_entry(self, event: LedgerEvent) -> LedgerEntry {
        let entry_content = serde_json::json!({
            "sequence": self.sequence,
            "previous_entry_digest": self.previous_entry_digest,
            "event": event,
        });

        let entry_json = serde_json::to_string(&entry_content).unwrap_or_default();
        let digest = sha256_text(&entry_json);

        LedgerEntry {
            sequence: self.sequence,
            previous_entry_digest: self.previous_entry_digest,
            event,
            entry_digest: digest,
        }
    }
}

/// Compute the digest for an entry.
pub fn compute_entry_digest(
    sequence: u64,
    previous_digest: Option<&str>,
    event: &LedgerEvent,
) -> String {
    let content = serde_json::json!({
        "sequence": sequence,
        "previous_entry_digest": previous_digest,
        "event": event,
    });
    let full_json = serde_json::to_string(&content).unwrap_or_default();
    sha256_text(&full_json)
}

/// Verify the integrity of a ledger from a list of entries.
pub fn verify_ledger(entries: &[LedgerEntry]) -> LedgerVerification {
    let mut verification = LedgerVerification::default();

    for (i, entry) in entries.iter().enumerate() {
        let expected_seq = (i + 1) as u64;
        if entry.sequence != expected_seq {
            verification.valid = false;
            verification.errors.push(format!(
                "sequence mismatch at index {}: expected {}, got {}",
                i, expected_seq, entry.sequence
            ));
        }

        if i == 0 {
            if entry.previous_entry_digest.is_some() {
                verification.valid = false;
                verification
                    .errors
                    .push("first entry should have no previous_entry_digest".to_string());
            }
        } else {
            let prev = &entries[i - 1];
            if entry.previous_entry_digest.as_ref() != Some(&prev.entry_digest) {
                verification.valid = false;
                verification.errors.push(format!(
                    "previous_entry_digest mismatch at sequence {}: expected {}, got {:?}",
                    entry.sequence, prev.entry_digest, entry.previous_entry_digest
                ));
            }
        }

        let computed = compute_entry_digest(
            entry.sequence,
            entry.previous_entry_digest.as_deref(),
            &entry.event,
        );
        if computed != entry.entry_digest {
            verification.valid = false;
            verification.errors.push(format!(
                "entry_digest mismatch at sequence {}: expected {}, got {}",
                entry.sequence, computed, entry.entry_digest
            ));
        }

        verification.last_sequence = entry.sequence;
        verification.last_entry_digest = Some(entry.entry_digest.clone());
    }

    verification
}

/// Parse ledger entries from JSONL text.
pub fn parse_ledger_entries(jsonl: &str) -> Vec<LedgerEntry> {
    jsonl
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// Serialize a ledger entry to a JSON line.
pub fn serialize_entry(entry: &LedgerEntry) -> String {
    serde_json::to_string(entry).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_entry_builder_add_claim() {
        let entry = LedgerEntryBuilder::new(1, None).add_claim(
            "clm_abc123",
            "src1",
            "sp1",
            "the sky is blue",
        );

        assert_eq!(entry.sequence, 1);
        assert!(entry.previous_entry_digest.is_none());
        assert!(!entry.entry_digest.is_empty());
        if let LedgerEvent::ClaimAdded { claim_id, .. } = entry.event {
            assert_eq!(claim_id, "clm_abc123");
        } else {
            panic!("expected ClaimAdded event");
        }
    }

    #[test]
    fn ledger_chain_integrity() {
        let entry1 = LedgerEntryBuilder::new(1, None).add_claim("clm_a", "src1", "sp1", "claim a");
        let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
            .add_claim("clm_b", "src1", "sp2", "claim b");

        assert_eq!(entry2.sequence, 2);
        assert_eq!(
            entry2.previous_entry_digest,
            Some(entry1.entry_digest.clone())
        );
    }

    #[test]
    fn verify_ledger_accepts_valid_chain() {
        let entry1 = LedgerEntryBuilder::new(1, None).add_claim("clm_a", "src1", "sp1", "claim a");
        let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
            .add_claim("clm_b", "src1", "sp2", "claim b");

        let verification = verify_ledger(&[entry1, entry2]);
        assert!(verification.valid);
        assert_eq!(verification.last_sequence, 2);
    }

    #[test]
    fn verify_ledger_detects_broken_chain() {
        let entry1 = LedgerEntryBuilder::new(1, None).add_claim("clm_a", "src1", "sp1", "claim a");
        // entry2 with wrong previous digest
        let entry2 = LedgerEntryBuilder::new(2, Some("wrong_digest".to_string()))
            .add_claim("clm_b", "src1", "sp2", "claim b");

        let verification = verify_ledger(&[entry1, entry2]);
        assert!(!verification.valid);
        assert!(!verification.errors.is_empty());
    }

    #[test]
    fn parse_ledger_entries_from_jsonl() {
        let jsonl = r#"{"sequence":1,"previous_entry_digest":null,"event":{"type":"claim_added","claim_id":"clm_1","source_id":"s1","span_id":"sp1","normalized_claim":"test"},"entry_digest":"abc"}
{"sequence":2,"previous_entry_digest":"abc","event":{"type":"claim_added","claim_id":"clm_2","source_id":"s1","span_id":"sp2","normalized_claim":"test2"},"entry_digest":"def"}"#;

        let entries = parse_ledger_entries(jsonl);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 1);
        assert_eq!(entries[1].sequence, 2);
    }

    #[test]
    fn serialize_entry_round_trips() {
        let entry =
            LedgerEntryBuilder::new(1, None).add_claim("clm_test", "src1", "sp1", "hello world");

        let json = serialize_entry(&entry);
        let parsed: LedgerEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sequence, entry.sequence);
        assert_eq!(parsed.entry_digest, entry.entry_digest);
    }

    #[test]
    fn ledger_event_type_name() {
        let event = LedgerEvent::ClaimAdded {
            claim_id: "clm_1".to_string(),
            source_id: "s1".to_string(),
            span_id: "sp1".to_string(),
            normalized_claim: "test".to_string(),
        };
        assert_eq!(event.type_name(), "claim_added");
    }
}
