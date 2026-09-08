use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractError {
    #[error("invalid schema version: {0}")]
    SchemaVersion(String),
    #[error("required field is empty: {0}")]
    EmptyField(&'static str),
    #[error("field exceeds resource limit: {field} ({value} > {limit})")]
    ResourceLimit {
        field: &'static str,
        value: usize,
        limit: usize,
    },
    #[error("invalid timestamp in {field}: {value}")]
    InvalidTimestamp { field: &'static str, value: String },
    #[error("invalid time window: {start} must precede {end}")]
    InvalidTimeWindow { start: String, end: String },
    #[error("invalid digest in {0}")]
    InvalidDigest(&'static str),
    #[error("invalid task transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: crate::TaskStatusV1,
        to: crate::TaskStatusV1,
    },
    #[error("owner mismatch: expected {expected}, got {actual}")]
    OwnerMismatch { expected: String, actual: String },
    #[error("stale lease epoch: expected {expected}, got {actual}")]
    StaleLease { expected: u64, actual: u64 },
    #[error("missing execution lineage: {0}")]
    MissingLineage(&'static str),
}
