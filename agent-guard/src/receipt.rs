//! Security receipt types.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a security decision made by the control plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDecision {
    /// Unique identifier for this decision.
    pub decision_id: String,
    /// The agent or process this decision applies to.
    pub subject: Subject,
    /// Action being attempted.
    pub action: Action,
    /// Whether the action was allowed or denied.
    pub allowed: bool,
    /// Reason for the decision.
    pub reason: String,
    /// Timestamp of the decision.
    pub timestamp: DateTime<Utc>,
    /// Applied security mechanisms.
    pub mechanisms: Vec<SecurityMechanism>,
}

/// The entity a security decision applies to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// PID of the subject (if process).
    pub pid: Option<u32>,
    /// Name of the subject.
    pub name: String,
    /// Optional cgroup path.
    pub cgroup_path: Option<String>,
}

/// An action being attempted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Type of action.
    pub action_type: ActionType,
    /// Resource being accessed.
    pub resource: String,
    /// Additional metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Types of actions that can be secured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    FileRead,
    FileWrite,
    FileExecute,
    NetworkConnect,
    NetworkBind,
    ProcessSpawn,
    SystemCall,
    CgroupModify,
}

/// A security mechanism that was applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMechanism {
    /// BPF LSM hook.
    BpfLsm { program: String },
    /// cgroup v2 restriction.
    CgroupV2 { path: String },
    /// Landlock rule.
    Landlock { ruleset_id: u64 },
    /// seccomp filter.
    Seccomp { filter_id: u32 },
    /// eBPF program.
    Ebpf { program: String },
}
