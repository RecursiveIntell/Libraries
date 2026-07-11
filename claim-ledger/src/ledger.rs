//! Append-only, hash-chained ledger for claim events.
//!
//! ## Canonical digest preimage (v1)
//!
//! Entry digests are SHA-256 over a binary, language-independent preimage. It
//! is deliberately not JSON, so it does not depend on map ordering or a JSON
//! serializer. A verifier writes the UTF-8 bytes of
//! `claim-ledger.entry-digest.v1`, then the sequence as an unsigned 64-bit
//! big-endian integer, then the previous digest as `0x00` for absent or `0x01`
//! followed by its byte-length-prefixed UTF-8 value. It then writes the event
//! tag as a byte-length-prefixed UTF-8 value and each event field in declaration
//! order. Strings are UTF-8 prefixed with their unsigned 64-bit big-endian byte
//! length; integer fields are unsigned 64-bit big-endian; booleans are one byte
//! (`0x00` or `0x01`); vectors are an unsigned 64-bit count followed by encoded
//! strings. Recorded timestamps and `entry_digest` are excluded.

use serde::{Deserialize, Serialize};

use super::types::SupportState;
use crate::{error::ClaimLedgerError, ids::sha256_bytes};

/// An entry in the claim ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Sequence number (1-indexed).
    pub sequence: u64,
    /// Digest of the previous entry (None for first entry).
    pub previous_entry_digest: Option<String>,
    /// The event payload.
    pub event: LedgerEvent,
    /// SHA-256 digest of the canonical entry preimage.
    pub entry_digest: String,
}

/// Events that can be appended to the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LedgerEvent {
    ClaimAdded {
        claim_id: String,
        source_id: String,
        span_id: String,
        normalized_claim: String,
    },
    SupportJudgment {
        support_judgment_id: String,
        claim_id: String,
        evidence_bundle_ref: String,
        support_state: SupportState,
        method: String,
    },
    SupportAdmission {
        support_admission_receipt_id: String,
        claim_id: String,
        previous_support_judgment_ref: String,
        new_support_judgment_ref: String,
        admitted_support_state: SupportState,
    },
    ContradictionCandidate {
        contradiction_id: String,
        claim_refs: Vec<String>,
        pattern: String,
        rationale: String,
    },
    ContradictionResolved {
        contradiction_resolution_receipt_id: String,
        contradiction_id: String,
        resolution: String,
        affected_claim_refs: Vec<String>,
    },
    EvidenceAttached {
        evidence_bundle_id: String,
        claim_id: String,
        evidence_link_count: usize,
    },
    BundleExported {
        bundle_id: String,
        export_receipt_id: String,
        output_ref: String,
        output_digest: String,
    },
    ProofDebtConsumed {
        budget_id: String,
        debit_id: String,
        amount_micros: u64,
        source: String,
        overdrawn: bool,
    },
    ProofDebtReplenished {
        budget_id: String,
        credit_id: String,
        amount_micros: u64,
        source: String,
    },
}

impl LedgerEvent {
    /// Get the stable event type name used in the digest preimage.
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

/// The head a verifier expects to authenticate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedLedgerHead {
    /// An intentionally empty ledger; no entries may be present.
    Empty,
    /// A non-empty ledger whose final sequence and digest must match exactly.
    Entry { sequence: u64, entry_digest: String },
}

impl ExpectedLedgerHead {
    /// Construct the expected head for an empty ledger.
    pub const fn empty() -> Self {
        Self::Empty
    }

    /// Construct the expected head for a non-empty ledger.
    pub fn new(sequence: u64, entry_digest: impl Into<String>) -> Self {
        Self::Entry {
            sequence,
            entry_digest: entry_digest.into(),
        }
    }
}

/// Successful verification details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerVerification {
    /// Sequence of the authenticated final entry, or zero when empty.
    pub last_sequence: u64,
    /// Digest of the authenticated final entry, if any.
    pub last_entry_digest: Option<String>,
}

/// A builder for ledger entries with hash-chain support.
pub struct LedgerEntryBuilder {
    sequence: u64,
    previous_entry_digest: Option<String>,
}

impl LedgerEntryBuilder {
    /// Create a builder for the next entry.
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
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::ClaimAdded {
            claim_id: claim_id.into(),
            source_id: source_id.into(),
            span_id: span_id.into(),
            normalized_claim: normalized_claim.into(),
        })
    }

    /// Add a support judgment event.
    pub fn add_support_judgment(
        self,
        support_judgment_id: &str,
        claim_id: &str,
        evidence_bundle_ref: &str,
        support_state: SupportState,
        method: &str,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::SupportJudgment {
            support_judgment_id: support_judgment_id.into(),
            claim_id: claim_id.into(),
            evidence_bundle_ref: evidence_bundle_ref.into(),
            support_state,
            method: method.into(),
        })
    }

    /// Add a support admission event.
    pub fn add_support_admission(
        self,
        receipt_id: &str,
        claim_id: &str,
        previous_ref: &str,
        new_ref: &str,
        admitted_state: SupportState,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::SupportAdmission {
            support_admission_receipt_id: receipt_id.into(),
            claim_id: claim_id.into(),
            previous_support_judgment_ref: previous_ref.into(),
            new_support_judgment_ref: new_ref.into(),
            admitted_support_state: admitted_state,
        })
    }

    /// Add a contradiction candidate event.
    pub fn add_contradiction_candidate(
        self,
        contradiction_id: &str,
        claim_refs: Vec<String>,
        pattern: &str,
        rationale: &str,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::ContradictionCandidate {
            contradiction_id: contradiction_id.into(),
            claim_refs,
            pattern: pattern.into(),
            rationale: rationale.into(),
        })
    }

    /// Add a contradiction resolution event.
    pub fn add_contradiction_resolved(
        self,
        receipt_id: &str,
        contradiction_id: &str,
        resolution: &str,
        affected_claim_refs: Vec<String>,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::ContradictionResolved {
            contradiction_resolution_receipt_id: receipt_id.into(),
            contradiction_id: contradiction_id.into(),
            resolution: resolution.into(),
            affected_claim_refs,
        })
    }

    /// Add a proof-debt consumption event.
    pub fn add_proof_debt_consumed(
        self,
        budget_id: &str,
        debit_id: &str,
        amount_micros: u64,
        source: &str,
        overdrawn: bool,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::ProofDebtConsumed {
            budget_id: budget_id.into(),
            debit_id: debit_id.into(),
            amount_micros,
            source: source.into(),
            overdrawn,
        })
    }

    /// Add a proof-debt replenishment event.
    pub fn add_proof_debt_replenished(
        self,
        budget_id: &str,
        credit_id: &str,
        amount_micros: u64,
        source: &str,
    ) -> Result<LedgerEntry, ClaimLedgerError> {
        self.build(LedgerEvent::ProofDebtReplenished {
            budget_id: budget_id.into(),
            credit_id: credit_id.into(),
            amount_micros,
            source: source.into(),
        })
    }

    fn build(self, event: LedgerEvent) -> Result<LedgerEntry, ClaimLedgerError> {
        let entry_digest =
            compute_entry_digest(self.sequence, self.previous_entry_digest.as_deref(), &event)?;
        Ok(LedgerEntry {
            sequence: self.sequence,
            previous_entry_digest: self.previous_entry_digest,
            event,
            entry_digest,
        })
    }
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn put_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}
fn put_str(out: &mut Vec<u8>, value: &str) -> Result<(), ClaimLedgerError> {
    let len = u64::try_from(value.len())
        .map_err(|_| ClaimLedgerError::SerializationError("string length exceeds u64".into()))?;
    put_u64(out, len);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}
fn put_vec(out: &mut Vec<u8>, values: &[String]) -> Result<(), ClaimLedgerError> {
    put_u64(
        out,
        u64::try_from(values.len()).map_err(|_| {
            ClaimLedgerError::SerializationError("vector length exceeds u64".into())
        })?,
    );
    for value in values {
        put_str(out, value)?;
    }
    Ok(())
}
fn support_state_name(state: SupportState) -> &'static str {
    match state {
        SupportState::Supported => "supported",
        SupportState::PartiallySupported => "partially_supported",
        SupportState::Unsupported => "unsupported",
        SupportState::Contradicted => "contradicted",
        SupportState::HeuristicOnly => "heuristic_only",
        SupportState::Unknown => "unknown",
    }
}

/// Produce the documented canonical preimage for an entry digest.
pub fn entry_digest_preimage(
    sequence: u64,
    previous_digest: Option<&str>,
    event: &LedgerEvent,
) -> Result<Vec<u8>, ClaimLedgerError> {
    let mut out = b"claim-ledger.entry-digest.v1".to_vec();
    put_u64(&mut out, sequence);
    match previous_digest {
        None => out.push(0),
        Some(value) => {
            out.push(1);
            put_str(&mut out, value)?;
        }
    }
    put_str(&mut out, event.type_name())?;
    match event {
        LedgerEvent::ClaimAdded {
            claim_id,
            source_id,
            span_id,
            normalized_claim,
        } => {
            put_str(&mut out, claim_id)?;
            put_str(&mut out, source_id)?;
            put_str(&mut out, span_id)?;
            put_str(&mut out, normalized_claim)?;
        }
        LedgerEvent::SupportJudgment {
            support_judgment_id,
            claim_id,
            evidence_bundle_ref,
            support_state,
            method,
        } => {
            put_str(&mut out, support_judgment_id)?;
            put_str(&mut out, claim_id)?;
            put_str(&mut out, evidence_bundle_ref)?;
            put_str(&mut out, support_state_name(*support_state))?;
            put_str(&mut out, method)?;
        }
        LedgerEvent::SupportAdmission {
            support_admission_receipt_id,
            claim_id,
            previous_support_judgment_ref,
            new_support_judgment_ref,
            admitted_support_state,
        } => {
            put_str(&mut out, support_admission_receipt_id)?;
            put_str(&mut out, claim_id)?;
            put_str(&mut out, previous_support_judgment_ref)?;
            put_str(&mut out, new_support_judgment_ref)?;
            put_str(&mut out, support_state_name(*admitted_support_state))?;
        }
        LedgerEvent::ContradictionCandidate {
            contradiction_id,
            claim_refs,
            pattern,
            rationale,
        } => {
            put_str(&mut out, contradiction_id)?;
            put_vec(&mut out, claim_refs)?;
            put_str(&mut out, pattern)?;
            put_str(&mut out, rationale)?;
        }
        LedgerEvent::ContradictionResolved {
            contradiction_resolution_receipt_id,
            contradiction_id,
            resolution,
            affected_claim_refs,
        } => {
            put_str(&mut out, contradiction_resolution_receipt_id)?;
            put_str(&mut out, contradiction_id)?;
            put_str(&mut out, resolution)?;
            put_vec(&mut out, affected_claim_refs)?;
        }
        LedgerEvent::EvidenceAttached {
            evidence_bundle_id,
            claim_id,
            evidence_link_count,
        } => {
            put_str(&mut out, evidence_bundle_id)?;
            put_str(&mut out, claim_id)?;
            put_u64(
                &mut out,
                u64::try_from(*evidence_link_count).map_err(|_| {
                    ClaimLedgerError::SerializationError("evidence link count exceeds u64".into())
                })?,
            );
        }
        LedgerEvent::BundleExported {
            bundle_id,
            export_receipt_id,
            output_ref,
            output_digest,
        } => {
            put_str(&mut out, bundle_id)?;
            put_str(&mut out, export_receipt_id)?;
            put_str(&mut out, output_ref)?;
            put_str(&mut out, output_digest)?;
        }
        LedgerEvent::ProofDebtConsumed {
            budget_id,
            debit_id,
            amount_micros,
            source,
            overdrawn,
        } => {
            put_str(&mut out, budget_id)?;
            put_str(&mut out, debit_id)?;
            put_u64(&mut out, *amount_micros);
            put_str(&mut out, source)?;
            put_bool(&mut out, *overdrawn);
        }
        LedgerEvent::ProofDebtReplenished {
            budget_id,
            credit_id,
            amount_micros,
            source,
        } => {
            put_str(&mut out, budget_id)?;
            put_str(&mut out, credit_id)?;
            put_u64(&mut out, *amount_micros);
            put_str(&mut out, source)?;
        }
    }
    Ok(out)
}

/// Compute the SHA-256 digest for an entry's canonical preimage.
pub fn compute_entry_digest(
    sequence: u64,
    previous_digest: Option<&str>,
    event: &LedgerEvent,
) -> Result<String, ClaimLedgerError> {
    Ok(sha256_bytes(&entry_digest_preimage(
        sequence,
        previous_digest,
        event,
    )?))
}

/// Verify a ledger's chain and bind it to the supplied expected head.
pub fn verify_ledger(
    entries: &[LedgerEntry],
    expected_head: &ExpectedLedgerHead,
) -> Result<LedgerVerification, ClaimLedgerError> {
    if entries.is_empty() {
        return match expected_head {
            ExpectedLedgerHead::Empty => Ok(LedgerVerification {
                last_sequence: 0,
                last_entry_digest: None,
            }),
            ExpectedLedgerHead::Entry { .. } => Err(ClaimLedgerError::LedgerCorrupt(
                "ledger is empty but a non-empty head was expected".into(),
            )),
        };
    }
    if matches!(expected_head, ExpectedLedgerHead::Empty) {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "ledger contains entries but an empty head was expected".into(),
        ));
    }
    for (index, entry) in entries.iter().enumerate() {
        let expected_sequence = u64::try_from(index + 1)
            .map_err(|_| ClaimLedgerError::LedgerCorrupt("ledger index exceeds u64".into()))?;
        if entry.sequence != expected_sequence {
            return Err(ClaimLedgerError::LedgerCorrupt(format!(
                "sequence mismatch at entry {}: expected {}, got {}",
                index + 1,
                expected_sequence,
                entry.sequence
            )));
        }
        let expected_previous = if index == 0 {
            None
        } else {
            Some(entries[index - 1].entry_digest.as_str())
        };
        if entry.previous_entry_digest.as_deref() != expected_previous {
            return Err(ClaimLedgerError::LedgerCorrupt(format!(
                "previous digest mismatch at sequence {}",
                entry.sequence
            )));
        }
        let computed = compute_entry_digest(
            entry.sequence,
            entry.previous_entry_digest.as_deref(),
            &entry.event,
        )?;
        if computed != entry.entry_digest {
            return Err(ClaimLedgerError::LedgerCorrupt(format!(
                "entry digest mismatch at sequence {}",
                entry.sequence
            )));
        }
    }
    let last = entries.last().ok_or_else(|| {
        ClaimLedgerError::LedgerCorrupt("ledger unexpectedly lost its final entry".into())
    })?;
    match expected_head {
        ExpectedLedgerHead::Entry {
            sequence,
            entry_digest,
        } if *sequence == last.sequence && entry_digest == &last.entry_digest => {
            Ok(LedgerVerification {
                last_sequence: last.sequence,
                last_entry_digest: Some(last.entry_digest.clone()),
            })
        }
        ExpectedLedgerHead::Entry {
            sequence,
            entry_digest,
        } => Err(ClaimLedgerError::LedgerCorrupt(format!(
            "ledger head mismatch: expected sequence {} digest {}, got sequence {} digest {}",
            sequence, entry_digest, last.sequence, last.entry_digest
        ))),
        ExpectedLedgerHead::Empty => Err(ClaimLedgerError::LedgerCorrupt(
            "an empty head cannot authenticate a non-empty ledger".into(),
        )),
    }
}

/// Parse JSONL strictly. Blank lines are ignored; malformed nonblank lines fail with a one-based line number.
pub fn parse_ledger_entries(jsonl: &str) -> Result<Vec<LedgerEntry>, ClaimLedgerError> {
    jsonl
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|error| {
                ClaimLedgerError::SerializationError(format!(
                    "invalid ledger JSONL at line {}: {}",
                    index + 1,
                    error
                ))
            })
        })
        .collect()
}

/// Serialize a ledger entry as exactly one JSON line.
pub fn serialize_entry(entry: &LedgerEntry) -> Result<String, ClaimLedgerError> {
    serde_json::to_string(entry)
        .map_err(|error| ClaimLedgerError::SerializationError(error.to_string()))
}
