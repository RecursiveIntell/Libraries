use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRun {
    pub run_id: String,
    pub trace: String,
    pub agent: AgentIdentity,
    pub repository: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<RunVerdict>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub provider: String,
    pub model: Option<String>,
    pub version: Option<String>,
    pub invocation_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySnapshot {
    pub path: String,
    pub baseline_sha: String,
    pub final_sha: String,
    pub is_clean: bool,
    pub diff_stat: String,
    pub diff: String,
    pub status: String,
    pub diff_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentClaim {
    pub id: String,
    pub text: String,
    pub normalized_predicate: String,
    pub source_quote: String,
    pub source_location: Option<String>,
    pub status: ClaimStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimStatus {
    Verified,
    Partial,
    Unsupported,
    Contradicted,
    NotChecked,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceKind {
    GitDiff,
    GitStatus,
    CommandResult,
    Transcript,
    FileChange,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: String,
    pub kind: EvidenceKind,
    pub source: String,
    pub digest: String,
    pub summary: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub command: String,
    pub outcome: CommandOutcome,
    pub policy: ExecutionPolicy,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub duration_ms: u128,
    pub passed: bool,
    #[serde(default)]
    pub redaction_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandOutcome {
    Completed,
    Failed,
    TimedOut,
    LaunchFailed,
    CaptureFailed,
}

impl CommandOutcome {
    pub fn to_v2_outcome(self) -> crate::v2::CommandOutcomeV2 {
        match self {
            Self::Completed => crate::v2::CommandOutcomeV2::Passed,
            Self::Failed => crate::v2::CommandOutcomeV2::Failed,
            Self::TimedOut => crate::v2::CommandOutcomeV2::TimedOut,
            Self::LaunchFailed | Self::CaptureFailed => crate::v2::CommandOutcomeV2::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPolicy {
    pub deadline_ms: u64,
    pub stdout_cap_bytes: usize,
    pub stderr_cap_bytes: usize,
}

/// Fixed execution policy; it never reads ambient configuration.
pub const DEFAULT_EXECUTION_POLICY: ExecutionPolicy = ExecutionPolicy {
    deadline_ms: 30_000,
    stdout_cap_bytes: 64 * 1024,
    stderr_cap_bytes: 64 * 1024,
};

/// Timeout cancellation terminates only the spawned direct child. It does not
/// prove descendant process cleanup, sandboxing, network denial, or rollback
/// of command side effects.
pub const TIMEOUT_LIMITATION: &str =
    "timeout cancellation terminates only the direct child; descendant cleanup is not guaranteed";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub run_id: String,
    pub verdict: RunVerdict,
    pub claims: Vec<AgentClaim>,
    pub checks: Vec<CheckResult>,
    pub diff: String,
    pub evidence_manifest: Vec<EvidenceItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunVerdict {
    Clean,
    Partial,
    Failed,
    Error,
}
