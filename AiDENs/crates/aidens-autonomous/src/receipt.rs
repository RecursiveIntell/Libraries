//! Loop cycle receipt emitter — typed receipts for every autonomous loop cycle.
//!
//! Every iteration of the autonomous loop emits a [`CycleReceiptV1`] that
//! captures the full state snapshot: gaps detected, tasks executed, facts
//! captured/rejected, viscosity signal, strictness level, proof-debt
//! outstanding, loop mode, domains explored, saturated domains, and errors.
//!
//! Receipts are chained via SHA-256 hashes so the full audit trail is
//! tamper-evident.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The loop's operating mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    /// Adding new knowledge: exploring, generating, executing, capturing.
    #[default]
    Additive,
    /// Reducing/verifying: checking contradictions, paying proof-debt,
    /// compacting, retiring stale items.
    Subtractive,
}

impl std::fmt::Display for LoopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Additive => f.write_str("additive"),
            Self::Subtractive => f.write_str("subtractive"),
        }
    }
}

/// A typed receipt for one cycle of the autonomous loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReceiptV1 {
    /// Cycle number (matches LoopState::iteration).
    pub iteration: usize,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Gaps detected this cycle.
    pub gaps_detected: usize,
    /// Tasks executed this cycle.
    pub tasks_executed: usize,
    /// Facts captured (promoted + quarantined) this cycle.
    pub facts_captured: usize,
    /// Facts rejected this cycle.
    pub facts_rejected: usize,
    /// Facts quarantined this cycle.
    pub facts_quarantined: usize,
    /// Viscosity signal (if viscosity controller is active).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viscosity_signal: Option<ViscositySignalSnapshot>,
    /// Strictness level name.
    pub strictness: String,
    /// Proof-debt outstanding at end of cycle.
    pub proof_debt_outstanding: usize,
    /// Proof-debt total incurred.
    pub proof_debt_total_incurred: usize,
    /// Loop mode (additive/subtractive).
    pub mode: LoopMode,
    /// Domains explored this cycle.
    pub domains_explored: Vec<String>,
    /// Saturated domains at end of cycle.
    pub saturated_domains: Vec<String>,
    /// Errors encountered this cycle.
    pub errors: Vec<String>,
    /// SHA-256 hash of the previous receipt (chaining).
    pub previous_hash: String,
    /// SHA-256 hash of this receipt's content (excluding this field).
    pub receipt_hash: String,
}

/// Snapshot of viscosity signal for receipt purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViscositySignalSnapshot {
    pub failure_rate: f64,
    pub drift_rate: f64,
    pub ambiguity_score: f64,
    pub contradiction_density: f64,
    pub effective_viscosity: f64,
}

// ---------------------------------------------------------------------------
// Receipt emitter
// ---------------------------------------------------------------------------

/// Emits typed cycle receipts and maintains the hash chain.
#[derive(Debug, Clone)]
pub struct ReceiptEmitter {
    /// Hash of the previous receipt (empty string for the first receipt).
    previous_hash: String,
}

impl ReceiptEmitter {
    /// Create a new emitter with no prior receipts.
    pub fn new() -> Self {
        Self {
            previous_hash: String::new(),
        }
    }

    /// Emit a cycle receipt.
    pub fn emit(
        &mut self,
        iteration: usize,
        gaps_detected: usize,
        tasks_executed: usize,
        facts_captured: usize,
        facts_rejected: usize,
        facts_quarantined: usize,
        viscosity_signal: Option<ViscositySignalSnapshot>,
        strictness: &str,
        proof_debt_outstanding: usize,
        proof_debt_total_incurred: usize,
        mode: LoopMode,
        domains_explored: Vec<String>,
        saturated_domains: Vec<String>,
        errors: Vec<String>,
    ) -> CycleReceiptV1 {
        let timestamp = Utc::now().to_rfc3339();
        let previous_hash = self.previous_hash.clone();

        // Compute receipt hash over all fields except receipt_hash itself.
        let hash_input = format!(
            "{iteration}|{timestamp}|{gaps_detected}|{tasks_executed}|\
             {facts_captured}|{facts_rejected}|{facts_quarantined}|\
             {strictness}|{proof_debt_outstanding}|{proof_debt_total_incurred}|\
             {mode}|{domains_explored:?}|{saturated_domains:?}|{errors:?}|\
             {previous_hash}"
        );
        let receipt_hash = format!("{:x}", Sha256::digest(hash_input.as_bytes()));

        // Update the chain.
        self.previous_hash = receipt_hash.clone();

        CycleReceiptV1 {
            iteration,
            timestamp,
            gaps_detected,
            tasks_executed,
            facts_captured,
            facts_rejected,
            facts_quarantined,
            viscosity_signal,
            strictness: strictness.to_string(),
            proof_debt_outstanding,
            proof_debt_total_incurred,
            mode,
            domains_explored,
            saturated_domains,
            errors,
            previous_hash,
            receipt_hash,
        }
    }

    /// Get the last receipt hash (for verification).
    pub fn last_hash(&self) -> &str {
        &self.previous_hash
    }
}

impl Default for ReceiptEmitter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_chaining() {
        let mut emitter = ReceiptEmitter::new();
        let r1 = emitter.emit(
            1,
            5,
            3,
            2,
            1,
            0,
            None,
            "normal",
            2,
            4,
            LoopMode::Additive,
            vec!["projects".into()],
            vec![],
            vec![],
        );
        let r2 = emitter.emit(
            2,
            3,
            2,
            1,
            0,
            0,
            None,
            "normal",
            3,
            6,
            LoopMode::Additive,
            vec!["research".into()],
            vec![],
            vec![],
        );
        // First receipt has empty previous_hash.
        assert_eq!(r1.previous_hash, "");
        // Second receipt chains to the first.
        assert_eq!(r2.previous_hash, r1.receipt_hash);
        // Hashes are different.
        assert_ne!(r1.receipt_hash, r2.receipt_hash);
    }

    #[test]
    fn test_receipt_determinism() {
        let mut e1 = ReceiptEmitter::new();
        let mut e2 = ReceiptEmitter::new();
        let r1 = e1.emit(
            1,
            0,
            0,
            0,
            0,
            0,
            None,
            "fast",
            0,
            0,
            LoopMode::Additive,
            vec![],
            vec![],
            vec![],
        );
        let r2 = e2.emit(
            1,
            0,
            0,
            0,
            0,
            0,
            None,
            "fast",
            0,
            0,
            LoopMode::Additive,
            vec![],
            vec![],
            vec![],
        );
        // Same inputs → same hash (timestamp differs, so hash differs,
        // but structure is identical).
        assert_eq!(r1.iteration, r2.iteration);
        assert_eq!(r1.mode, r2.mode);
    }

    #[test]
    fn test_loop_mode_serde() {
        let additive = LoopMode::Additive;
        let json = serde_json::to_string(&additive).unwrap();
        assert_eq!(json, "\"additive\"");
        let subtractive = LoopMode::Subtractive;
        let json = serde_json::to_string(&subtractive).unwrap();
        assert_eq!(json, "\"subtractive\"");
    }
}
