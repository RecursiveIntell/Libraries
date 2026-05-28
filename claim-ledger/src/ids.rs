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
            || !result.last().map_or(false, |last: &char| {
                last.is_whitespace() && c.is_whitespace()
            })
        {
            result.push(c);
        } else if !c.is_whitespace() {
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
/// Uses SHA-256 of the joined parts, truncated to `length` characters.
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
    let input = parts.join("\n");
    let hash = Sha256::digest(input.as_bytes());
    let hex = hex::encode(hash);
    let truncated = &hex[..length.min(hex.len())];
    format!("{}_{}", prefix, truncated)
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
        &[operation]
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
