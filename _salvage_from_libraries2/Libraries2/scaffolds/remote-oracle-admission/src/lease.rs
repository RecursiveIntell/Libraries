use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteOracleLeaseV1 {
    pub schema_version: String,
    pub remote_oracle_lease_id: String,
    pub oracle_identity: String,
    pub allowed_artifact_families: Vec<String>,
    pub allowed_graph_or_slice_kinds: Vec<String>,
    pub exactness_class_ceiling: String,
    pub budget_ceiling: String,
    pub disclosure_ceiling: String,
    pub replay_obligation: String,
    pub lease_expiry: String,
    pub policy_owner_refs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CrossRuntimeReplayTicketV1 {
    pub schema_version: String,
    pub cross_runtime_replay_ticket_id: String,
    pub artifact_refs: Vec<String>,
    pub time_coordinates: String,
    pub required_trust_roots: Vec<String>,
    pub allowed_disclosure: String,
    pub lease_window: String,
    pub replay_expectations: Vec<String>,
    pub failure_behavior: String,
}
