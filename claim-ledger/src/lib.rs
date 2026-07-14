//! # claim-ledger
//!
//! Deterministic, local-first claim/evidence/provenance ledger.
//!
//! Creates receipts for all material operations.
//!
//! ## Core Concepts
//!
//! - **Claim**: A source-spanned atomic assertion extracted from a document.
//! - **Evidence Bundle**: A collection of evidence links supporting a claim.
//! - **Support Judgment**: A scoped support state assigned to a claim via an evidence bundle.
//! - **Support Admission**: An operator-admitted or fixture-admitted upgrade to a support judgment.
//! - **Contradiction Record**: A detected conflict between two claims with a resolution lifecycle.
//! - **Claim Ledger**: An append-only, hash-chained ledger of claim events and support states.
//! - **Export Receipt**: A binding receipt digest for any material output operation.
//!
//! ## Crate Architecture
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`ids`] | ULID/Hash-based stable identifiers |
//! | [`error`] | Thiserror-based error types |
//! | [`types`] | Domain types: Claim, EvidenceBundle, SupportJudgment, etc. |
//! | [`ledger`] | Append-only hash-chained ledger |
//! | [`receipt`] | Export and admission receipt types |

pub mod budget;
pub mod candidate;
pub mod envelope;
pub mod error;
pub mod ids;
pub mod ledger;
pub mod receipt;
pub mod types;

// Re-export commonly used types at the crate root for ergonomic access.
pub use budget::{
    budget_for_claim, evaluate_proof_debt_gate, evaluate_proof_debt_gate_with_config,
    evaluate_proof_debt_gate_with_waiver, evaluate_proof_debt_gate_with_waiver_and_config,
    proof_debt_weight, total_proof_debt_weight, total_proof_debt_weight_with_config,
    verify_proof_debt_waiver, ProofDebtBudgetConfig, ProofDebtBudgetV1, ProofDebtCreditV1,
    ProofDebtDebitV1, ProofDebtGateDecision, ProofDebtGateResult, ProofDebtSummaryV1,
    ProofDebtWaiverReceipt, ProofDebtWaiverValidationError, VerifiedProofDebtWaiver,
    PROOF_DEBT_WAIVER_AUTHORIZATION_DOMAIN, PROOF_DEBT_WAIVER_SCHEMA_VERSION,
};
pub use candidate::{
    ProofPacketCandidateProvenanceV1, SimilarClaimCandidateV1,
    PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA, SIMILAR_CLAIM_CANDIDATE_V1_SCHEMA,
};
pub use envelope::{
    ArtifactEnvelopeV1, EnvelopeError, EnvelopeVerificationContext, EnvelopeVerificationReport,
    EnvelopeVerificationStatus, PolicyAdmission,
};
pub use error::ClaimLedgerError;
pub use ids::{normalize_text, sha256_bytes, sha256_text, stable_id, ulid};
pub use ledger::{
    compact_ledger, compact_ledger_from_snapshot, compute_entry_digest, compute_snapshot_digest,
    entry_digest_preimage, parse_ledger_entries, serialize_entry, verify_compaction, verify_ledger,
    verify_ledger_tail, verify_snapshot, CompactedLedger, CompactionPolicy, CompactionReceipt,
    ExpectedLedgerHead, LedgerEntry, LedgerEntryBuilder, LedgerEvent, LedgerSnapshot,
    LedgerVerification, SnapshotClaim, SnapshotClaimSupport, SnapshotContentClaimLink,
    SnapshotContradictionState, SnapshotFactClaimLink, SnapshotSupportJudgment,
    UnprojectableEventPolicy,
};
pub use receipt::{
    ContradictionResolutionReceipt, ExportReceipt, LedgerAppendReceipt, SupersessionReceipt,
    SupportAdmissionReceipt,
};
pub use types::{
    Claim, ContradictionRecord, ContradictionResolution, ContradictionResolutionRecord,
    ContradictionStatus, EvidenceBundle, EvidenceLink, EvidenceRelation, ProofDebt, SourceArtifact,
    SourceIndex, SourceSpan, Supersession, SupportAdmission, SupportAdmissionMethod,
    SupportJudgment, SupportProofPayload, SupportState,
};
