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
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub duration_ms: u128,
    pub passed: bool,
}
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
