//! MCP trusted-head query projection (FINISH_PLAN Phase 3.3).
//!
//! With `--trust-root` + `--expected-head`, the MCP server projects support
//! from an INDEPENDENTLY anchored ledger: the head file pins the expected
//! final entry ("ledger_digest") and the envelope-verified V2 admission
//! (P3.2) that anchors the head. Fail-closed at start: missing head, wrong
//! head, unproven admission anchor, or a signer that the trust root does not
//! own all abort with typed errors. Without those flags the server stays
//! exactly as merged in #43/#45: permanently unanchored.

use claim_ledger::trust_root::ProvisionedTrustRoot;
use claim_ledger::{EnvelopeVerificationStatus, SnapshotFoldV1, SupportState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Wire name of the projection stage recorded on the anchor.
pub const ANCHOR_FULLY_VERIFIED: &str = "fully_verified";

/// Typed failure modes of trusted-head provisioning. Every branch aborts;
/// none falls back to the unanchored contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustedHeadError {
    /// The ledger JSONL could not be parsed.
    LedgerParse(String),
    /// The ledger failed digest-chain/head verification.
    LedgerInvalid(String),
    /// The configured expected-head path does not exist or is unreadable.
    MissingExpectedHead(String),
    /// Head JSON malformed, mistyped, or carrying unknown fields.
    MalformedHead(String),
    /// The head's anchor admission id is not present as a `fully_verified`
    /// admission event in the verified chain (or envelope digest mismatches).
    AnchorUnmatched(String),
    /// The anchoring admission's signer id does not match the trust root.
    SignerMismatch(String),
}

impl std::fmt::Display for TrustedHeadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LedgerParse(m) => write!(f, "ledger parse failed: {m}"),
            Self::LedgerInvalid(m) => write!(f, "ledger verification failed: {m}"),
            Self::MissingExpectedHead(p) => {
                write!(f, "expected head file missing: {p}")
            }
            Self::MalformedHead(m) => write!(f, "expected head malformed: {m}"),
            Self::AnchorUnmatched(m) => write!(f, "anchor admission unmatched: {m}"),
            Self::SignerMismatch(m) => {
                write!(f, "anchor signer not in trust root: {m}")
            }
        }
    }
}

impl std::error::Error for TrustedHeadError {}

/// Independent trusted-head file (strict, deny unknown fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedHeadV1 {
    /// Schema marker; must be `ClaimLedgerMcpTrustedHeadV1`.
    pub schema: String,
    /// Expected final ledger sequence.
    pub expected_sequence: u64,
    /// Expected final ledger entry digest (the "ledger_digest" the operator
    /// provisioned).
    pub ledger_entry_digest: String,
    /// The envelope-verified admission event (P3.2) anchoring this head.
    pub anchor_admission_id: String,
    /// The envelope digest that admission admits.
    pub anchor_envelope_digest: String,
}

/// Parses and validates the trusted-head file text (fail-closed).
pub fn parse_expected_head(text: &str) -> Result<ExpectedHeadV1, TrustedHeadError> {
    let head: ExpectedHeadV1 =
        serde_json::from_str(text).map_err(|e| TrustedHeadError::MalformedHead(e.to_string()))?;
    if head.schema != "ClaimLedgerMcpTrustedHeadV1" {
        return Err(TrustedHeadError::MalformedHead(format!(
            "unsupported head schema: {schema}",
            schema = head.schema
        )));
    }
    if head.ledger_entry_digest.is_empty() || head.anchor_admission_id.is_empty() {
        return Err(TrustedHeadError::MalformedHead(
            "head fields must be non-empty".into(),
        ));
    }
    Ok(head)
}

/// Reads the head file from disk (typed error on missing/unreadable).
pub fn load_expected_head(path: &std::path::Path) -> Result<ExpectedHeadV1, TrustedHeadError> {
    let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            TrustedHeadError::MissingExpectedHead(path.display().to_string())
        }
        _ => TrustedHeadError::MissingExpectedHead(format!("{}: read error: {e}", path.display())),
    })?;
    parse_expected_head(&text)
}

/// Result of a verified (or explicitly unanchored) projection.
#[derive(Debug, Clone)]
pub struct TrustedHeadProjection {
    /// Whether the independent anchor verified.
    pub anchored: bool,
    /// Expected final ledger sequence (from the head).
    pub last_sequence: u64,
    /// The anchoring admission (set iff anchored).
    pub anchor: Option<AnchorEvidence>,
    /// claim_id → support state the anchor admits; empty in unanchored mode.
    pub claim_support: BTreeMap<String, SupportState>,
}

/// Read-only record of the anchoring admission for tool output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorEvidence {
    pub admission_id: String,
    pub signer_id: String,
    pub policy_id: String,
    pub trusted_timestamp_ns: i64,
    pub envelope_digest: String,
    pub envelope_verification_name: String,
}

/// Loads ledger text and projects under the configured mode.
///
/// - `trust_root_text` + `head_file_text` both `None` → the merged #43/#45
///   unanchored contract (`anchored: false`, no support projection).
/// - Trust root without head → typed fail-closed error (never a fallback).
///
/// `load_expected_head` reads the head from disk for the CLI; this function
/// takes already-read text so every code path is directly testable.
pub fn load_projected_entries(
    ledger_jsonl: &str,
    trust_root_text: Option<&str>,
    head_file_text: Option<&str>,
) -> Result<TrustedHeadProjection, TrustedHeadError> {
    match (trust_root_text, head_file_text) {
        (None, None) => Ok(TrustedHeadProjection {
            anchored: false,
            last_sequence: 0,
            anchor: None,
            claim_support: BTreeMap::new(),
        }),
        (Some(root_text), Some(head_text)) => {
            let root = claim_ledger::trust_root::load_trust_root_from_str(root_text)
                .map_err(|e| TrustedHeadError::SignerMismatch(e.to_string()))?;
            let head = parse_expected_head(head_text)?;
            project_with_trusted_head(ledger_jsonl, &root, &head)
        }
        (Some(_), None) => Err(TrustedHeadError::MissingExpectedHead(
            "trust root provisioned without an expected head".into(),
        )),
        // A head without a trust root cannot authorize any signer; nothing is
        // anchored. Unreachable via the MCP CLI (flags pair up), typed for
        // direct callers.
        (None, Some(_)) => Err(TrustedHeadError::SignerMismatch(
            "expected head provisioned without a trust root".into(),
        )),
    }
}

/// Verifies the chain against the expected head and projects with the
/// canonical V2 fold, gating support on the envelope-verified admission.
pub fn project_with_trusted_head(
    ledger_jsonl: &str,
    root: &ProvisionedTrustRoot,
    head: &ExpectedHeadV1,
) -> Result<TrustedHeadProjection, TrustedHeadError> {
    let entries = claim_ledger::parse_ledger_entries(ledger_jsonl)
        .map_err(|e| TrustedHeadError::LedgerParse(e.to_string()))?;

    // 1. Independent head verification (sequence + digest must match exactly).
    let expected = if head.expected_sequence == 0 && entries.is_empty() {
        claim_ledger::ExpectedLedgerHead::Empty
    } else {
        claim_ledger::ExpectedLedgerHead::new(head.expected_sequence, &head.ledger_entry_digest)
    };
    claim_ledger::verify_ledger(&entries, &expected)
        .map_err(|e| TrustedHeadError::LedgerInvalid(e.to_string()))?;

    // 2. The anchor admission must exist as a fully-verified admission event.
    let mut anchor_evidence = None;
    for entry in &entries {
        if let claim_ledger::LedgerEvent::AdmissionEvent { payload } = &entry.event {
            if payload.admission_id == head.anchor_admission_id {
                if payload.envelope_digest != head.anchor_envelope_digest {
                    return Err(TrustedHeadError::AnchorUnmatched(format!(
                        "anchor {} admits a different envelope digest than the head records",
                        payload.admission_id
                    )));
                }
                if payload.signer_id != root.signer_id {
                    return Err(TrustedHeadError::SignerMismatch(format!(
                        "anchor signer '{}' != trust-root signer '{}'",
                        payload.signer_id, root.signer_id
                    )));
                }
                if payload.envelope_verification != EnvelopeVerificationStatus::FullyVerified {
                    return Err(TrustedHeadError::AnchorUnmatched(format!(
                        "anchor admission {} is not fully_verified",
                        payload.admission_id
                    )));
                }
                anchor_evidence = Some(AnchorEvidence {
                    admission_id: payload.admission_id.clone(),
                    signer_id: payload.signer_id.clone(),
                    policy_id: payload.policy_id.clone(),
                    trusted_timestamp_ns: payload.trusted_timestamp_ns,
                    envelope_digest: payload.envelope_digest.clone(),
                    envelope_verification_name: payload.envelope_verification_name().to_string(),
                });
                break;
            }
        }
    }
    let anchor_evidence = anchor_evidence.ok_or_else(|| {
        TrustedHeadError::AnchorUnmatched(format!(
            "anchor admission {} not present in the verified ledger",
            head.anchor_admission_id
        ))
    })?;

    // 3. Canonical V2 fold (legacy SupportAdmission events stay unprojected;
    //    the projection is shared with snapshot v1 exactly as merged in
    //    P3.2, so the #43/#45 fence is structurally preserved).
    let mut fold = SnapshotFoldV1::default();
    fold.apply_chain(&entries)
        .map_err(|e| TrustedHeadError::LedgerInvalid(e.to_string()))?;
    let snapshot = fold
        .into_snapshot(chrono::Utc::now())
        .map_err(|e| TrustedHeadError::LedgerInvalid(e.to_string()))?;

    let claim_support = snapshot
        .claim_support
        .into_iter()
        .map(|item| (item.claim_id, item.support_state))
        .collect();

    Ok(TrustedHeadProjection {
        anchored: true,
        last_sequence: head.expected_sequence,
        anchor: Some(anchor_evidence),
        claim_support,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_anchor_admission_id() {
        let text = "{\"schema\":\"ClaimLedgerMcpTrustedHeadV1\",\"expected_sequence\":1,\"ledger_entry_digest\":\"abc\",\"anchor_admission_id\":\"\",\"anchor_envelope_digest\":\"def\"}";
        assert!(matches!(
            parse_expected_head(text),
            Err(TrustedHeadError::MalformedHead(_))
        ));
    }
}
