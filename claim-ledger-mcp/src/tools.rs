use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetParams {
    pub claim_id: String,
}
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProofDebtParams {
    #[serde(default)]
    pub claim_ids: Vec<String>,
    #[serde(default = "default_budget")]
    pub budget_micros: u64,
}
fn default_budget() -> u64 {
    500_000
}
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportParams {
    #[serde(default)]
    pub claim_ids: Vec<String>,
    #[serde(default = "default_operation")]
    pub operation: String,
    #[serde(default = "default_attempt")]
    pub attempt_id: String,
}
fn default_operation() -> String {
    "claim_ledger_export".into()
}
fn default_attempt() -> String {
    claim_ledger::ulid()
}
