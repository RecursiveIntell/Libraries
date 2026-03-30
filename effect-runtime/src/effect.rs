//! Core effect lifecycle artifacts: window, intent, preflight, and commit decision.
//!
//! These types model the governed effect pipeline from intent declaration through
//! commit authorization. Each artifact enforces owner-side invariants via its
//! `validate()` method and is constructed through a builder pattern.

use crate::error::{
    require_id, require_non_empty, require_non_empty_slice, require_schema_version,
    require_valid_timestamp, EffectRuntimeValidationError, EffectValidationResult,
};
use crate::v25::{V25ConstitutionCitation, V25ObligationRefs};
use crate::vocab::{
    BlastRadiusCeilingV1, BudgetSufficiencyResultV1, CheckResultV1, CloseMidflightBehaviorV1,
    CommitAtomicityV1, EffectClassV1, PublicationStatusV1, RetryOwnerV1, ReversibilityClassV1,
    RunModeV1,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApprovalGrantId, EffectCommitDecisionId, EffectIntentId, EffectPreflightReportId,
    EffectWindowId, ExecutionPermitId,
};

/// Outcome disposition of a preflight check determining whether an effect may proceed to commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectPreflightDispositionV1 {
    Blocked,
    AdvisoryOnly,
    CommitEligible,
}

/// Authorization outcome of a commit decision: denied, advisory-only, or authorized for execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectCommitDispositionV1 {
    Denied,
    AdvisoryOnly,
    Authorized,
}

/// Temporal execution window that bounds when an effect may start, timeout, and how mid-flight closures are handled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectWindowV1 {
    pub schema_version: String,
    pub effect_window_id: EffectWindowId,
    pub earliest_start: String,
    pub latest_admissible_start: String,
    pub timeout_at: String,
    pub commit_atomicity: CommitAtomicityV1,
    pub retry_owner: RetryOwnerV1,
    pub close_midflight_behavior: CloseMidflightBehaviorV1,
}

/// Declaration of an intended effect including its class, target surface, blast radius, and approval requirements.
///
/// This is the entry point of the effect lifecycle. Live-mode intents require
/// non-empty approval refs before they can proceed to preflight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectIntentV1 {
    pub schema_version: String,
    pub effect_intent_id: EffectIntentId,
    pub effect_class: EffectClassV1,
    pub target_surface: String,
    pub intended_outcome: String,
    pub initiating_artifact_refs: Vec<String>,
    pub required_capability_class: String,
    pub allowed_scope: String,
    pub blast_radius_ceiling: BlastRadiusCeilingV1,
    pub reversibility_class: ReversibilityClassV1,
    pub idempotency_key: String,
    pub execution_window_id: EffectWindowId,
    pub observability_obligation_refs: Vec<String>,
    pub approval_refs: Vec<String>,
    pub run_mode: RunModeV1,
    pub publication_status: PublicationStatusV1,
}

/// Result of preflight checks against an effect intent, gating commit eligibility.
///
/// A `CommitEligible` disposition requires all check results to be passing
/// and budget to be sufficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectPreflightReportV1 {
    pub schema_version: String,
    pub effect_preflight_report_id: EffectPreflightReportId,
    pub effect_intent_id: EffectIntentId,
    /// Carries ApplicabilityContextId, ProfileSetId, CompositionReceiptId,
    /// EffectiveConstitutionId, CompiledObligationSetId, and ProfileExceptionBundleId.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(flatten)]
    pub obligation_refs: V25ObligationRefs,
    pub input_integrity_result: CheckResultV1,
    pub admission_check_result: CheckResultV1,
    pub dependency_reachability_result: CheckResultV1,
    pub budget_sufficiency_result: BudgetSufficiencyResultV1,
    pub disposition: EffectPreflightDispositionV1,
    pub generated_at: String,
    pub notes: Vec<String>,
}

/// Authorization decision for committing an effect, linking intent and preflight report to an execution permit.
///
/// Authorized decisions must carry an execution permit and approver refs.
/// Denied decisions must carry a refusal reason and must not mint permits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectCommitDecisionV1 {
    pub schema_version: String,
    pub effect_commit_decision_id: EffectCommitDecisionId,
    pub effect_intent_id: EffectIntentId,
    pub effect_preflight_report_id: EffectPreflightReportId,
    /// Carries ApplicabilityContextId, ProfileSetId, CompositionReceiptId,
    /// EffectiveConstitutionId, CompiledObligationSetId, and ProfileExceptionBundleId.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(flatten)]
    pub obligation_refs: V25ObligationRefs,
    #[serde(default)]
    pub approval_obligation_refs: Vec<String>,
    pub decision_state: EffectCommitDispositionV1,
    pub approver_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_permit_id: Option<ExecutionPermitId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_grant_id: Option<ApprovalGrantId>,
    pub refusal_reason: String,
    pub generated_at: String,
}

impl EffectWindowV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectWindowV1";

    /// Returns a builder for a validated execution window artifact.
    pub fn builder(
        effect_window_id: EffectWindowId,
        earliest_start: impl Into<String>,
        latest_admissible_start: impl Into<String>,
        timeout_at: impl Into<String>,
        commit_atomicity: CommitAtomicityV1,
        retry_owner: RetryOwnerV1,
        close_midflight_behavior: CloseMidflightBehaviorV1,
    ) -> EffectWindowV1Builder {
        EffectWindowV1Builder::new(
            effect_window_id,
            earliest_start,
            latest_admissible_start,
            timeout_at,
            commit_atomicity,
            retry_owner,
            close_midflight_behavior,
        )
    }

    /// Validates the owner-side invariants for an execution window.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version("EffectWindowV1", Self::SCHEMA_VERSION, &self.schema_version)?;
        require_id(&self.effect_window_id, "effect_window_id")?;
        require_valid_timestamp(&self.earliest_start, "earliest_start")?;
        require_valid_timestamp(&self.latest_admissible_start, "latest_admissible_start")?;
        require_valid_timestamp(&self.timeout_at, "timeout_at")?;
        Ok(())
    }
}

/// Builder for [`EffectWindowV1`].
#[derive(Debug, Clone)]
pub struct EffectWindowV1Builder {
    effect_window_id: EffectWindowId,
    earliest_start: String,
    latest_admissible_start: String,
    timeout_at: String,
    commit_atomicity: CommitAtomicityV1,
    retry_owner: RetryOwnerV1,
    close_midflight_behavior: CloseMidflightBehaviorV1,
}

impl EffectWindowV1Builder {
    /// Creates a builder with the required execution window fields.
    pub fn new(
        effect_window_id: EffectWindowId,
        earliest_start: impl Into<String>,
        latest_admissible_start: impl Into<String>,
        timeout_at: impl Into<String>,
        commit_atomicity: CommitAtomicityV1,
        retry_owner: RetryOwnerV1,
        close_midflight_behavior: CloseMidflightBehaviorV1,
    ) -> Self {
        Self {
            effect_window_id,
            earliest_start: earliest_start.into(),
            latest_admissible_start: latest_admissible_start.into(),
            timeout_at: timeout_at.into(),
            commit_atomicity,
            retry_owner,
            close_midflight_behavior,
        }
    }

    /// Builds a validated execution window artifact.
    pub fn build(self) -> Result<EffectWindowV1, EffectRuntimeValidationError> {
        let value = EffectWindowV1 {
            schema_version: EffectWindowV1::SCHEMA_VERSION.to_string(),
            effect_window_id: self.effect_window_id,
            earliest_start: self.earliest_start,
            latest_admissible_start: self.latest_admissible_start,
            timeout_at: self.timeout_at,
            commit_atomicity: self.commit_atomicity,
            retry_owner: self.retry_owner,
            close_midflight_behavior: self.close_midflight_behavior,
        };
        value.validate()?;
        Ok(value)
    }
}

impl EffectIntentV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectIntentV1";

    /// Returns a builder for a validated effect intent artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        effect_intent_id: EffectIntentId,
        effect_class: EffectClassV1,
        target_surface: impl Into<String>,
        intended_outcome: impl Into<String>,
        initiating_artifact_refs: Vec<String>,
        required_capability_class: impl Into<String>,
        allowed_scope: impl Into<String>,
        blast_radius_ceiling: BlastRadiusCeilingV1,
        reversibility_class: ReversibilityClassV1,
        idempotency_key: impl Into<String>,
        execution_window_id: EffectWindowId,
        observability_obligation_refs: Vec<String>,
        approval_refs: Vec<String>,
        run_mode: RunModeV1,
        publication_status: PublicationStatusV1,
    ) -> EffectIntentV1Builder {
        EffectIntentV1Builder::new(
            effect_intent_id,
            effect_class,
            target_surface,
            intended_outcome,
            initiating_artifact_refs,
            required_capability_class,
            allowed_scope,
            blast_radius_ceiling,
            reversibility_class,
            idempotency_key,
            execution_window_id,
            observability_obligation_refs,
            approval_refs,
            run_mode,
            publication_status,
        )
    }

    /// Validates the owner-side invariants for an effect intent.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version("EffectIntentV1", Self::SCHEMA_VERSION, &self.schema_version)?;
        require_id(&self.effect_intent_id, "effect_intent_id")?;
        require_non_empty(&self.target_surface, "target_surface")?;
        require_non_empty(&self.intended_outcome, "intended_outcome")?;
        require_non_empty_slice(&self.initiating_artifact_refs, "initiating_artifact_refs")?;
        require_non_empty(&self.required_capability_class, "required_capability_class")?;
        require_non_empty(&self.allowed_scope, "allowed_scope")?;
        require_non_empty(&self.idempotency_key, "idempotency_key")?;
        require_id(&self.execution_window_id, "execution_window_id")?;
        require_non_empty_slice(
            &self.observability_obligation_refs,
            "observability_obligation_refs",
        )?;
        if matches!(self.run_mode, RunModeV1::Live) {
            require_non_empty_slice(&self.approval_refs, "approval_refs")?;
        }
        Ok(())
    }
}

/// Builder for [`EffectIntentV1`].
#[derive(Debug, Clone)]
pub struct EffectIntentV1Builder {
    effect_intent_id: EffectIntentId,
    effect_class: EffectClassV1,
    target_surface: String,
    intended_outcome: String,
    initiating_artifact_refs: Vec<String>,
    required_capability_class: String,
    allowed_scope: String,
    blast_radius_ceiling: BlastRadiusCeilingV1,
    reversibility_class: ReversibilityClassV1,
    idempotency_key: String,
    execution_window_id: EffectWindowId,
    observability_obligation_refs: Vec<String>,
    approval_refs: Vec<String>,
    run_mode: RunModeV1,
    publication_status: PublicationStatusV1,
}

impl EffectIntentV1Builder {
    /// Creates a builder with the required effect intent fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        effect_intent_id: EffectIntentId,
        effect_class: EffectClassV1,
        target_surface: impl Into<String>,
        intended_outcome: impl Into<String>,
        initiating_artifact_refs: Vec<String>,
        required_capability_class: impl Into<String>,
        allowed_scope: impl Into<String>,
        blast_radius_ceiling: BlastRadiusCeilingV1,
        reversibility_class: ReversibilityClassV1,
        idempotency_key: impl Into<String>,
        execution_window_id: EffectWindowId,
        observability_obligation_refs: Vec<String>,
        approval_refs: Vec<String>,
        run_mode: RunModeV1,
        publication_status: PublicationStatusV1,
    ) -> Self {
        Self {
            effect_intent_id,
            effect_class,
            target_surface: target_surface.into(),
            intended_outcome: intended_outcome.into(),
            initiating_artifact_refs,
            required_capability_class: required_capability_class.into(),
            allowed_scope: allowed_scope.into(),
            blast_radius_ceiling,
            reversibility_class,
            idempotency_key: idempotency_key.into(),
            execution_window_id,
            observability_obligation_refs,
            approval_refs,
            run_mode,
            publication_status,
        }
    }

    /// Set the effect class.
    pub fn with_effect_class(mut self, effect_class: EffectClassV1) -> Self {
        self.effect_class = effect_class;
        self
    }

    /// Set the blast radius ceiling.
    pub fn with_blast_radius_ceiling(mut self, ceiling: BlastRadiusCeilingV1) -> Self {
        self.blast_radius_ceiling = ceiling;
        self
    }

    /// Set the reversibility class.
    pub fn with_reversibility_class(mut self, class: ReversibilityClassV1) -> Self {
        self.reversibility_class = class;
        self
    }

    /// Set the run mode.
    pub fn with_run_mode(mut self, mode: RunModeV1) -> Self {
        self.run_mode = mode;
        self
    }

    /// Set the publication status.
    pub fn with_publication_status(mut self, status: PublicationStatusV1) -> Self {
        self.publication_status = status;
        self
    }

    /// Append approval references.
    pub fn with_approval_refs(mut self, refs: Vec<String>) -> Self {
        self.approval_refs = refs;
        self
    }

    /// Builds a validated effect intent artifact.
    pub fn build(self) -> Result<EffectIntentV1, EffectRuntimeValidationError> {
        let value = EffectIntentV1 {
            schema_version: EffectIntentV1::SCHEMA_VERSION.to_string(),
            effect_intent_id: self.effect_intent_id,
            effect_class: self.effect_class,
            target_surface: self.target_surface,
            intended_outcome: self.intended_outcome,
            initiating_artifact_refs: self.initiating_artifact_refs,
            required_capability_class: self.required_capability_class,
            allowed_scope: self.allowed_scope,
            blast_radius_ceiling: self.blast_radius_ceiling,
            reversibility_class: self.reversibility_class,
            idempotency_key: self.idempotency_key,
            execution_window_id: self.execution_window_id,
            observability_obligation_refs: self.observability_obligation_refs,
            approval_refs: self.approval_refs,
            run_mode: self.run_mode,
            publication_status: self.publication_status,
        };
        value.validate()?;
        Ok(value)
    }
}

impl EffectPreflightReportV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectPreflightReportV1";

    /// Returns a builder for a validated preflight report artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        effect_preflight_report_id: EffectPreflightReportId,
        effect_intent_id: EffectIntentId,
        citation: V25ConstitutionCitation,
        obligation_refs: V25ObligationRefs,
        input_integrity_result: CheckResultV1,
        admission_check_result: CheckResultV1,
        dependency_reachability_result: CheckResultV1,
        budget_sufficiency_result: BudgetSufficiencyResultV1,
        disposition: EffectPreflightDispositionV1,
        generated_at: impl Into<String>,
        notes: Vec<String>,
    ) -> EffectPreflightReportV1Builder {
        EffectPreflightReportV1Builder::new(
            effect_preflight_report_id,
            effect_intent_id,
            citation,
            obligation_refs,
            input_integrity_result,
            admission_check_result,
            dependency_reachability_result,
            budget_sufficiency_result,
            disposition,
            generated_at,
            notes,
        )
    }

    /// Validates the owner-side invariants for a preflight report.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "EffectPreflightReportV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(
            &self.effect_preflight_report_id,
            "effect_preflight_report_id",
        )?;
        require_id(&self.effect_intent_id, "effect_intent_id")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        require_non_empty_slice(
            &self.obligation_refs.required_obligation_refs,
            "required_obligation_refs",
        )?;
        if matches!(
            self.disposition,
            EffectPreflightDispositionV1::CommitEligible
        ) && !matches!(
            (
                self.input_integrity_result,
                self.admission_check_result,
                self.dependency_reachability_result,
                self.budget_sufficiency_result,
            ),
            (
                CheckResultV1::Pass,
                CheckResultV1::Pass,
                CheckResultV1::Pass,
                BudgetSufficiencyResultV1::Sufficient,
            )
        ) {
            return Err(EffectRuntimeValidationError::InvalidState(
                "commit-eligible preflight requires pass/pass/pass/sufficient checks",
            ));
        }
        Ok(())
    }
}

/// Builder for [`EffectPreflightReportV1`].
#[derive(Debug, Clone)]
pub struct EffectPreflightReportV1Builder {
    effect_preflight_report_id: EffectPreflightReportId,
    effect_intent_id: EffectIntentId,
    citation: V25ConstitutionCitation,
    obligation_refs: V25ObligationRefs,
    input_integrity_result: CheckResultV1,
    admission_check_result: CheckResultV1,
    dependency_reachability_result: CheckResultV1,
    budget_sufficiency_result: BudgetSufficiencyResultV1,
    disposition: EffectPreflightDispositionV1,
    generated_at: String,
    notes: Vec<String>,
}

impl EffectPreflightReportV1Builder {
    /// Creates a builder with the required preflight report fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        effect_preflight_report_id: EffectPreflightReportId,
        effect_intent_id: EffectIntentId,
        citation: V25ConstitutionCitation,
        obligation_refs: V25ObligationRefs,
        input_integrity_result: CheckResultV1,
        admission_check_result: CheckResultV1,
        dependency_reachability_result: CheckResultV1,
        budget_sufficiency_result: BudgetSufficiencyResultV1,
        disposition: EffectPreflightDispositionV1,
        generated_at: impl Into<String>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            effect_preflight_report_id,
            effect_intent_id,
            citation,
            obligation_refs,
            input_integrity_result,
            admission_check_result,
            dependency_reachability_result,
            budget_sufficiency_result,
            disposition,
            generated_at: generated_at.into(),
            notes,
        }
    }

    /// Builds a validated preflight report artifact.
    pub fn build(self) -> Result<EffectPreflightReportV1, EffectRuntimeValidationError> {
        let value = EffectPreflightReportV1 {
            schema_version: EffectPreflightReportV1::SCHEMA_VERSION.to_string(),
            effect_preflight_report_id: self.effect_preflight_report_id,
            effect_intent_id: self.effect_intent_id,
            citation: self.citation,
            obligation_refs: self.obligation_refs,
            input_integrity_result: self.input_integrity_result,
            admission_check_result: self.admission_check_result,
            dependency_reachability_result: self.dependency_reachability_result,
            budget_sufficiency_result: self.budget_sufficiency_result,
            disposition: self.disposition,
            generated_at: self.generated_at,
            notes: self.notes,
        };
        value.validate()?;
        Ok(value)
    }
}

impl EffectCommitDecisionV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectCommitDecisionV1";

    /// Returns a builder for a validated commit decision artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        effect_commit_decision_id: EffectCommitDecisionId,
        effect_intent_id: EffectIntentId,
        effect_preflight_report_id: EffectPreflightReportId,
        citation: V25ConstitutionCitation,
        obligation_refs: V25ObligationRefs,
        approval_obligation_refs: Vec<String>,
        decision_state: EffectCommitDispositionV1,
        approver_refs: Vec<String>,
        execution_permit_id: Option<ExecutionPermitId>,
        approval_grant_id: Option<ApprovalGrantId>,
        refusal_reason: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> EffectCommitDecisionV1Builder {
        EffectCommitDecisionV1Builder::new(
            effect_commit_decision_id,
            effect_intent_id,
            effect_preflight_report_id,
            citation,
            obligation_refs,
            approval_obligation_refs,
            decision_state,
            approver_refs,
            execution_permit_id,
            approval_grant_id,
            refusal_reason,
            generated_at,
        )
    }

    /// Validates the owner-side invariants for a commit decision.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "EffectCommitDecisionV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(&self.effect_commit_decision_id, "effect_commit_decision_id")?;
        require_id(&self.effect_intent_id, "effect_intent_id")?;
        require_id(
            &self.effect_preflight_report_id,
            "effect_preflight_report_id",
        )?;
        require_non_empty_slice(
            &self.obligation_refs.required_obligation_refs,
            "required_obligation_refs",
        )?;
        require_non_empty(&self.generated_at, "generated_at")?;
        match self.decision_state {
            EffectCommitDispositionV1::Authorized => {
                if self.execution_permit_id.is_none() {
                    return Err(EffectRuntimeValidationError::InvalidState(
                        "authorized commit decisions require execution_permit_id",
                    ));
                }
                require_non_empty_slice(&self.approver_refs, "approver_refs")?;
                require_non_empty_slice(
                    &self.approval_obligation_refs,
                    "approval_obligation_refs",
                )?;
                if !self.refusal_reason.trim().is_empty() {
                    return Err(EffectRuntimeValidationError::InvalidState(
                        "authorized commit decisions must not carry refusal_reason",
                    ));
                }
            }
            EffectCommitDispositionV1::AdvisoryOnly => {
                if self.execution_permit_id.is_some() || self.approval_grant_id.is_some() {
                    return Err(EffectRuntimeValidationError::InvalidState(
                        "advisory-only commit decisions must not mint execution permits or approval grants",
                    ));
                }
            }
            EffectCommitDispositionV1::Denied => {
                if self.execution_permit_id.is_some() || self.approval_grant_id.is_some() {
                    return Err(EffectRuntimeValidationError::InvalidState(
                        "denied commit decisions must not mint execution permits or approval grants",
                    ));
                }
                require_non_empty(&self.refusal_reason, "refusal_reason")?;
            }
        }
        Ok(())
    }
}

/// Builder for [`EffectCommitDecisionV1`].
#[derive(Debug, Clone)]
pub struct EffectCommitDecisionV1Builder {
    effect_commit_decision_id: EffectCommitDecisionId,
    effect_intent_id: EffectIntentId,
    effect_preflight_report_id: EffectPreflightReportId,
    citation: V25ConstitutionCitation,
    obligation_refs: V25ObligationRefs,
    approval_obligation_refs: Vec<String>,
    decision_state: EffectCommitDispositionV1,
    approver_refs: Vec<String>,
    execution_permit_id: Option<ExecutionPermitId>,
    approval_grant_id: Option<ApprovalGrantId>,
    refusal_reason: String,
    generated_at: String,
}

impl EffectCommitDecisionV1Builder {
    /// Creates a builder with the required commit decision fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        effect_commit_decision_id: EffectCommitDecisionId,
        effect_intent_id: EffectIntentId,
        effect_preflight_report_id: EffectPreflightReportId,
        citation: V25ConstitutionCitation,
        obligation_refs: V25ObligationRefs,
        approval_obligation_refs: Vec<String>,
        decision_state: EffectCommitDispositionV1,
        approver_refs: Vec<String>,
        execution_permit_id: Option<ExecutionPermitId>,
        approval_grant_id: Option<ApprovalGrantId>,
        refusal_reason: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> Self {
        Self {
            effect_commit_decision_id,
            effect_intent_id,
            effect_preflight_report_id,
            citation,
            obligation_refs,
            approval_obligation_refs,
            decision_state,
            approver_refs,
            execution_permit_id,
            approval_grant_id,
            refusal_reason: refusal_reason.into(),
            generated_at: generated_at.into(),
        }
    }

    /// Builds a validated commit decision artifact.
    pub fn build(self) -> Result<EffectCommitDecisionV1, EffectRuntimeValidationError> {
        let value = EffectCommitDecisionV1 {
            schema_version: EffectCommitDecisionV1::SCHEMA_VERSION.to_string(),
            effect_commit_decision_id: self.effect_commit_decision_id,
            effect_intent_id: self.effect_intent_id,
            effect_preflight_report_id: self.effect_preflight_report_id,
            citation: self.citation,
            obligation_refs: self.obligation_refs,
            approval_obligation_refs: self.approval_obligation_refs,
            decision_state: self.decision_state,
            approver_refs: self.approver_refs,
            execution_permit_id: self.execution_permit_id,
            approval_grant_id: self.approval_grant_id,
            refusal_reason: self.refusal_reason,
            generated_at: self.generated_at,
        };
        value.validate()?;
        Ok(value)
    }
}
