pub struct RemoteOracleLoopPlan {
    pub lease_id: String,
    pub request_id: String,
    pub admission_policy_id: String,
    pub stop_rules: Vec<String>,
    pub escalation_rules: Vec<String>,
    pub budget_class: String,
}

pub struct RemoteOracleLoopOutcome {
    pub result_ref: Option<String>,
    pub dispute_ref: Option<String>,
    pub downgraded_to_advisory: bool,
    pub notes: Vec<String>,
}
