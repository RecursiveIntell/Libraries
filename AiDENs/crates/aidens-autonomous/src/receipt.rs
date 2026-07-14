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
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Exact loop state committed by this receipt. Legacy receipts omit it and
    /// recover only the counters their schema can prove.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_state: Option<CommittedLoopStateV1>,
    /// SHA-256 hash of the previous receipt (chaining).
    pub previous_hash: String,
    /// SHA-256 hash of this receipt's content (excluding this field).
    pub receipt_hash: String,
}

/// Restart-safe state snapshot committed with a cycle receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommittedLoopStateV1 {
    pub iteration: usize,
    pub gaps_detected: usize,
    pub tasks_generated: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub facts_captured: usize,
    pub facts_rejected: usize,
    pub consecutive_failures: usize,
    pub current_job: Option<String>,
    pub last_error: Option<String>,
    pub safe_mode: bool,
    pub strictness: String,
    pub cycle_mode: LoopMode,
    pub proof_debt_outstanding: usize,
    pub domains_explored: Vec<String>,
    pub saturated_domains: Vec<String>,
}

/// Snapshot of viscosity signal for receipt purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViscositySignalSnapshot {
    pub failure_rate: f64,
    pub drift_rate: f64,
    pub ambiguity_score: f64,
    pub contradiction_density: f64,
    pub effective_viscosity: f64,
}

impl CycleReceiptV1 {
    /// Recompute this receipt's SHA-256 hash from every field except
    /// [`Self::receipt_hash`]. This does not depend on emitter state.
    pub fn compute_hash(&self) -> String {
        let mut material = b"aidens.autonomous.cycle-receipt.v1\0".to_vec();

        append_u64(&mut material, self.iteration as u64);
        append_string(&mut material, &self.timestamp);
        append_u64(&mut material, self.gaps_detected as u64);
        append_u64(&mut material, self.tasks_executed as u64);
        append_u64(&mut material, self.facts_captured as u64);
        append_u64(&mut material, self.facts_rejected as u64);
        append_u64(&mut material, self.facts_quarantined as u64);
        append_viscosity_signal(&mut material, self.viscosity_signal.as_ref());
        append_string(&mut material, &self.strictness);
        append_u64(&mut material, self.proof_debt_outstanding as u64);
        append_u64(&mut material, self.proof_debt_total_incurred as u64);
        material.push(match self.mode {
            LoopMode::Additive => 0,
            LoopMode::Subtractive => 1,
        });
        append_strings(&mut material, &self.domains_explored);
        append_strings(&mut material, &self.saturated_domains);
        append_strings(&mut material, &self.errors);
        append_string(&mut material, &self.previous_hash);
        if let Some(state) = &self.committed_state {
            material.extend_from_slice(b"\0committed-state-v1\0");
            append_committed_state(&mut material, state);
        }

        format!("{:x}", Sha256::digest(material))
    }

    /// Return whether `receipt_hash` matches the deterministic receipt content.
    pub fn verify_hash(&self) -> bool {
        self.receipt_hash == self.compute_hash()
    }
}

fn append_optional_string(material: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            material.push(1);
            append_string(material, value);
        }
        None => material.push(0),
    }
}

fn append_committed_state(material: &mut Vec<u8>, state: &CommittedLoopStateV1) {
    append_u64(material, state.iteration as u64);
    append_u64(material, state.gaps_detected as u64);
    append_u64(material, state.tasks_generated as u64);
    append_u64(material, state.tasks_completed as u64);
    append_u64(material, state.tasks_failed as u64);
    append_u64(material, state.facts_captured as u64);
    append_u64(material, state.facts_rejected as u64);
    append_u64(material, state.consecutive_failures as u64);
    append_optional_string(material, state.current_job.as_deref());
    append_optional_string(material, state.last_error.as_deref());
    material.push(u8::from(state.safe_mode));
    append_string(material, &state.strictness);
    material.push(match state.cycle_mode {
        LoopMode::Additive => 0,
        LoopMode::Subtractive => 1,
    });
    append_u64(material, state.proof_debt_outstanding as u64);
    append_strings(material, &state.domains_explored);
    append_strings(material, &state.saturated_domains);
}

fn append_u64(material: &mut Vec<u8>, value: u64) {
    material.extend_from_slice(&value.to_be_bytes());
}

fn append_f64(material: &mut Vec<u8>, value: f64) {
    material.extend_from_slice(&value.to_bits().to_be_bytes());
}

fn append_string(material: &mut Vec<u8>, value: &str) {
    append_u64(material, value.len() as u64);
    material.extend_from_slice(value.as_bytes());
}

fn append_strings(material: &mut Vec<u8>, values: &[String]) {
    append_u64(material, values.len() as u64);
    for value in values {
        append_string(material, value);
    }
}

fn append_viscosity_signal(
    material: &mut Vec<u8>,
    viscosity_signal: Option<&ViscositySignalSnapshot>,
) {
    match viscosity_signal {
        None => material.push(0),
        Some(signal) => {
            material.push(1);
            append_f64(material, signal.failure_rate);
            append_f64(material, signal.drift_rate);
            append_f64(material, signal.ambiguity_score);
            append_f64(material, signal.contradiction_density);
            append_f64(material, signal.effective_viscosity);
        }
    }
}

// ---------------------------------------------------------------------------
// Receipt emitter
// ---------------------------------------------------------------------------

/// Complete material input for one autonomous-loop cycle receipt.
#[derive(Debug, Clone)]
pub struct CycleReceiptInputV1 {
    pub iteration: usize,
    pub gaps_detected: usize,
    pub tasks_executed: usize,
    pub facts_captured: usize,
    pub facts_rejected: usize,
    pub facts_quarantined: usize,
    pub viscosity_signal: Option<ViscositySignalSnapshot>,
    pub strictness: String,
    pub proof_debt_outstanding: usize,
    pub proof_debt_total_incurred: usize,
    pub mode: LoopMode,
    pub domains_explored: Vec<String>,
    pub saturated_domains: Vec<String>,
    pub errors: Vec<String>,
}

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

    /// Emit a cycle receipt from complete typed input.
    pub fn emit(&mut self, input: CycleReceiptInputV1) -> CycleReceiptV1 {
        let timestamp = Utc::now().to_rfc3339();
        let previous_hash = self.previous_hash.clone();

        let mut receipt = CycleReceiptV1 {
            iteration: input.iteration,
            timestamp,
            gaps_detected: input.gaps_detected,
            tasks_executed: input.tasks_executed,
            facts_captured: input.facts_captured,
            facts_rejected: input.facts_rejected,
            facts_quarantined: input.facts_quarantined,
            viscosity_signal: input.viscosity_signal,
            strictness: input.strictness,
            proof_debt_outstanding: input.proof_debt_outstanding,
            proof_debt_total_incurred: input.proof_debt_total_incurred,
            mode: input.mode,
            domains_explored: input.domains_explored,
            saturated_domains: input.saturated_domains,
            errors: input.errors,
            committed_state: None,
            previous_hash,
            receipt_hash: String::new(),
        };
        receipt.receipt_hash = receipt.compute_hash();

        // Update the chain only after the complete receipt hash is computed.
        self.previous_hash = receipt.receipt_hash.clone();

        receipt
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

/// Append-only durable ledger for emitted cycle receipts.
#[derive(Debug, Default)]
pub struct ReceiptLedger {
    emitter: ReceiptEmitter,
    history: Vec<CycleReceiptV1>,
    path: Option<PathBuf>,
    persistence_error: Option<String>,
}

impl ReceiptLedger {
    /// Create an in-memory ledger. Production loop construction uses
    /// [`Self::open`] so cycle commits cannot silently lose durability.
    pub fn new() -> Self {
        Self::default()
    }

    /// Open or create an append-only JSON-lines receipt store, verify every
    /// committed receipt, and restore the chain head.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        OpenOptions::new().create(true).append(true).open(&path)?;

        let file = File::open(&path)?;
        let mut history = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let receipt: CycleReceiptV1 = serde_json::from_str(&line).map_err(|error| {
                anyhow::anyhow!("invalid receipt JSON at line {}: {error}", index + 1)
            })?;
            history.push(receipt);
        }

        verify_receipts(&history)?;
        let previous_hash = match history.last() {
            Some(receipt) => receipt.receipt_hash.clone(),
            None => String::new(),
        };
        Ok(Self {
            emitter: ReceiptEmitter { previous_hash },
            history,
            path: Some(path),
            persistence_error: None,
        })
    }

    /// Construct a ledger that reports a prior store-open failure on every
    /// durable emit instead of degrading to process memory.
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            persistence_error: Some(error.into()),
            ..Self::default()
        }
    }

    /// Emit, chain, and retain one receipt in process-owned memory.
    pub fn emit(&mut self, input: CycleReceiptInputV1) -> CycleReceiptV1 {
        let receipt = self.emitter.emit(input);
        self.history.push(receipt.clone());
        receipt
    }

    /// Emit and fsync a receipt before advancing the in-memory chain head.
    pub fn emit_durable(&mut self, input: CycleReceiptInputV1) -> anyhow::Result<CycleReceiptV1> {
        self.emit_durable_with_state(input, None)
    }

    /// Emit, bind an exact restart state, and fsync the receipt before exposing
    /// the new chain head.
    pub fn emit_durable_with_state(
        &mut self,
        input: CycleReceiptInputV1,
        committed_state: Option<CommittedLoopStateV1>,
    ) -> anyhow::Result<CycleReceiptV1> {
        if let Some(error) = &self.persistence_error {
            return Err(anyhow::anyhow!("receipt store unavailable: {error}"));
        }
        let path = self.path.as_ref().ok_or_else(|| {
            anyhow::anyhow!("durable receipt emission requires ReceiptLedger::open")
        })?;

        let mut receipt = self.emitter.emit(input);
        receipt.committed_state = committed_state;
        receipt.receipt_hash = receipt.compute_hash();
        self.emitter.previous_hash = receipt.receipt_hash.clone();
        let encoded = serde_json::to_vec(&receipt)?;
        let append_result = (|| -> anyhow::Result<()> {
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            file.write_all(&encoded)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(error) = append_result {
            self.emitter.previous_hash = receipt.previous_hash.clone();
            return Err(error);
        }
        self.history.push(receipt.clone());
        Ok(receipt)
    }

    /// Verify hashes, genesis, and every previous-hash link offline.
    pub fn verify_chain(&self) -> anyhow::Result<()> {
        verify_receipts(&self.history)
    }

    /// Last durably committed receipt, if one exists.
    pub fn last_committed(&self) -> Option<&CycleReceiptV1> {
        self.history.last()
    }

    /// Inspect retained receipts without permitting mutation.
    pub fn history(&self) -> &[CycleReceiptV1] {
        &self.history
    }
}

fn verify_receipts(receipts: &[CycleReceiptV1]) -> anyhow::Result<()> {
    let mut previous_hash = "";
    for (index, receipt) in receipts.iter().enumerate() {
        if !receipt.verify_hash() {
            return Err(anyhow::anyhow!(
                "receipt {} has an invalid content hash",
                index + 1
            ));
        }
        if receipt.previous_hash != previous_hash {
            return Err(anyhow::anyhow!(
                "receipt {} does not link to the previous receipt",
                index + 1
            ));
        }
        previous_hash = &receipt.receipt_hash;
    }
    Ok(())
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
        let r1 = emitter.emit(CycleReceiptInputV1 {
            iteration: 1,
            gaps_detected: 5,
            tasks_executed: 3,
            facts_captured: 2,
            facts_rejected: 1,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "normal".into(),
            proof_debt_outstanding: 2,
            proof_debt_total_incurred: 4,
            mode: LoopMode::Additive,
            domains_explored: vec!["projects".into()],
            saturated_domains: vec![],
            errors: vec![],
        });
        let r2 = emitter.emit(CycleReceiptInputV1 {
            iteration: 2,
            gaps_detected: 3,
            tasks_executed: 2,
            facts_captured: 1,
            facts_rejected: 0,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "normal".into(),
            proof_debt_outstanding: 3,
            proof_debt_total_incurred: 6,
            mode: LoopMode::Additive,
            domains_explored: vec!["research".into()],
            saturated_domains: vec![],
            errors: vec![],
        });
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
        let r1 = e1.emit(CycleReceiptInputV1 {
            iteration: 1,
            gaps_detected: 0,
            tasks_executed: 0,
            facts_captured: 0,
            facts_rejected: 0,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "fast".into(),
            proof_debt_outstanding: 0,
            proof_debt_total_incurred: 0,
            mode: LoopMode::Additive,
            domains_explored: vec![],
            saturated_domains: vec![],
            errors: vec![],
        });
        let r2 = e2.emit(CycleReceiptInputV1 {
            iteration: 1,
            gaps_detected: 0,
            tasks_executed: 0,
            facts_captured: 0,
            facts_rejected: 0,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "fast".into(),
            proof_debt_outstanding: 0,
            proof_debt_total_incurred: 0,
            mode: LoopMode::Additive,
            domains_explored: vec![],
            saturated_domains: vec![],
            errors: vec![],
        });
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

    fn receipt_with_viscosity() -> CycleReceiptV1 {
        let mut emitter = ReceiptEmitter::new();
        emitter.emit(CycleReceiptInputV1 {
            iteration: 7,
            gaps_detected: 1,
            tasks_executed: 2,
            facts_captured: 3,
            facts_rejected: 4,
            facts_quarantined: 5,
            viscosity_signal: Some(ViscositySignalSnapshot {
                failure_rate: 0.1,
                drift_rate: 0.2,
                ambiguity_score: 0.3,
                contradiction_density: 0.4,
                effective_viscosity: 0.5,
            }),
            strictness: "strict|日本語".into(),
            proof_debt_outstanding: 6,
            proof_debt_total_incurred: 7,
            mode: LoopMode::Subtractive,
            domains_explored: vec!["naïve|東京".into(), "a".into(), "b|c".into()],
            saturated_domains: vec!["saturated|Δ".into()],
            errors: vec!["error|🚫".into()],
        })
    }

    #[test]
    fn receipt_hash_rejects_every_viscosity_scalar_change() {
        let receipt = receipt_with_viscosity();
        assert!(receipt.verify_hash());

        let mut changed = receipt.clone();
        changed.viscosity_signal.as_mut().unwrap().failure_rate = 0.11;
        assert_ne!(changed.compute_hash(), receipt.receipt_hash);
        assert!(!changed.verify_hash());

        let mut changed = receipt.clone();
        changed.viscosity_signal.as_mut().unwrap().drift_rate = 0.21;
        assert_ne!(changed.compute_hash(), receipt.receipt_hash);
        assert!(!changed.verify_hash());

        let mut changed = receipt.clone();
        changed.viscosity_signal.as_mut().unwrap().ambiguity_score = 0.31;
        assert_ne!(changed.compute_hash(), receipt.receipt_hash);
        assert!(!changed.verify_hash());

        let mut changed = receipt.clone();
        changed
            .viscosity_signal
            .as_mut()
            .unwrap()
            .contradiction_density = 0.41;
        assert_ne!(changed.compute_hash(), receipt.receipt_hash);
        assert!(!changed.verify_hash());

        let mut changed = receipt.clone();
        changed
            .viscosity_signal
            .as_mut()
            .unwrap()
            .effective_viscosity = 0.51;
        assert_ne!(changed.compute_hash(), receipt.receipt_hash);
        assert!(!changed.verify_hash());
    }

    #[test]
    fn receipt_hash_is_deterministic_for_unicode_and_unambiguous_vectors() {
        let receipt = receipt_with_viscosity();
        assert_eq!(receipt.compute_hash(), receipt.compute_hash());
        assert!(receipt.verify_hash());

        let mut different_vector_boundaries = receipt.clone();
        different_vector_boundaries.domains_explored =
            vec!["naïve|東京".into(), "a|b".into(), "c".into()];
        assert_ne!(
            different_vector_boundaries.compute_hash(),
            receipt.compute_hash(),
            "vector element boundaries must be represented in hash material"
        );
        assert!(!different_vector_boundaries.verify_hash());
    }

    #[test]
    fn receipt_ledger_retains_emitted_history_for_read_only_inspection() {
        let mut ledger = ReceiptLedger::new();

        let first = ledger.emit(CycleReceiptInputV1 {
            iteration: 1,
            gaps_detected: 2,
            tasks_executed: 0,
            facts_captured: 0,
            facts_rejected: 0,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "normal".into(),
            proof_debt_outstanding: 0,
            proof_debt_total_incurred: 0,
            mode: LoopMode::Additive,
            domains_explored: vec!["project-a".into()],
            saturated_domains: vec![],
            errors: vec![],
        });
        let second = ledger.emit(CycleReceiptInputV1 {
            iteration: 2,
            gaps_detected: 0,
            tasks_executed: 0,
            facts_captured: 0,
            facts_rejected: 0,
            facts_quarantined: 0,
            viscosity_signal: None,
            strictness: "normal".into(),
            proof_debt_outstanding: 0,
            proof_debt_total_incurred: 0,
            mode: LoopMode::Subtractive,
            domains_explored: vec![],
            saturated_domains: vec![],
            errors: vec!["subtractive cycle failed: unavailable".into()],
        });

        assert_eq!(ledger.history(), &[first.clone(), second.clone()]);
        assert_eq!(second.previous_hash, first.receipt_hash);
    }

    #[test]
    fn durable_ledger_verifies_and_recovers_after_restart() {
        let path = std::env::temp_dir().join(format!(
            "aidens-autonomous-receipts-{}-{}.jsonl",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let _ = std::fs::remove_file(&path);

        {
            let mut ledger = ReceiptLedger::open(&path).expect("open durable ledger");
            for iteration in 1..=3 {
                ledger
                    .emit_durable(CycleReceiptInputV1 {
                        iteration,
                        gaps_detected: iteration,
                        tasks_executed: 1,
                        facts_captured: 1,
                        facts_rejected: 0,
                        facts_quarantined: 0,
                        viscosity_signal: None,
                        strictness: "normal".into(),
                        proof_debt_outstanding: 0,
                        proof_debt_total_incurred: iteration,
                        mode: LoopMode::Additive,
                        domains_explored: vec![],
                        saturated_domains: vec![],
                        errors: vec![],
                    })
                    .expect("persist receipt");
            }
            ledger.verify_chain().expect("verify live chain");
        }

        let recovered = ReceiptLedger::open(&path).expect("recover durable ledger");
        recovered.verify_chain().expect("verify recovered chain");
        assert_eq!(recovered.history().len(), 3);
        assert_eq!(recovered.last_committed().map(|r| r.iteration), Some(3));

        let _ = std::fs::remove_file(path);
    }
}
