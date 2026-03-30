//! OODA governance orchestrator for the RecursiveIntell stack.
//!
//! This crate drives the Observe-Orient-Decide-Act loop that inspects
//! semantic-memory state, selects verification targets, executes oracle
//! or paired-patch plans, and exports evidence bundles through the
//! canonical Forge bridge path.

pub mod act;
pub mod bootstrap;
pub mod bootstrap_source;
pub mod bundle_builder;
pub mod cli;
pub mod config;
pub mod decide;
pub mod error;
pub mod export;
#[cfg(feature = "governance")]
pub mod governance_gate;
pub mod history;
pub mod loop_runner;
pub mod observe;
pub mod orient;
pub mod receipts;
#[path = "repo_chat.rs"]
pub mod repo_chat;
pub mod targets;
pub mod types;

pub use act::{
    execute_plan, ActionFamily, ActionOutcome, AdvisoryPlan, OracleExecution, PatchExecution,
    PlanKind,
};
pub use bootstrap::{
    compute_manifest_delta, BootstrapCurrentState, BootstrapDeltaSummary, BootstrapManifestDelta,
    BootstrapManifestSnapshot, BootstrapRichness, BootstrapSourceObservation,
    BootstrapSourceRichness, SymbolRecord,
};
pub use bootstrap_source::{
    bootstrap_source_workspace, BootstrapSourceReport, BootstrapSourceSkippedFile,
};
pub use bundle_builder::{
    build_bundle_from_oracle, build_bundle_from_patch, OracleBundleInput, PatchBundleInput,
};
pub use cli::{
    render_candidate_report, render_loop_report, render_observation_report, render_repo_chat_answer,
};
pub use config::{LoopConfig, PatchPlanSeed};
pub use decide::{select_candidate, Decision};
pub use error::PilotError;
pub use export::{
    canonical_roundtrip, import_recent_forge_bundles, ImportBootstrapReport, RoundtripResult,
};
#[cfg(feature = "governance")]
pub use governance_gate::{
    build_governance_receipt, gate_execution, observe_governance, predicates as governance_predicates,
    GovernanceDegradation, GovernanceGateResult, GovernanceObservation,
    GovernanceObservationSummary, GovernanceReceiptV1, GOVERNANCE_PROJECTION_FAMILY,
    GOVERNANCE_RECEIPT_V1_SCHEMA, GOVERNANCE_SCOPE_NAMESPACE,
};
pub use history::{PilotHistory, TargetAttemptRecord, TargetHistoryEntry};
pub use loop_runner::{
    parse_loop_report_boundary, ExternalHaltFlag, HaltReason, LoopBudgetReceipt,
    LoopIterationReport, LoopReceipt, LoopReport, LoopRunner, LoopRunnerResources,
    LOOP_ITERATION_REPORT_V1_SCHEMA, LOOP_REPORT_V1_SCHEMA, PILOT_LOOP_RECEIPT_V1_SCHEMA,
};
pub use observe::{
    inspect_observation_paths, ImportRecordDisposition, ObservationDisposition, ObservationPaths,
    ObservationStatus, PathAvailability, SourceInventory,
};
pub use observe::{observe_scope, Observation, ObservationDegradation, ScopeHealthSummary};
pub use orient::{extract_targets, score_targets, TargetCandidate};
pub use repo_chat::{
    answer_repo_question, answer_repo_question_from_observation, provider_fallback_allowed,
    render_terminal_answer, repo_chat_provider_grounding_message, route_question, RepoChatAnswer,
    RepoChatCitation, RepoChatEvidence, RepoChatEvidenceType, RepoChatGroundingMode,
    RepoQuestionRoute,
};
pub use targets::{TargetKind, TargetPriority};
pub use types::{
    BlockedStepRecord, BudgetClass, CanonicalCaseClass, DecisionAudit, ExecutionLineageReceipt,
    ExportActionTraceV1, LawfulStepKind, PlannedStep, RepairClassV1, RepairRecordV1,
    StopRuleEvaluation, TargetNormalization, VerificationPlanArtifact,
};
