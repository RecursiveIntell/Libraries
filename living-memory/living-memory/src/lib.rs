//! # forge-engine
//!
//! Operational verification and evaluation engine for the local-first AI stack.
//!
//! `forge-engine` owns the execution-heavy lane: compile mindstate, validate and
//! apply structured patches, run checks, score candidate outcomes, persist
//! operational evidence, and update causal edit-attribution state.
//!
//! ## Authority boundary
//!
//! - `semantic-memory-forge` owns raw verification/export wire truth.
//! - `forge-memory-bridge` owns transformation only.
//! - `semantic-memory` owns durable projected/queryable truth.
//! - `forge-engine` owns operational verification work on top of those crates.
//!
//! The `danger-sm-write` feature remains a compatibility-only escape hatch and
//! is not the canonical export/import path.

pub mod adapters;
pub mod baseline;
pub mod cea;
pub mod config;
pub mod error;
pub mod exec;
pub mod experiment;
pub mod export;
pub mod failure;
pub mod invariants;
pub mod lab;
pub mod runtime;
pub mod scoring;
pub mod store;
pub mod tool_receipts;

// Re-export primary public types.
pub use config::ForgeConfig;
pub use error::{ForgeError, ForgeResult, Violation, ViolationKind};

// Runtime types
pub use runtime::mindstate::MindState;
pub use runtime::patch::{
    apply_patch, render_diff, validate_patch, Anchor, EditOp, FileEdit, FileMode,
    LineAttributionMap, LineRange, StructuredPatch, ValidationResult,
};
pub use runtime::stabilizer::Stabilizer;
pub use runtime::{compile_mindstate, extract_strategy_tags, AttemptPhase, DeltaPolicy};

// Exec types
pub use exec::{
    is_env_allowed, select_backend, CheckCommand, CheckKind, CheckResult, CommandOutput,
    CommandTimings, ContainerBackend, ContainerRuntime, EffectSignature, ExecutionBackend,
    ExecutionBackendKind, HostBackend, LocatedEffect, LogBundle, ParsedCheckOutput,
    PatchedWorkspace, Workspace,
};

// Adapter types
pub use adapters::{CargoAdapter, ProjectAdapter};

// Store types
pub use store::ForgeStore;

// Lab types
pub use config::ForgeLimits;
pub use lab::evaluate::{compute_correctness, compute_scores, persist_eval_run};
pub use lab::{
    archive_insert, compute_cell_key, compute_score_summary, promote, AlgebraSpec, ArchiveUpdate,
    BasisVersion, CausalFingerprint, CausalHypothesis, EvalRunResult, EvalSuite, EvalTask,
    ExperimentEvidenceBundle, ExperimentRunner, HypothesisStatus, LocalExperimentRunner,
    ParameterBounds, ScoreVector, VerificationPlan, VerificationType,
};

// CEA types
pub use cea::{
    attribute_effects, build_edit_op_signature, compute_run_hash, load_graph, load_graph_with_tx,
    predict, update_graph, update_graph_with_tx, AttributedRunResult, AttributionTriple,
    CausalEdge, CausalGraph, CausalNode, CausalPrediction, CoverageSummary, EditOpSignature,
    RiskFlag, UpdateResult,
};

// Baseline types
pub use baseline::{
    capture_baseline_provenance, BaselineDescriptor, BaselineSourceKind, ComparabilityPolicy,
    WorkspacePolicy,
};

// Experiment types
pub use experiment::{
    AnalysisRecord, CacheMode, EffectKind, EvidenceRecord, ExecutionRecord, ExperimentConfig,
    ExperimentDiff, ExperimentExportRecord, ExperimentMode, ExperimentResult,
    PairedExperimentRunner, RunIdentity, StatisticsPolicy, TrialRecord, TrialSide,
    TypedLocatedEffect,
};

// Scoring types
pub use scoring::{
    ComparabilityClass, ObjectiveKind, ObjectivePolicy, PatchExecutionPlan, PlannedCheck,
};

// Export types
pub use export::{
    compute_export_key, export_bundle, export_bundle_with_memory_write_through_compat,
    EpisodeExport, RENDERING_VERSION,
};
pub use tool_receipts::ForgeToolReceiptSink;

// Failure types
pub use failure::{FailureClass, FailureRecord};

// Evidence types (v2 + Phase 5)
pub use lab::evidence::{
    build_hypothesis_edges, compute_assessment, generate_verification_plan,
    local_hypothesis_support_confidence, update_hypotheses_from_diff, AssessmentCategory,
    BundleScope, ClaimStrength, ContradictionState, Covariates, DroppedStep, EvidenceAssessment,
    HypothesisEdge, HypothesisEdgeKind, PairComparability, PlanBudget, ReceiptKind, ReceiptRef,
    ReceiptStorage, SampleSupport, StepRequirement, Treatment, VerificationPolicy,
    VerificationState, VerificationStep,
};
