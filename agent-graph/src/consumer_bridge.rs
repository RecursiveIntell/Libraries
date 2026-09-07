use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFactV1 {
    pub fact_id: String,
    pub run_id: String,
    pub receipt_digest: String,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationGrantV1 {
    pub app_id: String,
    pub memory_namespace: String,
    pub permitted_fact_ids: BTreeSet<String>,
    pub output_policy_id: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerRequestV1 {
    pub app_id: String,
    pub fact_id: String,
    pub memory_namespace: String,
    pub domain_output: String,
    pub ui_cached_fact: Option<ExecutionFactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerProjectionV1 {
    pub app_id: String,
    pub execution_fact_id: String,
    pub run_id: String,
    pub receipt_digest: String,
    pub artifact_refs: Vec<String>,
    pub output_policy_id: String,
    pub domain_output: String,
    pub memory_namespace: String,
    pub memory_content_included: bool,
    pub canonical_fact_created: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConsumerBridgeError {
    #[error("application grant does not match caller")]
    GrantMismatch,
    #[error("memory namespace does not match the application grant")]
    MemoryNamespaceMismatch,
    #[error("execution fact is not admitted for this application")]
    FactNotPermitted,
    #[error("canonical execution fact is unavailable")]
    FactUnavailable,
    #[error("canonical execution fact identity does not match permitted request")]
    FactIdentityMismatch,
    #[error("domain output check failed")]
    DomainOutputRejected,
    #[error("UI/cache supplied a conflicting canonical fact")]
    ConflictingUiFact,
}

pub trait ConsumerOwnerPort: Send + Sync {
    fn current_grant(&self, app_id: &str) -> Option<ApplicationGrantV1>;
    fn execution_fact(&self, fact_id: &str) -> Option<ExecutionFactV1>;
    fn validate_domain_output(&self, app_id: &str, output_policy_id: &str, output: &str) -> bool;
}

pub fn project_execution_fact_for_application(
    request: &ConsumerRequestV1,
    owner: &dyn ConsumerOwnerPort,
) -> Result<ConsumerProjectionV1, ConsumerBridgeError> {
    let grant = owner
        .current_grant(&request.app_id)
        .ok_or(ConsumerBridgeError::GrantMismatch)?;
    if grant.app_id != request.app_id {
        return Err(ConsumerBridgeError::GrantMismatch);
    }
    if grant.memory_namespace != request.memory_namespace {
        return Err(ConsumerBridgeError::MemoryNamespaceMismatch);
    }
    if !grant.permitted_fact_ids.contains(&request.fact_id) {
        return Err(ConsumerBridgeError::FactNotPermitted);
    }
    let fact = owner
        .execution_fact(&request.fact_id)
        .ok_or(ConsumerBridgeError::FactUnavailable)?;
    if fact.fact_id != request.fact_id {
        return Err(ConsumerBridgeError::FactIdentityMismatch);
    }
    if let Some(cached) = &request.ui_cached_fact {
        if cached != &fact {
            return Err(ConsumerBridgeError::ConflictingUiFact);
        }
    }
    if !owner.validate_domain_output(
        &request.app_id,
        &grant.output_policy_id,
        &request.domain_output,
    ) {
        return Err(ConsumerBridgeError::DomainOutputRejected);
    }
    Ok(ConsumerProjectionV1 {
        app_id: request.app_id.clone(),
        execution_fact_id: fact.fact_id,
        run_id: fact.run_id,
        receipt_digest: fact.receipt_digest,
        artifact_refs: fact.artifact_refs,
        output_policy_id: grant.output_policy_id,
        domain_output: request.domain_output.clone(),
        memory_namespace: grant.memory_namespace,
        memory_content_included: false,
        canonical_fact_created: false,
    })
}

#[derive(Default)]
pub struct InMemoryConsumerOwners {
    pub facts: BTreeMap<String, ExecutionFactV1>,
    pub grants: BTreeMap<String, ApplicationGrantV1>,
    pub accepted_outputs: BTreeMap<String, BTreeSet<String>>,
}

impl ConsumerOwnerPort for InMemoryConsumerOwners {
    fn current_grant(&self, app_id: &str) -> Option<ApplicationGrantV1> {
        self.grants.get(app_id).cloned()
    }

    fn execution_fact(&self, fact_id: &str) -> Option<ExecutionFactV1> {
        self.facts.get(fact_id).cloned()
    }

    fn validate_domain_output(&self, app_id: &str, output_policy_id: &str, output: &str) -> bool {
        self.accepted_outputs
            .get(&format!("{app_id}:{output_policy_id}"))
            .is_some_and(|accepted| accepted.contains(output))
    }
}
