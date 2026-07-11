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

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
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

/// Current claim content captured by a ledger checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotClaim {
    pub claim_id: String,
    pub source_id: String,
    pub span_id: String,
    pub normalized_claim: String,
}

/// Durable fact-to-claim link derived from a claim's semantic-memory source id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFactClaimLink {
    pub fact_id: String,
    pub claim_id: String,
}

/// Last-writer-wins reverse mapping used for content-based claim linking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotContentClaimLink {
    pub normalized_claim: String,
    pub claim_id: String,
}

/// Latest support-judgment payload for a claim at the checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotSupportJudgment {
    pub support_judgment_id: String,
    pub claim_id: String,
    pub evidence_bundle_ref: String,
    pub support_state: SupportState,
    pub method: String,
}

/// Authoritative projected support state after all checkpoint events are folded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotClaimSupport {
    pub claim_id: String,
    pub support_state: SupportState,
}

/// Projected lifecycle state of a contradiction at the checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotContradictionState {
    pub contradiction_id: String,
    pub claim_refs: Vec<String>,
    pub pattern: String,
    pub rationale: String,
    pub resolution: Option<String>,
    pub affected_claim_refs: Vec<String>,
}

/// Verified checkpoint of the state projected from a prefix of the claim ledger.
///
/// Vector fields are sorted by their stable identifiers before hashing. The
/// `snapshot_digest` is SHA-256 over a domain-separated JSON encoding of this
/// structure with `snapshot_digest` cleared. Because this format contains no
/// JSON maps or floating-point values, its v1 encoding is deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub format_version: String,
    pub claims: Vec<SnapshotClaim>,
    pub fact_to_claim_links: Vec<SnapshotFactClaimLink>,
    pub content_to_claim_links: Vec<SnapshotContentClaimLink>,
    pub support_judgments: Vec<SnapshotSupportJudgment>,
    pub claim_support: Vec<SnapshotClaimSupport>,
    pub contradiction_states: Vec<SnapshotContradictionState>,
    pub last_compacted_sequence: u64,
    pub last_compacted_entry_digest: Option<String>,
    pub created_at: DateTime<Utc>,
    pub snapshot_digest: String,
}

/// Receipt binding a verified snapshot to the original pre-compaction head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionReceipt {
    pub format_version: String,
    pub snapshot_digest: String,
    pub snapshot_sequence: u64,
    pub snapshot_entry_digest: Option<String>,
    pub pre_compaction_sequence: u64,
    pub pre_compaction_entry_digest: Option<String>,
    pub retained_tail_start_sequence: Option<u64>,
    pub retained_tail_entries: u64,
    pub previous_snapshot_digest: Option<String>,
    pub created_at: DateTime<Utc>,
    pub receipt_digest: String,
}

/// Handling for a ledger event that a snapshot projector does not understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnprojectableEventPolicy {
    /// Keep the event and every later entry in the retained, verifiable tail.
    Retain,
    /// Refuse to compact rather than discard an event.
    FailClosed,
}

/// Explicit policy controlling which prefix may be checkpointed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionPolicy {
    /// Minimum number of most-recent events retained verbatim.
    pub retain_tail_entries: usize,
    /// Behavior for events not covered by the v1 projector.
    pub unprojectable_events: UnprojectableEventPolicy,
}

impl Default for CompactionPolicy {
    fn default() -> Self {
        Self {
            retain_tail_entries: 256,
            unprojectable_events: UnprojectableEventPolicy::Retain,
        }
    }
}

/// Result of a successful in-memory compaction plan.
#[derive(Debug, Clone)]
pub struct CompactedLedger {
    pub snapshot: LedgerSnapshot,
    pub retained_tail: Vec<LedgerEntry>,
    pub receipt: CompactionReceipt,
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

#[derive(Default)]
struct SnapshotProjection {
    claims: BTreeMap<String, SnapshotClaim>,
    fact_links: BTreeMap<String, SnapshotFactClaimLink>,
    content_links: BTreeMap<String, SnapshotContentClaimLink>,
    judgments: BTreeMap<String, SnapshotSupportJudgment>,
    claim_support: BTreeMap<String, SnapshotClaimSupport>,
    contradictions: BTreeMap<String, SnapshotContradictionState>,
}

impl SnapshotProjection {
    fn from_snapshot(snapshot: &LedgerSnapshot) -> Self {
        Self {
            claims: snapshot
                .claims
                .iter()
                .cloned()
                .map(|item| (item.claim_id.clone(), item))
                .collect(),
            fact_links: snapshot
                .fact_to_claim_links
                .iter()
                .cloned()
                .map(|item| (item.fact_id.clone(), item))
                .collect(),
            content_links: snapshot
                .content_to_claim_links
                .iter()
                .cloned()
                .map(|item| (item.normalized_claim.clone(), item))
                .collect(),
            judgments: snapshot
                .support_judgments
                .iter()
                .cloned()
                .map(|item| (item.claim_id.clone(), item))
                .collect(),
            claim_support: snapshot
                .claim_support
                .iter()
                .cloned()
                .map(|item| (item.claim_id.clone(), item))
                .collect(),
            contradictions: snapshot
                .contradiction_states
                .iter()
                .cloned()
                .map(|item| (item.contradiction_id.clone(), item))
                .collect(),
        }
    }

    fn event_is_projectable(event: &LedgerEvent) -> bool {
        matches!(
            event,
            LedgerEvent::ClaimAdded { .. }
                | LedgerEvent::SupportJudgment { .. }
                | LedgerEvent::ContradictionCandidate { .. }
                | LedgerEvent::ContradictionResolved { .. }
        )
    }

    fn apply(&mut self, event: &LedgerEvent) {
        match event {
            LedgerEvent::ClaimAdded {
                claim_id,
                source_id,
                span_id,
                normalized_claim,
            } => {
                self.claims.insert(
                    claim_id.clone(),
                    SnapshotClaim {
                        claim_id: claim_id.clone(),
                        source_id: source_id.clone(),
                        span_id: span_id.clone(),
                        normalized_claim: normalized_claim.clone(),
                    },
                );
                if let Some(fact_id) = source_id.strip_prefix("semantic-memory:fact:") {
                    self.fact_links.insert(
                        fact_id.to_string(),
                        SnapshotFactClaimLink {
                            fact_id: fact_id.to_string(),
                            claim_id: claim_id.clone(),
                        },
                    );
                }
                self.content_links.insert(
                    normalized_claim.clone(),
                    SnapshotContentClaimLink {
                        normalized_claim: normalized_claim.clone(),
                        claim_id: claim_id.clone(),
                    },
                );
            }
            LedgerEvent::SupportJudgment {
                support_judgment_id,
                claim_id,
                evidence_bundle_ref,
                support_state,
                method,
            } => {
                self.judgments.insert(
                    claim_id.clone(),
                    SnapshotSupportJudgment {
                        support_judgment_id: support_judgment_id.clone(),
                        claim_id: claim_id.clone(),
                        evidence_bundle_ref: evidence_bundle_ref.clone(),
                        support_state: *support_state,
                        method: method.clone(),
                    },
                );
                self.claim_support.insert(
                    claim_id.clone(),
                    SnapshotClaimSupport {
                        claim_id: claim_id.clone(),
                        support_state: *support_state,
                    },
                );
            }
            LedgerEvent::ContradictionCandidate {
                contradiction_id,
                claim_refs,
                pattern,
                rationale,
            } => {
                self.contradictions.insert(
                    contradiction_id.clone(),
                    SnapshotContradictionState {
                        contradiction_id: contradiction_id.clone(),
                        claim_refs: claim_refs.clone(),
                        pattern: pattern.clone(),
                        rationale: rationale.clone(),
                        resolution: None,
                        affected_claim_refs: Vec::new(),
                    },
                );
                for claim_id in claim_refs {
                    self.claim_support.insert(
                        claim_id.clone(),
                        SnapshotClaimSupport {
                            claim_id: claim_id.clone(),
                            support_state: SupportState::Contradicted,
                        },
                    );
                }
            }
            LedgerEvent::ContradictionResolved {
                contradiction_id,
                resolution,
                affected_claim_refs,
                ..
            } => {
                let state = self
                    .contradictions
                    .entry(contradiction_id.clone())
                    .or_insert_with(|| SnapshotContradictionState {
                        contradiction_id: contradiction_id.clone(),
                        claim_refs: Vec::new(),
                        pattern: String::new(),
                        rationale: String::new(),
                        resolution: None,
                        affected_claim_refs: Vec::new(),
                    });
                state.resolution = Some(resolution.clone());
                state.affected_claim_refs = affected_claim_refs.clone();
            }
            LedgerEvent::SupportAdmission { .. }
            | LedgerEvent::EvidenceAttached { .. }
            | LedgerEvent::BundleExported { .. }
            | LedgerEvent::ProofDebtConsumed { .. }
            | LedgerEvent::ProofDebtReplenished { .. } => {}
        }
    }

    fn into_snapshot(
        self,
        sequence: u64,
        entry_digest: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<LedgerSnapshot, ClaimLedgerError> {
        let mut snapshot = LedgerSnapshot {
            format_version: "claim-ledger.snapshot.v1".to_string(),
            claims: self.claims.into_values().collect(),
            fact_to_claim_links: self.fact_links.into_values().collect(),
            content_to_claim_links: self.content_links.into_values().collect(),
            support_judgments: self.judgments.into_values().collect(),
            claim_support: self.claim_support.into_values().collect(),
            contradiction_states: self.contradictions.into_values().collect(),
            last_compacted_sequence: sequence,
            last_compacted_entry_digest: entry_digest,
            created_at,
            snapshot_digest: String::new(),
        };
        snapshot.snapshot_digest = compute_snapshot_digest(&snapshot)?;
        Ok(snapshot)
    }
}

fn digest_json<T: Serialize>(domain: &[u8], value: &T) -> Result<String, ClaimLedgerError> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| ClaimLedgerError::SerializationError(error.to_string()))?;
    let mut preimage = domain.to_vec();
    put_u64(
        &mut preimage,
        u64::try_from(encoded.len()).map_err(|_| {
            ClaimLedgerError::SerializationError("canonical JSON exceeds u64".into())
        })?,
    );
    preimage.extend_from_slice(&encoded);
    Ok(sha256_bytes(&preimage))
}

/// Compute the canonical digest of a snapshot (excluding its digest field).
pub fn compute_snapshot_digest(snapshot: &LedgerSnapshot) -> Result<String, ClaimLedgerError> {
    let mut unsigned = snapshot.clone();
    unsigned.snapshot_digest.clear();
    digest_json(b"claim-ledger.snapshot-digest.v1", &unsigned)
}

fn compute_compaction_receipt_digest(
    receipt: &CompactionReceipt,
) -> Result<String, ClaimLedgerError> {
    let mut unsigned = receipt.clone();
    unsigned.receipt_digest.clear();
    digest_json(b"claim-ledger.compaction-receipt-digest.v1", &unsigned)
}

fn strictly_sorted_by<T, F>(items: &[T], key: F) -> bool
where
    F: Fn(&T) -> &str,
{
    items.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

/// Verify a snapshot's format, canonical ordering, anchor, and digest.
pub fn verify_snapshot(snapshot: &LedgerSnapshot) -> Result<(), ClaimLedgerError> {
    if snapshot.format_version != "claim-ledger.snapshot.v1" {
        return Err(ClaimLedgerError::LedgerCorrupt(format!(
            "unsupported snapshot format {}",
            snapshot.format_version
        )));
    }
    if snapshot.last_compacted_sequence == 0 && snapshot.last_compacted_entry_digest.is_some()
        || snapshot.last_compacted_sequence > 0 && snapshot.last_compacted_entry_digest.is_none()
    {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "snapshot sequence/digest anchor is inconsistent".into(),
        ));
    }
    let canonical = strictly_sorted_by(&snapshot.claims, |item| &item.claim_id)
        && strictly_sorted_by(&snapshot.fact_to_claim_links, |item| &item.fact_id)
        && strictly_sorted_by(&snapshot.content_to_claim_links, |item| {
            &item.normalized_claim
        })
        && strictly_sorted_by(&snapshot.support_judgments, |item| &item.claim_id)
        && strictly_sorted_by(&snapshot.claim_support, |item| &item.claim_id)
        && strictly_sorted_by(&snapshot.contradiction_states, |item| {
            &item.contradiction_id
        });
    if !canonical {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "snapshot collections are not in canonical unique order".into(),
        ));
    }
    let computed = compute_snapshot_digest(snapshot)?;
    if computed != snapshot.snapshot_digest {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "snapshot digest mismatch".into(),
        ));
    }
    Ok(())
}

/// Verify a tail whose first entry continues from a snapshot checkpoint.
pub fn verify_ledger_tail(
    entries: &[LedgerEntry],
    anchor_sequence: u64,
    anchor_digest: Option<&str>,
) -> Result<LedgerVerification, ClaimLedgerError> {
    if (anchor_sequence == 0) != anchor_digest.is_none() {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "tail anchor sequence/digest is inconsistent".into(),
        ));
    }
    let mut previous_sequence = anchor_sequence;
    let mut previous_digest = anchor_digest.map(str::to_owned);
    for entry in entries {
        let expected_sequence = previous_sequence
            .checked_add(1)
            .ok_or_else(|| ClaimLedgerError::LedgerCorrupt("ledger sequence overflow".into()))?;
        if entry.sequence != expected_sequence {
            return Err(ClaimLedgerError::LedgerCorrupt(format!(
                "tail sequence mismatch: expected {}, got {}",
                expected_sequence, entry.sequence
            )));
        }
        if entry.previous_entry_digest.as_deref() != previous_digest.as_deref() {
            return Err(ClaimLedgerError::LedgerCorrupt(format!(
                "tail previous digest mismatch at sequence {}",
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
                "tail entry digest mismatch at sequence {}",
                entry.sequence
            )));
        }
        previous_sequence = entry.sequence;
        previous_digest = Some(entry.entry_digest.clone());
    }
    Ok(LedgerVerification {
        last_sequence: previous_sequence,
        last_entry_digest: previous_digest,
    })
}

/// Verify the snapshot digest, anchored retained tail, receipt digest, and
/// binding to the original pre-compaction ledger head.
pub fn verify_compaction(
    snapshot: &LedgerSnapshot,
    retained_tail: &[LedgerEntry],
    receipt: &CompactionReceipt,
) -> Result<LedgerVerification, ClaimLedgerError> {
    verify_snapshot(snapshot)?;
    if receipt.format_version != "claim-ledger.compaction-receipt.v1" {
        return Err(ClaimLedgerError::LedgerCorrupt(format!(
            "unsupported compaction receipt format {}",
            receipt.format_version
        )));
    }
    if compute_compaction_receipt_digest(receipt)? != receipt.receipt_digest {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "compaction receipt digest mismatch".into(),
        ));
    }
    if receipt.snapshot_digest != snapshot.snapshot_digest
        || receipt.snapshot_sequence != snapshot.last_compacted_sequence
        || receipt.snapshot_entry_digest != snapshot.last_compacted_entry_digest
    {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "compaction receipt does not bind the supplied snapshot".into(),
        ));
    }
    let retained_count = u64::try_from(retained_tail.len())
        .map_err(|_| ClaimLedgerError::LedgerCorrupt("retained tail exceeds u64".into()))?;
    let retained_start = retained_tail.first().map(|entry| entry.sequence);
    if receipt.retained_tail_entries != retained_count
        || receipt.retained_tail_start_sequence != retained_start
    {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "compaction receipt retained-tail metadata mismatch".into(),
        ));
    }
    let verification = verify_ledger_tail(
        retained_tail,
        snapshot.last_compacted_sequence,
        snapshot.last_compacted_entry_digest.as_deref(),
    )?;
    if verification.last_sequence != receipt.pre_compaction_sequence
        || verification.last_entry_digest != receipt.pre_compaction_entry_digest
    {
        return Err(ClaimLedgerError::LedgerCorrupt(
            "compacted ledger does not reach the receipt's pre-compaction head".into(),
        ));
    }
    Ok(verification)
}

/// Compact a complete sequence-1 ledger into a verified checkpoint and tail.
pub fn compact_ledger(
    entries: &[LedgerEntry],
    policy: &CompactionPolicy,
) -> Result<CompactedLedger, ClaimLedgerError> {
    compact_ledger_from_snapshot(None, entries, policy)
}

/// Advance an existing verified checkpoint using its anchored retained tail.
pub fn compact_ledger_from_snapshot(
    prior_snapshot: Option<&LedgerSnapshot>,
    entries: &[LedgerEntry],
    policy: &CompactionPolicy,
) -> Result<CompactedLedger, ClaimLedgerError> {
    let (base_sequence, base_digest, mut projection, previous_snapshot_digest) =
        match prior_snapshot {
            Some(snapshot) => {
                verify_snapshot(snapshot)?;
                (
                    snapshot.last_compacted_sequence,
                    snapshot.last_compacted_entry_digest.clone(),
                    SnapshotProjection::from_snapshot(snapshot),
                    Some(snapshot.snapshot_digest.clone()),
                )
            }
            None => (0, None, SnapshotProjection::default(), None),
        };
    let pre_head = verify_ledger_tail(entries, base_sequence, base_digest.as_deref())?;
    let mut compact_count = entries.len().saturating_sub(policy.retain_tail_entries);
    if let Some(index) = entries[..compact_count]
        .iter()
        .position(|entry| !SnapshotProjection::event_is_projectable(&entry.event))
    {
        match policy.unprojectable_events {
            UnprojectableEventPolicy::Retain => compact_count = index,
            UnprojectableEventPolicy::FailClosed => {
                return Err(ClaimLedgerError::LedgerCorrupt(format!(
                    "event type {} at sequence {} is not projectable by snapshot v1",
                    entries[index].event.type_name(),
                    entries[index].sequence
                )));
            }
        }
    }
    for entry in &entries[..compact_count] {
        projection.apply(&entry.event);
    }
    let checkpoint = compact_count
        .checked_sub(1)
        .and_then(|index| entries.get(index));
    let checkpoint_sequence = checkpoint.map_or(base_sequence, |entry| entry.sequence);
    let checkpoint_digest = checkpoint
        .map(|entry| entry.entry_digest.clone())
        .or(base_digest);
    let created_at = Utc::now();
    let snapshot =
        projection.into_snapshot(checkpoint_sequence, checkpoint_digest.clone(), created_at)?;
    let retained_tail = entries[compact_count..].to_vec();
    let mut receipt = CompactionReceipt {
        format_version: "claim-ledger.compaction-receipt.v1".to_string(),
        snapshot_digest: snapshot.snapshot_digest.clone(),
        snapshot_sequence: snapshot.last_compacted_sequence,
        snapshot_entry_digest: snapshot.last_compacted_entry_digest.clone(),
        pre_compaction_sequence: pre_head.last_sequence,
        pre_compaction_entry_digest: pre_head.last_entry_digest,
        retained_tail_start_sequence: retained_tail.first().map(|entry| entry.sequence),
        retained_tail_entries: u64::try_from(retained_tail.len()).map_err(|_| {
            ClaimLedgerError::SerializationError("retained tail exceeds u64".into())
        })?,
        previous_snapshot_digest,
        created_at,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = compute_compaction_receipt_digest(&receipt)?;
    verify_compaction(&snapshot, &retained_tail, &receipt)?;
    Ok(CompactedLedger {
        snapshot,
        retained_tail,
        receipt,
    })
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
