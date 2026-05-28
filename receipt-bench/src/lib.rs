//! receipt-bench: Replayable benchmark substrate.
//!
//! Captures structured receipts timestamped and keyed to commit hash + machine fingerprint,
//! enabling reproducible diffs between benchmark runs.

mod error;
mod fingerprint;
mod receipt;
mod suite;

pub use error::BenchError;
pub use fingerprint::MachineFingerprint;
pub use receipt::{BenchmarkReceipt, BenchmarkResult, ReceiptDiff};
pub use suite::BenchmarkSuite;