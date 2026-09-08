#![deny(unsafe_code)]

pub mod artifact;
pub mod capability;
pub mod error;
pub mod event;
pub mod lease;
pub mod receipt;
pub mod task;

pub use artifact::{ArtifactManifestV1, ArtifactRefV1};
pub use capability::CapabilityManifestV1;
pub use error::ContractError;
pub use event::{TaskEventKindV1, TaskEventV1};
pub use lease::LeaseV1;
pub use receipt::{AuthorityDecisionRefV1, ConflictRecordV1, DeliveryReceiptV1, TaskReceiptV1};
pub use task::{
    ResourceLimitsV1, TaskAcceptanceDispositionV1, TaskAcceptanceV1, TaskAttemptV1, TaskEnvelopeV1,
    TaskStatusV1,
};

pub const CONTRACT_SCHEMA_VERSION: &str = "agent_collaboration_contract_v1";

pub(crate) fn validate_version(value: &str) -> Result<(), ContractError> {
    if value == CONTRACT_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::SchemaVersion(value.to_owned()))
    }
}

pub(crate) fn validate_nonempty(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        Err(ContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_timestamp(
    value: &str,
    field: &'static str,
) -> Result<chrono::DateTime<chrono::Utc>, ContractError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&chrono::Utc))
        .map_err(|_| ContractError::InvalidTimestamp {
            field,
            value: value.to_owned(),
        })
}

pub(crate) fn validate_time_window(start: &str, end: &str) -> Result<(), ContractError> {
    let start_time = validate_timestamp(start, "start")?;
    let end_time = validate_timestamp(end, "end")?;
    if start_time >= end_time {
        return Err(ContractError::InvalidTimeWindow {
            start: start.to_owned(),
            end: end.to_owned(),
        });
    }
    Ok(())
}
