//! Control plane trait for agent-guard.
//!
//! This trait defines the interface for security control planes on Linux.

use crate::error::Result;
use crate::receipt::{Action, SecurityDecision, Subject};

/// Control plane for enforcing security policies on agents.
///
/// Implementations provide Linux-native security mechanisms such as:
/// - BPF LSM hooks
/// - cgroup v2 restrictions
/// - Landlock sandboxing
/// - seccomp filters
/// - eBPF programs
pub trait ControlPlane {
    /// Initialize the control plane.
    fn initialize(&mut self) -> Result<()>;

    /// Check if the control plane is initialized and ready.
    fn is_initialized(&self) -> bool;

    /// Evaluate an action and return a security decision.
    fn evaluate(&mut self, subject: &Subject, action: &Action) -> Result<SecurityDecision>;

    /// Enforce a security decision.
    fn enforce(&mut self, decision: &SecurityDecision) -> Result<()>;

    /// Release resources held by this control plane.
    fn shutdown(&mut self) -> Result<()>;
}
