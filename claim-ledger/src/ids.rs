//! Stable identifier generation using ULID and cryptographic hash.
//!
//! All identifiers are deterministic: the same inputs always produce the same identifier.

use sha2::{Digest, Sha256};
use ulid::Ulid;

/// Normalize text for comparison (collapse whitespace, trim).
pub fn normalize_text(text: &str) -> String {
    let trimmed = text.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    let mut result = Vec::with_capacity(chars.len());
    for c in chars {
        if result.is_empty()
            || !result
                .last()
                .is_some_and(|last: &char| last.is_whitespace() && c.is_whitespace())
        {
            result.push(c);
        }
    }
    let leading = result.iter().take_while(|c| c.is_whitespace()).count();
    let trailing = result
        .iter()
        .rev()
        .take_while(|c| c.is_whitespace())
        .count();
    let end = result.len() - trailing;
    result[leading..end].iter().collect()
}

/// Generate a stable identifier from a prefix and variable parts.
///
/// Uses SHA-256 of a versioned, domain-separated binary preimage, truncated to
/// `length` characters. The preimage is `claim-ledger.stable-id.v2` followed by
/// a byte-length-prefixed UTF-8 prefix, an unsigned 64-bit big-endian part
/// count, and each part as an unsigned 64-bit big-endian byte length followed
/// by its UTF-8 bytes. This distinguishes embedded newlines, empty parts, and
/// differing partitions of the same text.
/// The result is prefixed with `{prefix}_` so it is easy to identify the kind of ID.
///
/// # Arguments
///
/// * `prefix` — Short lowercase prefix (e.g., `clm`, `evb`, `sup`)
/// * `parts` — Variable number of string parts to include in the hash input
/// * `length` — Number of characters of the hex digest to use (default: 20)
///
/// # Example
///
/// ```
/// use claim_ledger::ids::stable_id;
///
/// let id = stable_id("clm", &["source-123", "span-456", "claim text"], 20);
/// assert!(id.starts_with("clm_"));
/// ```
pub fn stable_id(prefix: &str, parts: &[&str], length: usize) -> String {
    let mut input = b"claim-ledger.stable-id.v2".to_vec();
    put_length_prefixed(&mut input, prefix);
    input.extend_from_slice(&(parts.len() as u64).to_be_bytes());
    for part in parts {
        put_length_prefixed(&mut input, part);
    }
    let hash = Sha256::digest(&input);
    let hex = hex::encode(hash);
    let truncated = &hex[..length.min(hex.len())];
    format!("{}_{}", prefix, truncated)
}

fn put_length_prefixed(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u64).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}

/// Generate a new unique ULID.
pub fn ulid() -> String {
    Ulid::new().to_string()
}

/// Compute SHA-256 of a string and return the hex digest.
pub fn sha256_text(text: &str) -> String {
    let hash = Sha256::digest(text.as_bytes());
    hex::encode(hash)
}

/// Compute SHA-256 of raw bytes and return the hex digest.
pub fn sha256_bytes(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

// Convenience constructor helpers

/// Build a claim ID from source and span identifiers.
pub fn claim_id(source_id: &str, span_id: &str, text: &str) -> String {
    stable_id("clm", &[source_id, span_id, text], 20)
}

/// Build an evidence bundle ID for a given claim.
pub fn evidence_bundle_id(claim_id: &str) -> String {
    stable_id("evb", &[claim_id], 20)
}

/// Build a support judgment ID from a claim and support state.
pub fn support_judgment_id(claim_id: &str, support_state: &str) -> String {
    stable_id("sup", &[claim_id, support_state], 20)
}

/// Build a contradiction ID from two ordered claim IDs and a pattern.
pub fn contradiction_id(left_claim_id: &str, right_claim_id: &str, pattern: &str) -> String {
    let (a, b) = if left_claim_id < right_claim_id {
        (left_claim_id, right_claim_id)
    } else {
        (right_claim_id, left_claim_id)
    };
    stable_id("ctr", &[a, b, pattern], 20)
}

/// Build an export receipt ID from operation and input references.
pub fn export_receipt_id(operation: &str, input_refs: &[&str]) -> String {
    stable_id(
        "xpt",
        [operation]
            .iter()
            .chain(input_refs.iter())
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
        20,
    )
}

/// Build a support admission receipt ID from claim and judgment refs.
pub fn support_admission_receipt_id(claim_id: &str, prev_ref: &str, new_ref: &str) -> String {
    stable_id("sar", &[claim_id, prev_ref, new_ref], 20)
}

/// Build a ledger append receipt ID from ledger path, sequence, and entry digest.
pub fn ledger_append_receipt_id(ledger_path: &str, sequence: u64, entry_digest: &str) -> String {
    stable_id(
        "lap",
        &[ledger_path, &sequence.to_string(), entry_digest],
        20,
    )
}

/// Build a supersession receipt ID from superseded and superseding refs.
pub fn supersession_receipt_id(superseded_ref: &str, superseding_ref: &str) -> String {
    stable_id("ssr", &[superseded_ref, superseding_ref], 20)
}

/// Build a contradiction resolution receipt ID.
pub fn contradiction_resolution_receipt_id(
    contradiction_id: &str,
    prev_status: &str,
    resolution: &str,
) -> String {
    stable_id("crr", &[contradiction_id, prev_status, resolution], 20)
}

/// Build a proof-debt budget ID from a scope.
pub fn proof_debt_budget_id(scope: &str) -> String {
    stable_id("pdb", &[scope], 20)
}

/// Build a proof-debt debit ID from budget, source, and amount.
pub fn proof_debt_debit_id(budget_id: &str, source: &str, amount_micros: u64) -> String {
    stable_id("pdd", &[budget_id, source, &amount_micros.to_string()], 20)
}

/// Build a proof-debt credit ID from budget, source, and amount.
pub fn proof_debt_credit_id(budget_id: &str, source: &str, amount_micros: u64) -> String {
    stable_id("pdc", &[budget_id, source, &amount_micros.to_string()], 20)
}

/// Complete deterministic input for a proof-debt waiver identity.
///
/// `recorded_time` must use the canonical RFC 3339 UTC representation from
/// [`crate::budget::ProofDebtWaiverReceipt::canonical_recorded_time`].
pub struct ProofDebtWaiverIdParts<'a> {
    /// Receipt schema version.
    pub schema_version: &'a str,
    /// Authorization domain.
    pub authorization_domain: &'a str,
    /// Bound budget identity.
    pub budget_id: &'a str,
    /// Bound budget scope.
    pub scope: &'a str,
    /// Bound budget ceiling.
    pub budget_micros: u64,
    /// Outstanding debt captured when issuing the waiver.
    pub outstanding_debt_micros: u64,
    /// Amount authorized by the waiver.
    pub waived_amount_micros: u64,
    /// Operator recorded as the waiver issuer.
    pub operator_ref: &'a str,
    /// Reason recorded for the waiver.
    pub rationale: &'a str,
    /// Canonical recorded time.
    pub recorded_time: &'a str,
}

/// Build a proof-debt waiver ID from its complete authorization semantics.
pub fn proof_debt_waiver_id(parts: ProofDebtWaiverIdParts<'_>) -> String {
    let budget_micros = parts.budget_micros.to_string();
    let outstanding_debt_micros = parts.outstanding_debt_micros.to_string();
    let waived_amount_micros = parts.waived_amount_micros.to_string();
    stable_id(
        "pdw",
        &[
            parts.schema_version,
            parts.authorization_domain,
            parts.budget_id,
            parts.scope,
            &budget_micros,
            &outstanding_debt_micros,
            &waived_amount_micros,
            parts.operator_ref,
            parts.rationale,
            parts.recorded_time,
        ],
        20,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_is_deterministic() {
        let id1 = stable_id("test", &["a", "b", "c"], 16);
        let id2 = stable_id("test", &["a", "b", "c"], 16);
        assert_eq!(id1, id2);
    }

    #[test]
    fn stable_id_differs_for_different_inputs() {
        let id1 = stable_id("test", &["a", "b"], 16);
        let id2 = stable_id("test", &["a", "c"], 16);
        assert_ne!(id1, id2);
    }

    #[test]
    fn stable_id_differs_for_different_prefixes() {
        let id1 = stable_id("aaa", &["x"], 16);
        let id2 = stable_id("bbb", &["x"], 16);
        assert_ne!(id1, id2);
    }

    #[test]
    fn stable_id_distinguishes_part_boundaries_and_empty_parts() {
        assert_ne!(
            stable_id("test", &["a\nb"], 16),
            stable_id("test", &["a", "b"], 16)
        );
        assert_ne!(stable_id("test", &[], 16), stable_id("test", &[""], 16));
        assert_ne!(
            stable_id("test", &["", "a"], 16),
            stable_id("test", &["a", ""], 16)
        );
    }

    #[test]
    fn stable_id_is_deterministic_for_unicode() {
        assert_eq!(
            stable_id("test", &["東京🙂"], 16),
            stable_id("test", &["東京🙂"], 16)
        );
    }

    #[test]
    fn ulid_is_unique() {
        let ids: Vec<_> = (0..100).map(|_| ulid()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 100);
    }

    #[test]
    fn sha256_text_is_deterministic() {
        let h1 = sha256_text("hello world");
        let h2 = sha256_text("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn sha256_text_differs_for_different_inputs() {
        let h1 = sha256_text("hello");
        let h2 = sha256_text("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn claim_id_begins_with_clm() {
        let id = claim_id("src1", "sp1", "some claim text");
        assert!(id.starts_with("clm_"));
    }

    #[test]
    fn evidence_bundle_id_begins_with_evb() {
        let id = evidence_bundle_id("clm_abc123");
        assert!(id.starts_with("evb_"));
    }

    #[test]
    fn support_judgment_id_begins_with_sup() {
        let id = support_judgment_id("clm_abc123", "supported");
        assert!(id.starts_with("sup_"));
    }

    #[test]
    fn contradiction_id_is_ordered_and_deterministic() {
        let id1 = contradiction_id("clm_a", "clm_b", "exact");
        let id2 = contradiction_id("clm_b", "clm_a", "exact");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("ctr_"));
    }

    #[test]
    fn normalize_text_collapse_whitespace() {
        let input = "  hello    world  \n\n  foo  ";
        let normalized = normalize_text(input);
        assert_eq!(normalized, "hello world foo");
    }
}
