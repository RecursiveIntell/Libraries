use crate::error::{ContinuityValidationError, ContinuityValidationResult};
use crate::validation::{require_non_empty, require_schema_version};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceLevelProfileV1 {
    pub schema_version: String,
    pub service_level_profile_id: String,
    pub service_name: String,
    pub availability_target: String,
    pub latency_target: String,
    pub owner_ref: String,
    pub error_budget_ledger_id: String,
}

impl ServiceLevelProfileV1 {
    pub const SCHEMA_VERSION: &'static str = "ServiceLevelProfileV1";

    pub fn new(
        service_level_profile_id: impl Into<String>,
        service_name: impl Into<String>,
        availability_target: impl Into<String>,
        latency_target: impl Into<String>,
        owner_ref: impl Into<String>,
        error_budget_ledger_id: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            service_level_profile_id: service_level_profile_id.into(),
            service_name: service_name.into(),
            availability_target: availability_target.into(),
            latency_target: latency_target.into(),
            owner_ref: owner_ref.into(),
            error_budget_ledger_id: error_budget_ledger_id.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.service_level_profile_id, "service_level_profile_id")?;
        require_non_empty(&self.service_name, "service_name")?;
        require_non_empty(&self.availability_target, "availability_target")?;
        require_non_empty(&self.latency_target, "latency_target")?;
        require_non_empty(&self.owner_ref, "owner_ref")?;
        require_non_empty(&self.error_budget_ledger_id, "error_budget_ledger_id")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorBudgetLedgerV1 {
    pub schema_version: String,
    pub error_budget_ledger_id: String,
    pub service_level_profile_id: String,
    pub window: String,
    pub budget_consumed_percent: f64,
    pub remaining_budget_percent: f64,
    pub last_updated_at: String,
}

impl ErrorBudgetLedgerV1 {
    pub const SCHEMA_VERSION: &'static str = "ErrorBudgetLedgerV1";

    pub fn new(
        error_budget_ledger_id: impl Into<String>,
        service_level_profile_id: impl Into<String>,
        window: impl Into<String>,
        budget_consumed_percent: f64,
        remaining_budget_percent: f64,
        last_updated_at: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            error_budget_ledger_id: error_budget_ledger_id.into(),
            service_level_profile_id: service_level_profile_id.into(),
            window: window.into(),
            budget_consumed_percent,
            remaining_budget_percent,
            last_updated_at: last_updated_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.error_budget_ledger_id, "error_budget_ledger_id")?;
        require_non_empty(&self.service_level_profile_id, "service_level_profile_id")?;
        require_non_empty(&self.window, "window")?;
        require_non_empty(&self.last_updated_at, "last_updated_at")?;
        if !(0.0..=100.0).contains(&self.budget_consumed_percent) {
            return Err(ContinuityValidationError::InvalidState(
                "budget_consumed_percent",
            ));
        }
        if !(0.0..=100.0).contains(&self.remaining_budget_percent) {
            return Err(ContinuityValidationError::InvalidState(
                "remaining_budget_percent",
            ));
        }
        Ok(())
    }
}
