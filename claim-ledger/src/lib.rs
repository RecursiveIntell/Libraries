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

pub mod error;
pub mod ids;
pub mod ledger;
pub mod receipt;
pub mod types;

// Re-export commonly used types at the crate root for ergonomic access.
pub use error::ClaimLedgerError;
pub use ids::{normalize_text, sha256_bytes, sha256_text, stable_id, ulid};
pub use ledger::{
    parse_ledger_entries, serialize_entry, verify_ledger, LedgerEntry, LedgerEntryBuilder,
    LedgerEvent, LedgerVerification,
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
