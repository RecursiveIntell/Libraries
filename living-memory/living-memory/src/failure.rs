//! Failure classification, recording, and retry semantics.

use serde::{Deserialize, Serialize};

/// Classification of failures for structured handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    /// Baseline execution timed out.
    BaselineTimeout,
    /// Patched execution crashed after baseline succeeded.
    PatchedCrash,
    /// Duplicate CEA replay attempted.
    DuplicateCeaReplay,
    /// Database was busy during insert.
    DbBusy,
    /// Crash after export receipt but before memory ingest.
    PostReceiptCrash,
    /// Semantic-memory ingest unavailable.
    MemoryIngestUnavailable,
    /// Partial verification completion.
    PartialVerification,
    /// Migration was interrupted.
    MigrationInterrupted,
    /// Workspace exceeded size limit.
    WorkspaceSizeExceeded,
    /// Unknown or unclassified failure.
    Other,
}

/// A persisted failure record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub failure_id: String,
    pub run_id: String,
    pub class: FailureClass,
    pub message: String,
    /// Phase during which the failure occurred.
    pub phase: String,
    /// Whether this failure is retriable.
    pub retriable: bool,
    /// Number of retries attempted so far.
    pub retry_count: u32,
    pub occurred_at: String,
}

impl FailureClass {
    /// Whether this failure class is generally retriable.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::DbBusy
                | Self::MemoryIngestUnavailable
                | Self::PostReceiptCrash
                | Self::BaselineTimeout
        )
    }

    /// Classify an error into a FailureClass.
    pub fn from_error(error: &crate::error::ForgeError) -> Self {
        match error {
            crate::error::ForgeError::CommandTimeout { .. } => Self::BaselineTimeout,
            crate::error::ForgeError::Database(e) => {
                if e.to_string().contains("database is locked") {
                    Self::DbBusy
                } else {
                    Self::Other
                }
            }
            _ => Self::Other,
        }
    }
}
