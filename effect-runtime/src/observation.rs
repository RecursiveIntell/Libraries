use crate::error::{
    require_id, require_non_empty, require_non_empty_slice, require_schema_version,
    EffectRuntimeValidationError, EffectValidationResult,
};
use crate::v25::V25ConstitutionCitation;
use crate::vocab::{
    ClosureRecommendationV1, ExecutionStateV1, ExternalObservationStateV1, ObservationStateV1,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    EffectCommitDecisionId, EffectExecutionReceiptId, EffectObservationBundleId,
    ExternalEffectLedgerEntryId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectExecutionReceiptV1 {
    pub schema_version: String,
    pub effect_execution_receipt_id: EffectExecutionReceiptId,
    pub effect_commit_decision_id: EffectCommitDecisionId,
    /// Carries EffectiveConstitutionId and CompiledObligationSetId via the canonical v25 citation.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub provider_route: Vec<String>,
    pub execution_state: ExecutionStateV1,
    pub partial_execution: bool,
    pub cancellation_reason: String,
    pub timeout_exhausted: bool,
    pub generated_effect_refs: Vec<String>,
    pub observed_external_handle: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectObservationBundleV1 {
    pub schema_version: String,
    pub effect_observation_bundle_id: EffectObservationBundleId,
    pub effect_execution_receipt_id: EffectExecutionReceiptId,
    /// Carries EffectiveConstitutionId and CompiledObligationSetId via the canonical v25 citation.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub observation_window: String,
    pub observation_state: ObservationStateV1,
    pub observed_outcome: String,
    pub drift_summary: String,
    pub external_evidence_refs: Vec<String>,
    pub closure_recommendation: ClosureRecommendationV1,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalEffectLedgerEntryV1 {
    pub schema_version: String,
    pub external_effect_ledger_entry_id: ExternalEffectLedgerEntryId,
    pub effect_execution_receipt_id: EffectExecutionReceiptId,
    /// Carries EffectiveConstitutionId and CompiledObligationSetId via the canonical v25 citation.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub target_system: String,
    pub external_handle: String,
    pub externally_observed_state: ExternalObservationStateV1,
    pub entered_at: String,
    pub immutable: bool,
}

impl EffectExecutionReceiptV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectExecutionReceiptV1";

    /// Returns a builder for a validated execution receipt artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        effect_execution_receipt_id: EffectExecutionReceiptId,
        effect_commit_decision_id: EffectCommitDecisionId,
        citation: V25ConstitutionCitation,
        provider_route: Vec<String>,
        execution_state: ExecutionStateV1,
        partial_execution: bool,
        cancellation_reason: impl Into<String>,
        timeout_exhausted: bool,
        generated_effect_refs: Vec<String>,
        observed_external_handle: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> EffectExecutionReceiptV1Builder {
        EffectExecutionReceiptV1Builder::new(
            effect_execution_receipt_id,
            effect_commit_decision_id,
            citation,
            provider_route,
            execution_state,
            partial_execution,
            cancellation_reason,
            timeout_exhausted,
            generated_effect_refs,
            observed_external_handle,
            generated_at,
        )
    }

    /// Validates the owner-side invariants for an execution receipt.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "EffectExecutionReceiptV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(
            &self.effect_execution_receipt_id,
            "effect_execution_receipt_id",
        )?;
        require_id(&self.effect_commit_decision_id, "effect_commit_decision_id")?;
        require_non_empty_slice(&self.provider_route, "provider_route")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        require_non_empty(&self.observed_external_handle, "observed_external_handle")?;
        if self.partial_execution != matches!(self.execution_state, ExecutionStateV1::Partial) {
            return Err(EffectRuntimeValidationError::InvalidState(
                "partial_execution must match execution_state=partial",
            ));
        }
        if self.timeout_exhausted
            && !matches!(
                self.execution_state,
                ExecutionStateV1::TimedOut | ExecutionStateV1::Partial
            )
        {
            return Err(EffectRuntimeValidationError::InvalidState(
                "timeout_exhausted requires execution_state=timed_out or partial",
            ));
        }
        if matches!(self.execution_state, ExecutionStateV1::Completed)
            && !self.cancellation_reason.trim().is_empty()
        {
            return Err(EffectRuntimeValidationError::InvalidState(
                "completed execution receipts must not carry cancellation_reason",
            ));
        }
        if matches!(
            self.execution_state,
            ExecutionStateV1::Completed | ExecutionStateV1::Partial
        ) {
            require_non_empty_slice(&self.generated_effect_refs, "generated_effect_refs")?;
        }
        Ok(())
    }
}

/// Builder for [`EffectExecutionReceiptV1`].
#[derive(Debug, Clone)]
pub struct EffectExecutionReceiptV1Builder {
    effect_execution_receipt_id: EffectExecutionReceiptId,
    effect_commit_decision_id: EffectCommitDecisionId,
    citation: V25ConstitutionCitation,
    provider_route: Vec<String>,
    execution_state: ExecutionStateV1,
    partial_execution: bool,
    cancellation_reason: String,
    timeout_exhausted: bool,
    generated_effect_refs: Vec<String>,
    observed_external_handle: String,
    generated_at: String,
}

impl EffectExecutionReceiptV1Builder {
    /// Creates a builder with the required execution receipt fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        effect_execution_receipt_id: EffectExecutionReceiptId,
        effect_commit_decision_id: EffectCommitDecisionId,
        citation: V25ConstitutionCitation,
        provider_route: Vec<String>,
        execution_state: ExecutionStateV1,
        partial_execution: bool,
        cancellation_reason: impl Into<String>,
        timeout_exhausted: bool,
        generated_effect_refs: Vec<String>,
        observed_external_handle: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> Self {
        Self {
            effect_execution_receipt_id,
            effect_commit_decision_id,
            citation,
            provider_route,
            execution_state,
            partial_execution,
            cancellation_reason: cancellation_reason.into(),
            timeout_exhausted,
            generated_effect_refs,
            observed_external_handle: observed_external_handle.into(),
            generated_at: generated_at.into(),
        }
    }

    /// Builds a validated execution receipt artifact.
    pub fn build(self) -> Result<EffectExecutionReceiptV1, EffectRuntimeValidationError> {
        let value = EffectExecutionReceiptV1 {
            schema_version: EffectExecutionReceiptV1::SCHEMA_VERSION.to_string(),
            effect_execution_receipt_id: self.effect_execution_receipt_id,
            effect_commit_decision_id: self.effect_commit_decision_id,
            citation: self.citation,
            provider_route: self.provider_route,
            execution_state: self.execution_state,
            partial_execution: self.partial_execution,
            cancellation_reason: self.cancellation_reason,
            timeout_exhausted: self.timeout_exhausted,
            generated_effect_refs: self.generated_effect_refs,
            observed_external_handle: self.observed_external_handle,
            generated_at: self.generated_at,
        };
        value.validate()?;
        Ok(value)
    }
}

impl EffectObservationBundleV1 {
    pub const SCHEMA_VERSION: &'static str = "EffectObservationBundleV1";

    /// Returns a builder for a validated observation bundle artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        effect_observation_bundle_id: EffectObservationBundleId,
        effect_execution_receipt_id: EffectExecutionReceiptId,
        citation: V25ConstitutionCitation,
        observation_window: impl Into<String>,
        observation_state: ObservationStateV1,
        observed_outcome: impl Into<String>,
        drift_summary: impl Into<String>,
        external_evidence_refs: Vec<String>,
        closure_recommendation: ClosureRecommendationV1,
        generated_at: impl Into<String>,
    ) -> EffectObservationBundleV1Builder {
        EffectObservationBundleV1Builder::new(
            effect_observation_bundle_id,
            effect_execution_receipt_id,
            citation,
            observation_window,
            observation_state,
            observed_outcome,
            drift_summary,
            external_evidence_refs,
            closure_recommendation,
            generated_at,
        )
    }

    /// Validates the owner-side invariants for an observation bundle.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "EffectObservationBundleV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(
            &self.effect_observation_bundle_id,
            "effect_observation_bundle_id",
        )?;
        require_id(
            &self.effect_execution_receipt_id,
            "effect_execution_receipt_id",
        )?;
        require_non_empty(&self.observation_window, "observation_window")?;
        require_non_empty(&self.observed_outcome, "observed_outcome")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        if matches!(self.observation_state, ObservationStateV1::Complete) {
            require_non_empty_slice(&self.external_evidence_refs, "external_evidence_refs")?;
        }
        Ok(())
    }
}

/// Builder for [`EffectObservationBundleV1`].
#[derive(Debug, Clone)]
pub struct EffectObservationBundleV1Builder {
    effect_observation_bundle_id: EffectObservationBundleId,
    effect_execution_receipt_id: EffectExecutionReceiptId,
    citation: V25ConstitutionCitation,
    observation_window: String,
    observation_state: ObservationStateV1,
    observed_outcome: String,
    drift_summary: String,
    external_evidence_refs: Vec<String>,
    closure_recommendation: ClosureRecommendationV1,
    generated_at: String,
}

impl EffectObservationBundleV1Builder {
    /// Creates a builder with the required observation bundle fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        effect_observation_bundle_id: EffectObservationBundleId,
        effect_execution_receipt_id: EffectExecutionReceiptId,
        citation: V25ConstitutionCitation,
        observation_window: impl Into<String>,
        observation_state: ObservationStateV1,
        observed_outcome: impl Into<String>,
        drift_summary: impl Into<String>,
        external_evidence_refs: Vec<String>,
        closure_recommendation: ClosureRecommendationV1,
        generated_at: impl Into<String>,
    ) -> Self {
        Self {
            effect_observation_bundle_id,
            effect_execution_receipt_id,
            citation,
            observation_window: observation_window.into(),
            observation_state,
            observed_outcome: observed_outcome.into(),
            drift_summary: drift_summary.into(),
            external_evidence_refs,
            closure_recommendation,
            generated_at: generated_at.into(),
        }
    }

    /// Builds a validated observation bundle artifact.
    pub fn build(self) -> Result<EffectObservationBundleV1, EffectRuntimeValidationError> {
        let value = EffectObservationBundleV1 {
            schema_version: EffectObservationBundleV1::SCHEMA_VERSION.to_string(),
            effect_observation_bundle_id: self.effect_observation_bundle_id,
            effect_execution_receipt_id: self.effect_execution_receipt_id,
            citation: self.citation,
            observation_window: self.observation_window,
            observation_state: self.observation_state,
            observed_outcome: self.observed_outcome,
            drift_summary: self.drift_summary,
            external_evidence_refs: self.external_evidence_refs,
            closure_recommendation: self.closure_recommendation,
            generated_at: self.generated_at,
        };
        value.validate()?;
        Ok(value)
    }
}

impl ExternalEffectLedgerEntryV1 {
    pub const SCHEMA_VERSION: &'static str = "ExternalEffectLedgerEntryV1";

    /// Returns a builder for a validated external ledger entry artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        external_effect_ledger_entry_id: ExternalEffectLedgerEntryId,
        effect_execution_receipt_id: EffectExecutionReceiptId,
        citation: V25ConstitutionCitation,
        target_system: impl Into<String>,
        external_handle: impl Into<String>,
        externally_observed_state: ExternalObservationStateV1,
        entered_at: impl Into<String>,
        immutable: bool,
    ) -> ExternalEffectLedgerEntryV1Builder {
        ExternalEffectLedgerEntryV1Builder::new(
            external_effect_ledger_entry_id,
            effect_execution_receipt_id,
            citation,
            target_system,
            external_handle,
            externally_observed_state,
            entered_at,
            immutable,
        )
    }

    /// Validates the owner-side invariants for an external ledger entry.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "ExternalEffectLedgerEntryV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(
            &self.external_effect_ledger_entry_id,
            "external_effect_ledger_entry_id",
        )?;
        require_id(
            &self.effect_execution_receipt_id,
            "effect_execution_receipt_id",
        )?;
        require_non_empty(&self.target_system, "target_system")?;
        require_non_empty(&self.external_handle, "external_handle")?;
        require_non_empty(&self.entered_at, "entered_at")?;
        Ok(())
    }
}

/// Builder for [`ExternalEffectLedgerEntryV1`].
#[derive(Debug, Clone)]
pub struct ExternalEffectLedgerEntryV1Builder {
    external_effect_ledger_entry_id: ExternalEffectLedgerEntryId,
    effect_execution_receipt_id: EffectExecutionReceiptId,
    citation: V25ConstitutionCitation,
    target_system: String,
    external_handle: String,
    externally_observed_state: ExternalObservationStateV1,
    entered_at: String,
    immutable: bool,
}

impl ExternalEffectLedgerEntryV1Builder {
    /// Creates a builder with the required external ledger entry fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        external_effect_ledger_entry_id: ExternalEffectLedgerEntryId,
        effect_execution_receipt_id: EffectExecutionReceiptId,
        citation: V25ConstitutionCitation,
        target_system: impl Into<String>,
        external_handle: impl Into<String>,
        externally_observed_state: ExternalObservationStateV1,
        entered_at: impl Into<String>,
        immutable: bool,
    ) -> Self {
        Self {
            external_effect_ledger_entry_id,
            effect_execution_receipt_id,
            citation,
            target_system: target_system.into(),
            external_handle: external_handle.into(),
            externally_observed_state,
            entered_at: entered_at.into(),
            immutable,
        }
    }

    /// Builds a validated external ledger entry artifact.
    pub fn build(self) -> Result<ExternalEffectLedgerEntryV1, EffectRuntimeValidationError> {
        let value = ExternalEffectLedgerEntryV1 {
            schema_version: ExternalEffectLedgerEntryV1::SCHEMA_VERSION.to_string(),
            external_effect_ledger_entry_id: self.external_effect_ledger_entry_id,
            effect_execution_receipt_id: self.effect_execution_receipt_id,
            citation: self.citation,
            target_system: self.target_system,
            external_handle: self.external_handle,
            externally_observed_state: self.externally_observed_state,
            entered_at: self.entered_at,
            immutable: self.immutable,
        };
        value.validate()?;
        Ok(value)
    }
}
