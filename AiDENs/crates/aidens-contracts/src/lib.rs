//! Versioned base contracts for AiDENs.
//!
//! This crate defines shared primitive artifact shapes. Semantic crates own
//! their deeper behavior; this crate prevents duplicate wire-visible types.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

mod agent_bundle;
mod app_status;
mod artifact;
mod boundary;
mod capability_turn;
mod daemon_queue;
mod execution;
mod mechanism_display;
mod operator;
mod proof;
mod provider;
mod release_completion;
mod reserved_v11;
mod schema_catalog;
mod semantic;
mod tool_artifacts;
pub mod view_disclosure;
mod view_runtime;

pub use agent_bundle::*;
pub use app_status::*;
pub use artifact::*;
pub use boundary::*;
pub use capability_turn::*;
pub use daemon_queue::*;
pub use execution::*;
pub use mechanism_display::*;
pub use operator::*;
pub use proof::*;
pub use provider::*;
pub use release_completion::*;
pub use reserved_v11::*;
pub use schema_catalog::*;
pub use semantic::*;
pub use tool_artifacts::*;
pub use view_runtime::*;

use schema_catalog::{sorted_unique_artifact_ids, sorted_unique_invariants, sorted_unique_strings};

/// Canonical identity and digest surfaces from the shared stack.
///
/// AiDENs re-exports the real stack ID crate. App-only constructors that need a
/// non-replay display ID use `display_only_unstable_id`.
pub mod canonical_stack {
    pub use kernel_execution::{
        ExecutionReport as CanonicalKernelExecutionReport,
        ExecutionStopReason as CanonicalKernelStopReason,
    };
    pub use kernel_oracles::OracleAssessment as CanonicalOracleAssessment;
    pub use llm_tool_runtime::ToolSideEffectClass as CanonicalToolSideEffectClass;
    pub use recursive_kernel_core::KernelRun as CanonicalKernelRun;
    pub use semantic_memory_forge::{
        DispatchOutcomeV1 as ForgeDispatchOutcomeV1, EpisodeBundleV1 as ForgeEpisodeBundleV1,
        ExecutionContextV1 as ForgeExecutionContextV1,
    };
    pub use stack_ids::{
        ArtifactId, AttemptId, BoundaryRepairRecordId, ClaimId, ClaimVersionId, ContentDigest,
        ControlReceiptId, DegradationRecordId, DigestBuilder, EnvelopeId, EpisodeId, ImportBatchId,
        KernelRunId, ProjectionId, RegionId, ResidualId, Scope, ScopeKey, SyndromeId,
        ToolEffectDispatchReceiptId, TraceCtx, TrialId,
    };
    pub use verification_adjudication::VerificationDisposition as CanonicalVerificationDisposition;
    pub use verification_control::{
        BoundaryRepairRecord as CanonicalBoundaryRepairRecord,
        CheckPlan as CanonicalVerificationPlan, ControlReceipt as CanonicalControlReceipt,
        VerificationCase as CanonicalVerificationCase,
    };

    pub fn digest_json(value: &serde_json::Value) -> Result<ContentDigest, stack_ids::DigestError> {
        ContentDigest::compute_json(value)
    }
}

pub use attestation_exchange::AttestationEnvelopeV1;
pub use canonical_stack::CanonicalVerificationDisposition;
pub use canonical_stack::{
    ArtifactId, ArtifactId as StackArtifactId, AttemptId as StackAttemptId,
    BoundaryRepairRecordId as StackBoundaryRepairRecordId, CanonicalKernelStopReason,
    CanonicalToolSideEffectClass, ClaimId as StackClaimId, ClaimVersionId as StackClaimVersionId,
    ContentDigest as StackContentDigest, ControlReceiptId as StackControlReceiptId,
    DegradationRecordId as StackDegradationRecordId, DigestBuilder as StackDigestBuilder,
    EnvelopeId as StackEnvelopeId, EpisodeId as StackEpisodeId,
    ImportBatchId as StackImportBatchId, KernelRunId as StackKernelRunId,
    ProjectionId as StackProjectionId, RegionId as StackRegionId, ResidualId as StackResidualId,
    Scope as StackScope, ScopeKey as StackScopeKey, SyndromeId as StackSyndromeId,
    ToolEffectDispatchReceiptId as StackToolEffectDispatchReceiptId, TraceCtx as StackTraceCtx,
    TrialId as StackTrialId,
};
pub use federated_settlement::{SettlementCaseV1, SharedDispositionV1};
pub use mechanism_runtime::{HypothesisLibraryV1, TheoryRefuterSuiteV1, TheoryVersionV1};

static DISPLAY_ONLY_UNSTABLE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Construct a deterministic AiDENs-local artifact ID from explicit material.
///
/// This helper is for replay-sensitive local display artifacts. Canonical IDs
/// and digests remain owned by `stack-ids`.
pub fn generated_artifact_id_from_material(prefix: &str, material: &str) -> ArtifactId {
    local_artifact_id_from_stack_digest(prefix, material)
}

/// Construct a process-local AiDENs display ID for non-replay equality paths.
///
/// This deliberately avoids random UUIDs. Callers that need replay equality
/// must use `generated_artifact_id_from_material` with stable material.
pub fn display_only_unstable_id(prefix: &str) -> ArtifactId {
    let sequence = DISPLAY_ONLY_UNSTABLE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    ArtifactId::new(format!("{prefix}:local-process-seq-{sequence}"))
}

/// AiDENs-local wrapper pointer to a canonical owner artifact or owner type.
///
/// This does not make AiDENs authoritative for the pointed-to concept; it
/// records where a display/report DTO must be reconciled for canonical truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CanonicalBackpointerV1 {
    pub owner_crate: String,
    pub owner_type: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl CanonicalBackpointerV1 {
    pub fn owner_type(
        owner_crate: impl Into<String>,
        owner_type: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            owner_crate: owner_crate.into(),
            owner_type: owner_type.into(),
            role: role.into(),
            artifact_id: None,
            external_id: None,
            reason_codes: vec!["canonical-owner-backpointer".into()],
        }
    }

    pub fn artifact(
        owner_crate: impl Into<String>,
        owner_type: impl Into<String>,
        role: impl Into<String>,
        artifact_id: ArtifactId,
    ) -> Self {
        Self {
            owner_crate: owner_crate.into(),
            owner_type: owner_type.into(),
            role: role.into(),
            artifact_id: Some(artifact_id),
            external_id: None,
            reason_codes: vec!["canonical-artifact-backpointer".into()],
        }
    }

    pub fn external(
        owner_crate: impl Into<String>,
        owner_type: impl Into<String>,
        role: impl Into<String>,
        external_id: impl Into<String>,
    ) -> Self {
        Self {
            owner_crate: owner_crate.into(),
            owner_type: owner_type.into(),
            role: role.into(),
            artifact_id: None,
            external_id: Some(external_id.into()),
            reason_codes: vec!["canonical-external-id-backpointer".into()],
        }
    }
}

fn canonical_owner_backpointer(
    owner_crate: &'static str,
    owner_type: &'static str,
    role: &'static str,
) -> Vec<CanonicalBackpointerV1> {
    vec![CanonicalBackpointerV1::owner_type(
        owner_crate,
        owner_type,
        role,
    )]
}

fn json_digest(value: &serde_json::Value) -> String {
    non_authoritative_json_display_digest(value)
}

pub fn non_authoritative_json_display_digest(value: &serde_json::Value) -> String {
    let digest = canonical_stack::digest_json(value)
        .expect("serde_json::Value serialization is infallible for stack digesting");
    non_authoritative_display_digest_string(&digest)
}

pub fn non_authoritative_text_display_digest(text: &str) -> String {
    let digest = StackContentDigest::compute_str(text);
    non_authoritative_display_digest_string(&digest)
}

fn non_authoritative_display_digest_string(digest: &StackContentDigest) -> String {
    format!("blake3:{}", digest.hex())
}

fn local_artifact_id_from_stack_digest(prefix: &str, material: &str) -> ArtifactId {
    let digest = StackContentDigest::compute_str(material);
    ArtifactId::new(format!("{prefix}:{}", digest.hex()))
}

fn generated_artifact_id_from_framed_fields(
    prefix: &str,
    domain: &str,
    fields: &[(&str, &str)],
) -> ArtifactId {
    let mut material = String::new();
    append_material_frame(&mut material, "domain", domain);
    for (label, value) in fields {
        append_material_frame(&mut material, label, value);
    }
    generated_artifact_id_from_material(prefix, &material)
}

fn append_material_frame(material: &mut String, label: &str, value: &str) {
    material.push_str(&label.len().to_string());
    material.push(':');
    material.push_str(label);
    material.push('=');
    material.push_str(&value.len().to_string());
    material.push(':');
    material.push_str(value);
    material.push(';');
}

// Display formatting only. Stack-owned digest computation must go through
// `stack_ids::ContentDigest` and this string must not be used as identity.
fn display_json_string(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactVersion {
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AidensRunContextV1 {
    pub run_id: ArtifactId,
    pub trace_id: ArtifactId,
    pub attempt_family_id: ArtifactId,
    pub attempt_id: ArtifactId,
    pub started_at: DateTime<Utc>,
    pub app_id: String,
    pub config_generation: Option<String>,
}

impl AidensRunContextV1 {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self::for_execution(app_id, "unspecified-execution-input")
    }

    pub fn for_execution(app_id: impl Into<String>, execution_input: impl AsRef<str>) -> Self {
        Self::for_execution_at(app_id, execution_input, Utc::now())
    }

    pub fn for_execution_at(
        app_id: impl Into<String>,
        execution_input: impl AsRef<str>,
        started_at: DateTime<Utc>,
    ) -> Self {
        let app_id = app_id.into();
        let started_at_material = started_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
        let fields = [
            ("app-id", app_id.as_str()),
            ("execution-input", execution_input.as_ref()),
            ("started-at", started_at_material.as_str()),
        ];
        Self {
            run_id: generated_artifact_id_from_framed_fields(
                "run",
                "aidens-run-context-v1",
                &fields,
            ),
            trace_id: generated_artifact_id_from_framed_fields(
                "trace",
                "aidens-run-context-v1",
                &fields,
            ),
            attempt_family_id: generated_artifact_id_from_framed_fields(
                "attempt-family",
                "aidens-run-context-v1",
                &fields,
            ),
            attempt_id: generated_artifact_id_from_framed_fields(
                "attempt",
                "aidens-run-context-v1",
                &fields,
            ),
            started_at,
            app_id,
            config_generation: None,
        }
    }

    pub fn stack_trace_ctx(&self) -> StackTraceCtx {
        StackTraceCtx::from_trace_id(self.trace_id.0.clone())
    }

    pub fn stack_attempt_id(&self) -> StackAttemptId {
        StackAttemptId::new(self.attempt_id.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKindV1 {
    Run,
    Turn,
    ProviderRoute,
    ProviderReadiness,
    ToolExposure,
    ToolAttempt,
    ToolInvocation,
    StopRule,
    BudgetExhaustion,
    Approval,
    PermitUse,
    BoundaryRepair,
    SchemaValidation,
    RepoRead,
    RepoList,
    PatchProposal,
    PatchApply,
    CommandRun,
    CodexPacket,
    SandboxTruth,
    MemoryWrite,
    MemoryQuery,
    Job,
    QueueLease,
    ScheduleOccurrence,
    WakeSignal,
    DaemonNamespace,
    SafeMode,
    DuplicateSuppression,
    RuntimeViewRequest,
    QueryWidening,
    RetrievalPolicy,
    ProjectionDigest,
    DegradationEvent,
    ReleaseReadinessReport,
    OperatorStatusReport,
    ExampleAppManifest,
    InstallSmokeReport,
    CompiledRegionGraph,
    RegionContract,
    Syndrome,
    Residual,
    OracleSliceRequest,
    KernelRunReport,
    ConvergenceReport,
    BoundaryMessage,
    BoundaryReceipt,
    SubtractionPlan,
    SupportCore,
    RemovalFrontier,
    InvariantBudget,
    CompactionReport,
    HistoryPreservationReport,
    Backfill,
    ConfigApply,
    ApiHonesty,
    PlanRuntimeParity,
    CapabilityTruth,
    AttestationEnvelope,
    AdmissionDecision,
    TrustRoot,
    Treaty,
    RemoteOracleReport,
    SettlementCase,
    SharedDisposition,
    MechanismReport,
    TheoryVersion,
    HypothesisLibrary,
    SimulatorContract,
    FitRunReport,
    InvarianceReport,
    TheoryRefuterSuite,
    CompletionAuditReport,
    ReleaseArtifactManifest,
    CrossPassTraceabilityMatrix,
    KnownLimitationsRegister,
    RegressionDebtLedger,
    QueueHop,
    Schedule,
    ViewDisclosure,
}

impl JsonSchema for ArtifactKindV1 {
    fn schema_name() -> String {
        "ArtifactKindV1".into()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::String,
            ))),
            ..Default::default()
        };
        schema.metadata = Some(Box::new(schemars::schema::Metadata {
            description: Some(
                "AiDENs-local display discriminator; not a canonical artifact family schema."
                    .into(),
            ),
            ..Default::default()
        }));
        schemars::schema::Schema::Object(schema)
    }
}

impl fmt::Display for ArtifactKindV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Run => "run",
            Self::Turn => "turn",
            Self::ProviderRoute => "provider-route",
            Self::ProviderReadiness => "provider-readiness",
            Self::ToolExposure => "tool-exposure",
            Self::ToolAttempt => "tool-attempt",
            Self::ToolInvocation => "tool-invocation",
            Self::StopRule => "stop-rule",
            Self::BudgetExhaustion => "budget-exhaustion",
            Self::Approval => "approval",
            Self::PermitUse => "permit-use",
            Self::BoundaryRepair => "boundary-repair",
            Self::SchemaValidation => "schema-validation",
            Self::RepoRead => "repo-read",
            Self::RepoList => "repo-list",
            Self::PatchProposal => "patch-proposal",
            Self::PatchApply => "patch-apply",
            Self::CommandRun => "command-run",
            Self::CodexPacket => "codex-packet",
            Self::SandboxTruth => "sandbox-truth",
            Self::MemoryWrite => "memory-write",
            Self::MemoryQuery => "memory-query",
            Self::Job => "job",
            Self::QueueLease => "queue-lease",
            Self::ScheduleOccurrence => "schedule-occurrence",
            Self::WakeSignal => "wake-signal",
            Self::DaemonNamespace => "daemon-namespace",
            Self::SafeMode => "safe-mode",
            Self::DuplicateSuppression => "duplicate-suppression",
            Self::RuntimeViewRequest => "runtime-view-request",
            Self::QueryWidening => "query-widening",
            Self::RetrievalPolicy => "retrieval-policy",
            Self::ProjectionDigest => "projection-digest",
            Self::DegradationEvent => "degradation-event",
            Self::ReleaseReadinessReport => "release-readiness-report",
            Self::OperatorStatusReport => "operator-status-report",
            Self::ExampleAppManifest => "example-app-manifest",
            Self::InstallSmokeReport => "install-smoke-report",
            Self::CompiledRegionGraph => "compiled-region-graph",
            Self::RegionContract => "region-contract",
            Self::Syndrome => "syndrome",
            Self::Residual => "residual",
            Self::OracleSliceRequest => "oracle-slice-request",
            Self::KernelRunReport => "kernel-run-report",
            Self::ConvergenceReport => "convergence-report",
            Self::BoundaryMessage => "boundary-message",
            Self::BoundaryReceipt => "boundary-receipt",
            Self::SubtractionPlan => "subtraction-plan",
            Self::SupportCore => "support-core",
            Self::RemovalFrontier => "removal-frontier",
            Self::InvariantBudget => "invariant-budget",
            Self::CompactionReport => "compaction-report",
            Self::HistoryPreservationReport => "history-preservation-report",
            Self::Backfill => "backfill",
            Self::ConfigApply => "config-apply",
            Self::ApiHonesty => "api-honesty",
            Self::PlanRuntimeParity => "plan-runtime-parity",
            Self::CapabilityTruth => "capability-truth",
            Self::AttestationEnvelope => "attestation-envelope",
            Self::AdmissionDecision => "admission-decision",
            Self::TrustRoot => "trust-root",
            Self::Treaty => "treaty",
            Self::RemoteOracleReport => "remote-oracle-report",
            Self::SettlementCase => "settlement-case",
            Self::SharedDisposition => "shared-disposition",
            Self::MechanismReport => "mechanism-report",
            Self::TheoryVersion => "theory-version",
            Self::HypothesisLibrary => "hypothesis-library",
            Self::SimulatorContract => "simulator-contract",
            Self::FitRunReport => "fit-run-report",
            Self::InvarianceReport => "invariance-report",
            Self::TheoryRefuterSuite => "theory-refuter-suite",
            Self::CompletionAuditReport => "completion-audit-report",
            Self::ReleaseArtifactManifest => "release-artifact-manifest",
            Self::CrossPassTraceabilityMatrix => "cross-pass-traceability-matrix",
            Self::KnownLimitationsRegister => "known-limitations-register",
            Self::RegressionDebtLedger => "regression-debt-ledger",
            Self::QueueHop => "queue-hop",
            Self::Schedule => "schedule",
            Self::ViewDisclosure => "view-disclosure",
        })
    }
}

#[cfg(test)]
mod tests;
