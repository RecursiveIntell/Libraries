//! Controlled vocabulary enums for effect-runtime artifact fields.
//!
//! Each enum represents a closed set of domain values used across effect
//! lifecycle artifacts. All variants serialize as `snake_case`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Atomicity guarantee level for an effect commit operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommitAtomicityV1 {
    BoundedMultiStep,
    ExactlyOnce,
    AtLeastOnce,
    BestEffort,
}

/// Entity responsible for retrying a failed or timed-out effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetryOwnerV1 {
    EffectRuntime,
    ForgeOrchestration,
    Verification,
    Provider,
    Operator,
}

/// Behavior to apply when an effect window closes while execution is still in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CloseMidflightBehaviorV1 {
    AbortAndEmitReceipt,
    FailClosed,
    ContinueObservation,
    BestEffortCompensation,
}

/// Classification of the type of side-effect an intent describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectClassV1 {
    ExternalWrite,
    ExternalRead,
    ExternalDelete,
    StateTransition,
    Compensation,
}

/// Upper bound on the scope of impact an effect may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlastRadiusCeilingV1 {
    SingleTenant,
    SingleService,
    Regional,
    Global,
}

/// Classification of how reversible an effect is after execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClassV1 {
    Compensatable,
    Replayable,
    Irreversible,
}

/// Execution mode governing whether an effect produces real side-effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunModeV1 {
    Live,
    Simulated,
    DryRun,
    Replay,
}

/// Whether an effect intent must be published to governance observers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationStatusV1 {
    Required,
    AdvisoryOnly,
    Deferred,
    Suppressed,
}

/// Binary pass/fail result for a preflight check gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckResultV1 {
    Pass,
    Fail,
}

/// Whether the budget allocation is sufficient for the intended effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetSufficiencyResultV1 {
    Sufficient,
    Insufficient,
}

/// Terminal or in-progress state of an effect or compensation execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStateV1 {
    Completed,
    Partial,
    Cancelled,
    TimedOut,
    Failed,
    InProgress,
}

/// State of post-execution observation for an effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStateV1 {
    Complete,
    InProgress,
    DriftDetected,
}

/// Recommended next action after observing an executed effect's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosureRecommendationV1 {
    CloseSuccessfully,
    OpenCompensation,
    EscalateReview,
    ReplayReady,
}

/// Classification of the compensation strategy to be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompensationClassV1 {
    StateRestore,
    LocalRollback,
    ForwardFix,
    OperatorRunbook,
}

/// Observed state of a side-effect in an external system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExternalObservationStateV1 {
    Committed,
    Pending,
    RolledBack,
    Failed,
}
