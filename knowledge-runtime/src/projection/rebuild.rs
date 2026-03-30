//! Projection rebuild driver interface (KR-003).
//!
//! The [`ProjectionTracker`](super::lifecycle::ProjectionTracker) is **observability only**.
//! It records projection health transitions but does NOT execute, schedule, or
//! retry rebuilds. External callers must implement the [`RebuildDriver`] trait
//! to drive rebuild execution and report outcomes back to the tracker.
//!
//! ## Why a trait instead of built-in execution?
//!
//! Rebuild execution depends on:
//! - Which upstream data sources to re-query
//! - How to batch/throttle rebuilds
//! - Retry and error handling policies
//! - Concurrency and resource management
//!
//! These concerns are owned by the orchestrator, not the runtime. The runtime
//! provides projection health inspection so the driver can make informed
//! decisions, but never initiates work autonomously.
//!
//! ## Phase status: KR-003 architecture closure
//!
//! This trait is the explicit architecture answer to "who drives rebuilds?"
//! The tracker is observability; this trait is the execution contract.

use crate::error::RuntimeError;
use crate::ids::ProjectionId;
use crate::projection::lifecycle::{ProjectionHealth, ProjectionTracker, ProjectionVersion};

/// External rebuild driver trait (KR-003).
///
/// Implement this trait in your orchestration layer to drive projection
/// rebuilds based on tracker health state. The runtime never calls this
/// trait itself — it is for external callers that poll projection health
/// and decide when to rebuild.
///
/// ## Example flow
///
/// ```text
/// 1. Orchestrator polls tracker.health(&id) -> Stale
/// 2. Orchestrator calls driver.rebuild(&id) -> Ok(outcome)
/// 3. Orchestrator calls runtime.record_projection_build(id, ...) or
///    runtime.record_projection_failure(id, error)
/// 4. Tracker state transitions to Healthy or remains Stale
/// ```
#[allow(async_fn_in_trait)]
pub trait RebuildDriver {
    /// Execute a rebuild for the given projection.
    ///
    /// Returns a [`RebuildOutcome`] on success, or an error if the rebuild
    /// could not be attempted. The caller is responsible for reporting the
    /// outcome back to the tracker via `record_projection_build()` or
    /// `record_projection_failure()`.
    async fn rebuild(&self, id: &ProjectionId) -> Result<RebuildOutcome, RuntimeError>;

    /// Check whether this driver can handle a rebuild for the given projection.
    ///
    /// Returns `false` if the projection kind/scope is not supported by this
    /// driver. The orchestrator can use this to skip unsupported projections
    /// without error.
    fn can_rebuild(&self, id: &ProjectionId) -> bool;
}

/// Outcome of a successful rebuild.
#[derive(Debug, Clone)]
pub struct RebuildOutcome {
    /// How many source items were processed.
    pub source_count: usize,
    /// How long the rebuild took, in milliseconds.
    pub build_duration_ms: u64,
    /// Version metadata for the rebuilt projection.
    pub version: Option<ProjectionVersion>,
}

/// Drive rebuilds for all stale projections using the provided driver.
///
/// This is a convenience function that polls the tracker for stale projections
/// and calls the driver for each one. It reports outcomes back to the tracker
/// automatically.
///
/// Returns the number of projections that were successfully rebuilt.
pub async fn rebuild_stale<D: RebuildDriver>(
    tracker: &mut ProjectionTracker,
    driver: &D,
) -> Result<usize, RuntimeError> {
    let stale_ids: Vec<ProjectionId> = tracker
        .all_ids()
        .filter(|id| tracker.health(id) == ProjectionHealth::Stale)
        .cloned()
        .collect();

    let mut rebuilt = 0;
    for id in stale_ids {
        if !driver.can_rebuild(&id) {
            continue;
        }
        match driver.rebuild(&id).await {
            Ok(outcome) => {
                tracker.record_build(
                    id,
                    outcome.source_count,
                    outcome.build_duration_ms,
                    outcome.version,
                );
                rebuilt += 1;
            }
            Err(e) => {
                tracker.record_failure(id, format!("{e}"));
            }
        }
    }
    Ok(rebuilt)
}
