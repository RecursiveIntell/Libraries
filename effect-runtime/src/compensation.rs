//! Compensation lifecycle artifacts: plans and execution receipts for reversing or mitigating effects.
//!
//! When an effect fails or needs rollback, a compensation plan defines the
//! recovery steps, and a compensation execution receipt records their outcome.

use crate::error::{
    require_id, require_non_empty, require_non_empty_slice, require_schema_version,
    EffectRuntimeValidationError, EffectValidationResult,
};
use crate::v25::V25ConstitutionCitation;
use crate::vocab::{CompensationClassV1, ExecutionStateV1};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{CompensationExecutionReceiptId, CompensationPlanId, EffectIntentId};

/// Plan describing the compensation steps required to reverse or mitigate a failed effect.
///
/// When `compensation_required` is true, `compensation_steps` must be non-empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompensationPlanV1 {
    pub schema_version: String,
    pub compensation_plan_id: CompensationPlanId,
    pub effect_intent_id: EffectIntentId,
    /// Carries EffectiveConstitutionId and CompiledObligationSetId via the canonical v25 citation.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub compensation_required: bool,
    pub compensation_class: CompensationClassV1,
    pub compensation_steps: Vec<String>,
    pub success_criteria: Vec<String>,
    pub owner_ref: String,
    pub latest_start: String,
    pub advisory_only: bool,
}

/// Receipt recording the outcome of executing a compensation plan, including completed and failed steps.
///
/// Completed receipts must not list any failed steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompensationExecutionReceiptV1 {
    pub schema_version: String,
    pub compensation_execution_receipt_id: CompensationExecutionReceiptId,
    pub compensation_plan_id: CompensationPlanId,
    /// Carries EffectiveConstitutionId and CompiledObligationSetId via the canonical v25 citation.
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub execution_state: ExecutionStateV1,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub residual_risk_summary: String,
    pub generated_at: String,
}

impl CompensationPlanV1 {
    pub const SCHEMA_VERSION: &'static str = "CompensationPlanV1";

    /// Returns a builder for a validated compensation plan artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        compensation_plan_id: CompensationPlanId,
        effect_intent_id: EffectIntentId,
        citation: V25ConstitutionCitation,
        compensation_required: bool,
        compensation_class: CompensationClassV1,
        compensation_steps: Vec<String>,
        success_criteria: Vec<String>,
        owner_ref: impl Into<String>,
        latest_start: impl Into<String>,
        advisory_only: bool,
    ) -> CompensationPlanV1Builder {
        CompensationPlanV1Builder::new(
            compensation_plan_id,
            effect_intent_id,
            citation,
            compensation_required,
            compensation_class,
            compensation_steps,
            success_criteria,
            owner_ref,
            latest_start,
            advisory_only,
        )
    }

    /// Validates the owner-side invariants for a compensation plan.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "CompensationPlanV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(&self.compensation_plan_id, "compensation_plan_id")?;
        require_id(&self.effect_intent_id, "effect_intent_id")?;
        require_non_empty_slice(&self.success_criteria, "success_criteria")?;
        require_non_empty(&self.owner_ref, "owner_ref")?;
        require_non_empty(&self.latest_start, "latest_start")?;
        if self.compensation_required {
            require_non_empty_slice(&self.compensation_steps, "compensation_steps")?;
        }
        Ok(())
    }
}

/// Builder for [`CompensationPlanV1`].
#[derive(Debug, Clone)]
pub struct CompensationPlanV1Builder {
    compensation_plan_id: CompensationPlanId,
    effect_intent_id: EffectIntentId,
    citation: V25ConstitutionCitation,
    compensation_required: bool,
    compensation_class: CompensationClassV1,
    compensation_steps: Vec<String>,
    success_criteria: Vec<String>,
    owner_ref: String,
    latest_start: String,
    advisory_only: bool,
}

impl CompensationPlanV1Builder {
    /// Creates a builder with the required compensation plan fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        compensation_plan_id: CompensationPlanId,
        effect_intent_id: EffectIntentId,
        citation: V25ConstitutionCitation,
        compensation_required: bool,
        compensation_class: CompensationClassV1,
        compensation_steps: Vec<String>,
        success_criteria: Vec<String>,
        owner_ref: impl Into<String>,
        latest_start: impl Into<String>,
        advisory_only: bool,
    ) -> Self {
        Self {
            compensation_plan_id,
            effect_intent_id,
            citation,
            compensation_required,
            compensation_class,
            compensation_steps,
            success_criteria,
            owner_ref: owner_ref.into(),
            latest_start: latest_start.into(),
            advisory_only,
        }
    }

    /// Builds a validated compensation plan artifact.
    pub fn build(self) -> Result<CompensationPlanV1, EffectRuntimeValidationError> {
        let value = CompensationPlanV1 {
            schema_version: CompensationPlanV1::SCHEMA_VERSION.to_string(),
            compensation_plan_id: self.compensation_plan_id,
            effect_intent_id: self.effect_intent_id,
            citation: self.citation,
            compensation_required: self.compensation_required,
            compensation_class: self.compensation_class,
            compensation_steps: self.compensation_steps,
            success_criteria: self.success_criteria,
            owner_ref: self.owner_ref,
            latest_start: self.latest_start,
            advisory_only: self.advisory_only,
        };
        value.validate()?;
        Ok(value)
    }
}

impl CompensationExecutionReceiptV1 {
    pub const SCHEMA_VERSION: &'static str = "CompensationExecutionReceiptV1";

    /// Returns a builder for a validated compensation execution receipt artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn builder(
        compensation_execution_receipt_id: CompensationExecutionReceiptId,
        compensation_plan_id: CompensationPlanId,
        citation: V25ConstitutionCitation,
        execution_state: ExecutionStateV1,
        completed_steps: Vec<String>,
        failed_steps: Vec<String>,
        residual_risk_summary: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> CompensationExecutionReceiptV1Builder {
        CompensationExecutionReceiptV1Builder::new(
            compensation_execution_receipt_id,
            compensation_plan_id,
            citation,
            execution_state,
            completed_steps,
            failed_steps,
            residual_risk_summary,
            generated_at,
        )
    }

    /// Validates the owner-side invariants for a compensation execution receipt.
    pub fn validate(&self) -> EffectValidationResult {
        require_schema_version(
            "CompensationExecutionReceiptV1",
            Self::SCHEMA_VERSION,
            &self.schema_version,
        )?;
        require_id(
            &self.compensation_execution_receipt_id,
            "compensation_execution_receipt_id",
        )?;
        require_id(&self.compensation_plan_id, "compensation_plan_id")?;
        require_non_empty(&self.residual_risk_summary, "residual_risk_summary")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        if matches!(self.execution_state, ExecutionStateV1::Completed)
            && !self.failed_steps.is_empty()
        {
            return Err(EffectRuntimeValidationError::InvalidState(
                "completed compensation receipts must not list failed_steps",
            ));
        }
        Ok(())
    }
}

/// Builder for [`CompensationExecutionReceiptV1`].
#[derive(Debug, Clone)]
pub struct CompensationExecutionReceiptV1Builder {
    compensation_execution_receipt_id: CompensationExecutionReceiptId,
    compensation_plan_id: CompensationPlanId,
    citation: V25ConstitutionCitation,
    execution_state: ExecutionStateV1,
    completed_steps: Vec<String>,
    failed_steps: Vec<String>,
    residual_risk_summary: String,
    generated_at: String,
}

impl CompensationExecutionReceiptV1Builder {
    /// Creates a builder with the required compensation execution receipt fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        compensation_execution_receipt_id: CompensationExecutionReceiptId,
        compensation_plan_id: CompensationPlanId,
        citation: V25ConstitutionCitation,
        execution_state: ExecutionStateV1,
        completed_steps: Vec<String>,
        failed_steps: Vec<String>,
        residual_risk_summary: impl Into<String>,
        generated_at: impl Into<String>,
    ) -> Self {
        Self {
            compensation_execution_receipt_id,
            compensation_plan_id,
            citation,
            execution_state,
            completed_steps,
            failed_steps,
            residual_risk_summary: residual_risk_summary.into(),
            generated_at: generated_at.into(),
        }
    }

    /// Builds a validated compensation execution receipt artifact.
    pub fn build(self) -> Result<CompensationExecutionReceiptV1, EffectRuntimeValidationError> {
        let value = CompensationExecutionReceiptV1 {
            schema_version: CompensationExecutionReceiptV1::SCHEMA_VERSION.to_string(),
            compensation_execution_receipt_id: self.compensation_execution_receipt_id,
            compensation_plan_id: self.compensation_plan_id,
            citation: self.citation,
            execution_state: self.execution_state,
            completed_steps: self.completed_steps,
            failed_steps: self.failed_steps,
            residual_risk_summary: self.residual_risk_summary,
            generated_at: self.generated_at,
        };
        value.validate()?;
        Ok(value)
    }
}
