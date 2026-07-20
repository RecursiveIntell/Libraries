//! Operator-approved, bounded authority for the autonomous learning lane.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::error::{
    require_non_empty, require_non_empty_slice, AuthorityValidationError, AuthorityValidationResult,
};
use crate::sod::DualControlApprovalV1;

/// Effects which may be delegated to the learning controller.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum LearningAuthorizedActionV1 {
    ExecuteAndVerify,
    PromoteCandidate,
    ReplayProcedure,
    PublishTerminalEvidence,
}

/// The durable request submitted for operator approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRequestV1 {
    pub schema_version: String,
    pub request_id: String,
    pub principal: String,
    pub controller_id: String,
    pub plan_digest: String,
    pub allowed_actions: BTreeSet<LearningAuthorizedActionV1>,
    pub store_scope_digests: BTreeSet<String>,
    pub publication_namespace: String,
    pub image_digest: String,
    pub maximum_transitions: u64,
    pub maximum_executions: u64,
    pub expires_at: String,
    pub may_derive_scoped_child_permits: bool,
    pub requested_by: String,
    pub request_digest: String,
}

/// Minimal append-only request projection used by the authority functions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LearningAuthorityRequestStoreV1 {
    requests: BTreeMap<String, ApprovalRequestV1>,
}

impl LearningAuthorityRequestStoreV1 {
    pub fn get(&self, request_id: &str) -> Option<&ApprovalRequestV1> {
        self.requests.get(request_id)
    }
}

/// An operator-approved lease. Its scope is the ceiling for every child permit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LearningAutonomyLeaseV1 {
    pub schema_version: String,
    pub lease_id: String,
    pub principal: String,
    pub controller_id: String,
    pub plan_digest: String,
    pub allowed_actions: BTreeSet<LearningAuthorizedActionV1>,
    pub store_scope_digests: BTreeSet<String>,
    pub publication_namespace: String,
    pub image_digest: String,
    pub maximum_transitions: u64,
    pub maximum_executions: u64,
    pub expires_at: String,
    pub may_derive_scoped_child_permits: bool,
    pub approval_receipt_refs: BTreeSet<String>,
    pub lease_digest: String,
}

/// A non-wildcard permit derived from an approved learning lease.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LearningPermitGrantV1 {
    pub schema_version: String,
    pub permit_id: String,
    pub lease_id: String,
    pub controller_id: String,
    pub action: LearningAuthorizedActionV1,
    pub plan_digest: String,
    pub store_scope_digests: BTreeSet<String>,
    pub publication_namespace: String,
    pub image_digest: String,
    pub maximum_transitions: u64,
    pub maximum_executions: u64,
    pub expires_at: String,
    pub permit_digest: String,
}

/// Compatibility name for callers that use the existing authority vocabulary.
pub type PermitGrantV1 = LearningPermitGrantV1;

#[allow(clippy::too_many_arguments)]
fn validate_common(
    principal: &str,
    controller_id: &str,
    plan_digest: &str,
    actions: &BTreeSet<LearningAuthorizedActionV1>,
    stores: &BTreeSet<String>,
    namespace: &str,
    image_digest: &str,
    expires_at: &str,
    transitions: u64,
    executions: u64,
) -> AuthorityValidationResult {
    require_non_empty(principal, "principal")?;
    require_non_empty(controller_id, "controller_id")?;
    require_non_empty(plan_digest, "plan_digest")?;
    if actions.is_empty() {
        return Err(AuthorityValidationError::MissingField("allowed_actions"));
    }
    if stores.is_empty() {
        return Err(AuthorityValidationError::MissingField(
            "store_scope_digests",
        ));
    }
    require_non_empty(namespace, "publication_namespace")?;
    require_non_empty(image_digest, "image_digest")?;
    require_non_empty(expires_at, "expires_at")?;
    if transitions == 0 {
        return Err(AuthorityValidationError::InvalidState(
            "maximum_transitions",
        ));
    }
    if executions == 0 {
        return Err(AuthorityValidationError::InvalidState("maximum_executions"));
    }
    Ok(())
}

impl ApprovalRequestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_id: impl Into<String>,
        principal: impl Into<String>,
        controller_id: impl Into<String>,
        plan_digest: impl Into<String>,
        allowed_actions: BTreeSet<LearningAuthorizedActionV1>,
        store_scope_digests: BTreeSet<String>,
        publication_namespace: impl Into<String>,
        image_digest: impl Into<String>,
        maximum_transitions: u64,
        maximum_executions: u64,
        expires_at: impl Into<String>,
        may_derive_scoped_child_permits: bool,
        requested_by: impl Into<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "ApprovalRequestV1".into(),
            request_id: request_id.into(),
            principal: principal.into(),
            controller_id: controller_id.into(),
            plan_digest: plan_digest.into(),
            allowed_actions,
            store_scope_digests,
            publication_namespace: publication_namespace.into(),
            image_digest: image_digest.into(),
            maximum_transitions,
            maximum_executions,
            expires_at: expires_at.into(),
            may_derive_scoped_child_permits,
            requested_by: requested_by.into(),
            request_digest: request_digest.into(),
        };
        require_non_empty(&value.request_id, "request_id")?;
        validate_common(
            &value.principal,
            &value.controller_id,
            &value.plan_digest,
            &value.allowed_actions,
            &value.store_scope_digests,
            &value.publication_namespace,
            &value.image_digest,
            &value.expires_at,
            value.maximum_transitions,
            value.maximum_executions,
        )?;
        require_non_empty(&value.requested_by, "requested_by")?;
        require_non_empty(&value.request_digest, "request_digest")?;
        Ok(value)
    }
}

/// Persist a request after validating it; an existing request id cannot be replaced.
pub fn persist_learning_authority_request(
    store: &mut LearningAuthorityRequestStoreV1,
    request: ApprovalRequestV1,
) -> Result<ApprovalRequestV1, AuthorityValidationError> {
    request.validate()?;
    if store
        .requests
        .insert(request.request_id.clone(), request.clone())
        .is_some()
    {
        return Err(AuthorityValidationError::InvalidState(
            "request_id_already_persisted",
        ));
    }
    Ok(request)
}

impl ApprovalRequestV1 {
    pub fn validate(&self) -> AuthorityValidationResult {
        Self::new(
            self.request_id.clone(),
            self.principal.clone(),
            self.controller_id.clone(),
            self.plan_digest.clone(),
            self.allowed_actions.clone(),
            self.store_scope_digests.clone(),
            self.publication_namespace.clone(),
            self.image_digest.clone(),
            self.maximum_transitions,
            self.maximum_executions,
            self.expires_at.clone(),
            self.may_derive_scoped_child_permits,
            self.requested_by.clone(),
            self.request_digest.clone(),
        )
        .map(|_| ())
    }
}

/// Turn a persisted request into a lease only with an independent dual-control approval.
pub fn approve_learning_autonomy_lease(
    store: &LearningAuthorityRequestStoreV1,
    request_id: &str,
    approval: &DualControlApprovalV1,
    lease_id: impl Into<String>,
    lease_digest: impl Into<String>,
) -> Result<LearningAutonomyLeaseV1, AuthorityValidationError> {
    let request = store
        .get(request_id)
        .ok_or(AuthorityValidationError::InvalidState(
            "request_not_persisted",
        ))?;
    approval.validate()?;
    if !approval.independence_verified
        || approval.approver_refs.iter().any(|ref_id| {
            ref_id == &request.principal
                || ref_id == &request.controller_id
                || ref_id == &request.requested_by
        })
    {
        return Err(AuthorityValidationError::InvalidState(
            "approval_identity_not_independent",
        ));
    }
    let mut approval_refs: BTreeSet<String> = approval.approver_refs.iter().cloned().collect();
    if approval_refs.len() != approval.approver_refs.len() {
        return Err(AuthorityValidationError::InvalidState(
            "approver_refs_not_distinct",
        ));
    }
    approval_refs.insert(approval.dual_control_approval_id.clone());
    let lease = LearningAutonomyLeaseV1 {
        schema_version: "LearningAutonomyLeaseV1".into(),
        lease_id: lease_id.into(),
        principal: request.principal.clone(),
        controller_id: request.controller_id.clone(),
        plan_digest: request.plan_digest.clone(),
        allowed_actions: request.allowed_actions.clone(),
        store_scope_digests: request.store_scope_digests.clone(),
        publication_namespace: request.publication_namespace.clone(),
        image_digest: request.image_digest.clone(),
        maximum_transitions: request.maximum_transitions,
        maximum_executions: request.maximum_executions,
        expires_at: request.expires_at.clone(),
        may_derive_scoped_child_permits: request.may_derive_scoped_child_permits,
        approval_receipt_refs: approval_refs,
        lease_digest: lease_digest.into(),
    };
    lease.validate()?;
    Ok(lease)
}

impl LearningAutonomyLeaseV1 {
    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.lease_id, "lease_id")?;
        validate_common(
            &self.principal,
            &self.controller_id,
            &self.plan_digest,
            &self.allowed_actions,
            &self.store_scope_digests,
            &self.publication_namespace,
            &self.image_digest,
            &self.expires_at,
            self.maximum_transitions,
            self.maximum_executions,
        )?;
        require_non_empty_slice(
            &self.approval_receipt_refs.iter().collect::<Vec<_>>(),
            "approval_receipt_refs",
        )?;
        require_non_empty(&self.lease_digest, "lease_digest")?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn derive(
    lease: &LearningAutonomyLeaseV1,
    action: LearningAuthorizedActionV1,
    controller_id: &str,
    now: &str,
    maximum_transitions: u64,
    maximum_executions: u64,
    permit_id: impl Into<String>,
    permit_digest: impl Into<String>,
) -> Result<PermitGrantV1, AuthorityValidationError> {
    lease.validate()?;
    require_non_empty(controller_id, "controller_id")?;
    require_non_empty(now, "now")?;
    if controller_id != lease.controller_id {
        return Err(AuthorityValidationError::InvalidState(
            "controller_mismatch",
        ));
    }
    if now >= lease.expires_at.as_str() {
        return Err(AuthorityValidationError::InvalidState("lease_expired"));
    }
    if !lease.may_derive_scoped_child_permits {
        return Err(AuthorityValidationError::InvalidState(
            "child_permit_derivation_not_delegated",
        ));
    }
    if !lease.allowed_actions.contains(&action) {
        return Err(AuthorityValidationError::InvalidState(
            "action_not_delegated",
        ));
    }
    if maximum_transitions == 0 || maximum_transitions > lease.maximum_transitions {
        return Err(AuthorityValidationError::InvalidState(
            "transition_budget_widened",
        ));
    }
    if maximum_executions == 0 || maximum_executions > lease.maximum_executions {
        return Err(AuthorityValidationError::InvalidState(
            "execution_budget_widened",
        ));
    }
    Ok(PermitGrantV1 {
        schema_version: "LearningPermitGrantV1".into(),
        permit_id: permit_id.into(),
        lease_id: lease.lease_id.clone(),
        controller_id: lease.controller_id.clone(),
        action,
        plan_digest: lease.plan_digest.clone(),
        store_scope_digests: lease.store_scope_digests.clone(),
        publication_namespace: lease.publication_namespace.clone(),
        image_digest: lease.image_digest.clone(),
        maximum_transitions,
        maximum_executions,
        expires_at: lease.expires_at.clone(),
        permit_digest: permit_digest.into(),
    })
}

pub fn derive_lifecycle_permit(
    lease: &LearningAutonomyLeaseV1,
    controller_id: &str,
    now: &str,
    maximum_transitions: u64,
    maximum_executions: u64,
    permit_id: impl Into<String>,
    permit_digest: impl Into<String>,
) -> Result<PermitGrantV1, AuthorityValidationError> {
    derive(
        lease,
        LearningAuthorizedActionV1::ExecuteAndVerify,
        controller_id,
        now,
        maximum_transitions,
        maximum_executions,
        permit_id,
        permit_digest,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn derive_action_permit(
    lease: &LearningAutonomyLeaseV1,
    action: LearningAuthorizedActionV1,
    controller_id: &str,
    now: &str,
    maximum_transitions: u64,
    maximum_executions: u64,
    permit_id: impl Into<String>,
    permit_digest: impl Into<String>,
) -> Result<PermitGrantV1, AuthorityValidationError> {
    derive(
        lease,
        action,
        controller_id,
        now,
        maximum_transitions,
        maximum_executions,
        permit_id,
        permit_digest,
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn request() -> ApprovalRequestV1 {
        ApprovalRequestV1::new(
            "request-1",
            "operator:owner",
            "controller:learning",
            "sha256:plan",
            [
                LearningAuthorizedActionV1::ExecuteAndVerify,
                LearningAuthorizedActionV1::ReplayProcedure,
            ]
            .into_iter()
            .collect(),
            ["sha256:store".to_string()].into_iter().collect(),
            "aidens-learning",
            "sha256:image",
            4,
            2,
            "2026-12-31T00:00:00Z",
            true,
            "operator:owner",
            "sha256:request",
        )
        .expect("request")
    }

    fn approval() -> DualControlApprovalV1 {
        DualControlApprovalV1::new(
            "approval-1",
            vec!["operator:alice".into(), "operator:bob".into()],
            "approved bounded learning plan",
            true,
            "2026-07-20T00:00:00Z",
        )
        .expect("approval")
    }

    #[test]
    fn approval_requires_persisted_request_and_independent_operators() {
        let mut store = LearningAuthorityRequestStoreV1::default();
        let persisted = persist_learning_authority_request(&mut store, request()).expect("persist");
        assert_eq!(persisted.request_id, "request-1");
        let lease = approve_learning_autonomy_lease(
            &store,
            "request-1",
            &approval(),
            "lease-1",
            "sha256:lease",
        )
        .expect("lease");
        assert_eq!(lease.approval_receipt_refs.len(), 3);

        let mut self_approval = approval();
        self_approval.approver_refs[0] = "controller:learning".into();
        assert_eq!(
            approve_learning_autonomy_lease(
                &store,
                "request-1",
                &self_approval,
                "lease-2",
                "sha256:lease"
            ),
            Err(AuthorityValidationError::InvalidState(
                "approval_identity_not_independent"
            ))
        );
    }

    #[test]
    fn child_permits_are_bounded_by_lease() {
        let mut store = LearningAuthorityRequestStoreV1::default();
        persist_learning_authority_request(&mut store, request()).expect("persist");
        let lease = approve_learning_autonomy_lease(
            &store,
            "request-1",
            &approval(),
            "lease-1",
            "sha256:lease",
        )
        .expect("lease");
        let permit = derive_action_permit(
            &lease,
            LearningAuthorizedActionV1::ReplayProcedure,
            "controller:learning",
            "2026-07-20T01:00:00Z",
            2,
            1,
            "permit-1",
            "sha256:permit",
        )
        .expect("permit");
        assert_eq!(permit.maximum_transitions, 2);
        assert_eq!(permit.maximum_executions, 1);

        assert_eq!(
            derive_action_permit(
                &lease,
                LearningAuthorizedActionV1::ReplayProcedure,
                "controller:learning",
                "2026-07-20T01:00:00Z",
                5,
                1,
                "permit-2",
                "sha256:permit"
            ),
            Err(AuthorityValidationError::InvalidState(
                "transition_budget_widened"
            ))
        );
        assert_eq!(
            derive_action_permit(
                &lease,
                LearningAuthorizedActionV1::PublishTerminalEvidence,
                "controller:learning",
                "2026-07-20T01:00:00Z",
                1,
                1,
                "permit-3",
                "sha256:permit"
            ),
            Err(AuthorityValidationError::InvalidState(
                "action_not_delegated"
            ))
        );
        assert_eq!(
            derive_lifecycle_permit(
                &lease,
                "controller:learning",
                "2027-01-01T00:00:00Z",
                1,
                1,
                "permit-4",
                "sha256:permit"
            ),
            Err(AuthorityValidationError::InvalidState("lease_expired"))
        );
    }
}
