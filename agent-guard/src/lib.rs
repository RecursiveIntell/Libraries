//! AgentGuard - Linux control plane for AI agent security.
//!
//! This crate provides security mechanisms for AI agents running on Linux.
//! It supports BPF LSM, cgroup v2, Landlock, seccomp, and eBPF.
//!
//! # Linux Only
//!
//! This crate is only available on Linux. It will not compile on other systems.
//!
//! # Example
//!
//! ```ignore
//! use agent_guard::{AgentGuard, Subject, Action, ActionType};
//!
//! let mut guard = AgentGuard::new();
//! guard.initialize()?;
//! ```

#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

mod control_plane;
mod error;
mod receipt;

pub use control_plane::ControlPlane;
pub use error::{Error, Result};
pub use receipt::{
    Action, ActionType, SecurityDecision, SecurityMechanism, Subject,
};

use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};

/// AgentGuard security manager.
///
/// This is the main entry point for the agent-guard crate. It provides
/// a unified interface to Linux security mechanisms.
///
/// # Platform Support
///
/// This struct is only available on Linux. On other platforms, all methods
/// will return errors indicating the platform is unsupported.
#[derive(Debug)]
pub struct AgentGuard {
    initialized: AtomicBool,
}

impl AgentGuard {
    /// Create a new AgentGuard instance.
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
        }
    }

    /// Create a new AgentGuard instance (non-Linux stub).
    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the security control plane.
    ///
    /// On Linux, this sets up the security mechanisms.
    /// On other platforms, this returns an error.
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // Linux-specific initialization would go here
            // For now, mark as initialized
            self.initialized.store(true, Ordering::SeqCst);
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::InvalidConfig(
                "AgentGuard is only available on Linux".to_string(),
            ))
        }
    }

    /// Check if the guard is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    /// Create a security decision for a subject and action.
    fn make_decision(&self, subject: &Subject, action: &Action) -> SecurityDecision {
        let decision_id = format!("guard-{}-{}", subject.name, Utc::now().timestamp());
        SecurityDecision {
            decision_id,
            subject: subject.clone(),
            action: action.clone(),
            allowed: true,
            reason: "Allowed by AgentGuard control plane".to_string(),
            timestamp: Utc::now(),
            mechanisms: vec![],
        }
    }
}

impl Default for AgentGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_guard_new() {
        let guard = AgentGuard::new();
        assert!(!guard.is_initialized());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_agent_guard_initialize() {
        let mut guard = AgentGuard::new();
        assert!(guard.initialize().is_ok());
        assert!(guard.is_initialized());
    }

    #[test]
    fn test_make_decision() {
        let guard = AgentGuard::new();
        let subject = Subject {
            pid: Some(1234),
            name: "test-agent".to_string(),
            cgroup_path: None,
        };
        let action = Action {
            action_type: ActionType::FileRead,
            resource: "/etc/passwd".to_string(),
            metadata: None,
        };
        let decision = guard.make_decision(&subject, &action);
        assert!(decision.allowed);
        assert_eq!(decision.subject.name, "test-agent");
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_agent_guard_not_available() {
        let mut guard = AgentGuard::new();
        let result = guard.initialize();
        assert!(result.is_err());
    }
}
