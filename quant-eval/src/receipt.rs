//! Benchmark receipt — timestamped provenance record for benchmark runs.
//!
//! Results are compatible with `receipt-bench::BenchmarkReceipt`.

use crate::error::QuantEvalError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A benchmark receipt captures the results and context of a single benchmark run.
///
/// Receipts are keyed to:
/// - A commit hash (git SHA)
/// - A machine fingerprint
/// - A timestamp (UTC)
///
/// This enables reproducible diffs between runs across different machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReceipt {
    /// Timestamp of the benchmark run (UTC)
    pub timestamp: DateTime<Utc>,

    /// Git commit SHA of the code being benchmarked
    pub commit_hash: String,

    /// Machine fingerprint (hostname + username + arch + OS + cpu count + machine-id)
    pub machine_fingerprint: String,

    /// Benchmark results indexed by name
    pub results: Vec<BenchmarkResult>,

    /// Optional note about the run environment
    pub note: Option<String>,
}

/// A single benchmark result within a receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark (e.g., "semantic_search", "compression_round_trip")
    pub name: String,

    /// Number of iterations performed
    pub iterations: u64,

    /// Total elapsed time in nanoseconds
    pub elapsed_ns: u64,

    /// Computed nanoseconds per iteration
    pub ns_per_iter: u64,

    /// Optional throughput metric (items per second)
    pub throughput: Option<f64>,

    /// Optional error message if benchmark failed
    pub error: Option<String>,
}

/// Comparison result between two benchmark receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptDiff {
    /// Receipt hash of the baseline (first) receipt
    pub baseline_hash: String,

    /// Receipt hash of the target (second) receipt
    pub target_hash: String,

    /// Timestamp of the diff computation
    pub computed_at: DateTime<Utc>,

    /// Per-benchmark differences
    pub benchmark_diffs: Vec<BenchmarkDiff>,
}

/// Individual benchmark difference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDiff {
    pub name: String,
    pub baseline_ns_per_iter: u64,
    pub target_ns_per_iter: u64,
    /// Difference in ns per iteration (target - baseline)
    pub diff_ns: i64,
    /// Percentage change (positive = slower, negative = faster)
    pub percent_change: f64,
}

impl BenchmarkReceipt {
    /// Create a new receipt with the current timestamp.
    pub fn new(commit_hash: String, machine_fingerprint: String) -> Self {
        Self {
            timestamp: Utc::now(),
            commit_hash,
            machine_fingerprint,
            results: Vec::new(),
            note: None,
        }
    }

    /// Create a new receipt from a fingerprint struct.
    pub fn with_fingerprint(commit_hash: String, fingerprint: &super::MachineFingerprint) -> Self {
        Self {
            timestamp: Utc::now(),
            commit_hash,
            machine_fingerprint: fingerprint.as_str().to_string(),
            results: Vec::new(),
            note: None,
        }
    }

    /// Add a benchmark result to this receipt.
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Compute a short hash of this receipt for quick comparison.
    ///
    /// Uses only the commit hash, machine fingerprint, and result names/values.
    /// Timestamp is excluded to allow comparing logically equivalent runs.
    pub fn receipt_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.commit_hash.as_bytes());
        hasher.update(self.machine_fingerprint.as_bytes());
        for result in &self.results {
            hasher.update(result.name.as_bytes());
            hasher.update(result.ns_per_iter.to_le_bytes());
        }
        hex::encode(&hasher.finalize()[..16])
    }

    /// Set an optional note about this run.
    pub fn set_note(&mut self, note: impl Into<String>) {
        self.note = Some(note.into());
    }

    /// Serialize this receipt to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, QuantEvalError> {
        serde_json::to_vec(self).map_err(|e| QuantEvalError::Serialization(e.to_string()))
    }

    /// Deserialize a receipt from JSON bytes.
    pub fn from_json(bytes: &[u8]) -> Result<Self, QuantEvalError> {
        serde_json::from_slice(bytes).map_err(|e| QuantEvalError::Serialization(e.to_string()))
    }
}

impl ReceiptDiff {
    /// Compare two receipts and produce a diff.
    pub fn compare(baseline: &BenchmarkReceipt, target: &BenchmarkReceipt) -> Self {
        let mut benchmark_diffs = Vec::new();

        // Index baseline results by name
        let baseline_map: std::collections::HashMap<_, _> = baseline
            .results
            .iter()
            .map(|r| (r.name.as_str(), r))
            .collect();

        for target_result in &target.results {
            if let Some(baseline_result) = baseline_map.get(target_result.name.as_str()) {
                let diff_ns = target_result.ns_per_iter as i64 - baseline_result.ns_per_iter as i64;
                let percent_change = if baseline_result.ns_per_iter > 0 {
                    (diff_ns as f64 / baseline_result.ns_per_iter as f64) * 100.0
                } else {
                    0.0
                };

                benchmark_diffs.push(BenchmarkDiff {
                    name: target_result.name.clone(),
                    baseline_ns_per_iter: baseline_result.ns_per_iter,
                    target_ns_per_iter: target_result.ns_per_iter,
                    diff_ns,
                    percent_change,
                });
            }
        }

        Self {
            baseline_hash: baseline.receipt_hash(),
            target_hash: target.receipt_hash(),
            computed_at: Utc::now(),
            benchmark_diffs,
        }
    }

    /// Serialize this diff to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, QuantEvalError> {
        serde_json::to_vec(self).map_err(|e| QuantEvalError::Serialization(e.to_string()))
    }

    /// Deserialize a diff from JSON bytes.
    pub fn from_json(bytes: &[u8]) -> Result<Self, QuantEvalError> {
        serde_json::from_slice(bytes).map_err(|e| QuantEvalError::Serialization(e.to_string()))
    }
}

// Simple hex encoder
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let bytes = data.as_ref();
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_fingerprint() -> String {
        "0".repeat(64)
    }

    #[test]
    fn test_receipt_creation() {
        let receipt = BenchmarkReceipt::new("abc123".to_string(), dummy_fingerprint());
        assert_eq!(receipt.commit_hash, "abc123");
        assert!(receipt.results.is_empty());
    }

    #[test]
    fn test_receipt_add_result() {
        let mut receipt = BenchmarkReceipt::new("abc123".to_string(), dummy_fingerprint());
        receipt.add_result(BenchmarkResult {
            name: "test_bench".to_string(),
            iterations: 1000,
            elapsed_ns: 1_000_000,
            ns_per_iter: 1000,
            throughput: Some(1_000_000.0),
            error: None,
        });
        assert_eq!(receipt.results.len(), 1);
        assert_eq!(receipt.results[0].name, "test_bench");
    }

    #[test]
    fn test_receipt_serialization() {
        let mut receipt = BenchmarkReceipt::new("abc123".to_string(), dummy_fingerprint());
        receipt.add_result(BenchmarkResult {
            name: "test_bench".to_string(),
            iterations: 1000,
            elapsed_ns: 1_000_000,
            ns_per_iter: 1000,
            throughput: Some(1_000_000.0),
            error: None,
        });

        let json = receipt.to_json().expect("serialization should work");
        let deserialized = BenchmarkReceipt::from_json(&json).expect("deserialization should work");
        assert_eq!(deserialized.commit_hash, receipt.commit_hash);
        assert_eq!(deserialized.results.len(), 1);
    }

    #[test]
    fn test_diff_comparison() {
        let mut baseline = BenchmarkReceipt::new("abc123".to_string(), dummy_fingerprint());
        baseline.add_result(BenchmarkResult {
            name: "test_bench".to_string(),
            iterations: 1000,
            elapsed_ns: 1_000_000,
            ns_per_iter: 1000,
            throughput: Some(1_000_000.0),
            error: None,
        });

        let mut target = BenchmarkReceipt::new("def456".to_string(), dummy_fingerprint());
        target.add_result(BenchmarkResult {
            name: "test_bench".to_string(),
            iterations: 1000,
            elapsed_ns: 1_200_000,
            ns_per_iter: 1200,
            throughput: Some(833_333.0),
            error: None,
        });

        let diff = ReceiptDiff::compare(&baseline, &target);
        assert_eq!(diff.benchmark_diffs.len(), 1);
        assert_eq!(diff.benchmark_diffs[0].name, "test_bench");
        assert_eq!(diff.benchmark_diffs[0].diff_ns, 200);
        assert!((diff.benchmark_diffs[0].percent_change - 20.0).abs() < 0.01);
    }
}
