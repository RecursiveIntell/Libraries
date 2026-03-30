use crate::{
    approval_matches, ApprovalRecord, PolicyDecision, V25CitationContext, V25ControlObligationRefs,
};
use stack_ids::{
    ApprovalRecordId, CheckPlanId, ExecutionPermitId, PolicyDecisionId, VerificationCaseId,
};
use std::fmt;
use verification_control::{CheckMethod, CheckPlan, PromotionClass, VerificationCase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPermitScope {
    namespace: String,
    target_key: String,
    method: CheckMethod,
    promotion_class: PromotionClass,
}

impl ExecutionPermitScope {
    /// Returns the namespace covered by the permit.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the concrete target key covered by the permit.
    pub fn target_key(&self) -> &str {
        &self.target_key
    }

    /// Returns the verification method covered by the permit.
    pub fn method(&self) -> CheckMethod {
        self.method
    }

    /// Returns the promotion class covered by the permit.
    pub fn promotion_class(&self) -> PromotionClass {
        self.promotion_class
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitToken {
    execution_permit_id: ExecutionPermitId,
    decision_id: PolicyDecisionId,
    seal: seal::RuntimeSeal,
}

impl CommitToken {
    /// Runtime-only commit tokens cannot be forged outside this crate.
    ///
    /// ```compile_fail
    /// use stack_ids::{ExecutionPermitId, PolicyDecisionId};
    /// use verification_policy::CommitToken;
    ///
    /// let _forged = CommitToken {
    ///     execution_permit_id: ExecutionPermitId::new("permit-1"),
    ///     decision_id: PolicyDecisionId::new("decision-1"),
    /// };
    /// ```
    ///
    /// Returns the permit identifier bound to this runtime token.
    pub fn execution_permit_id(&self) -> &ExecutionPermitId {
        &self.execution_permit_id
    }

    /// Returns the policy decision that minted this runtime token.
    pub fn decision_id(&self) -> &PolicyDecisionId {
        &self.decision_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPermit {
    execution_permit_id: ExecutionPermitId,
    decision_id: PolicyDecisionId,
    case_id: VerificationCaseId,
    plan_id: CheckPlanId,
    scope: ExecutionPermitScope,
    approval_record_id: Option<ApprovalRecordId>,
    issued_at: String,
    citation: V25CitationContext,
    obligation_refs: V25ControlObligationRefs,
    commit_token: CommitToken,
}

impl ExecutionPermit {
    /// Returns the permit identifier.
    pub fn execution_permit_id(&self) -> &ExecutionPermitId {
        &self.execution_permit_id
    }

    /// Returns the policy decision that issued the permit.
    pub fn decision_id(&self) -> &PolicyDecisionId {
        &self.decision_id
    }

    /// Returns the case covered by this permit.
    pub fn case_id(&self) -> &VerificationCaseId {
        &self.case_id
    }

    /// Returns the verification plan covered by this permit.
    pub fn plan_id(&self) -> &CheckPlanId {
        &self.plan_id
    }

    /// Returns the effect scope covered by this permit.
    pub fn scope(&self) -> &ExecutionPermitScope {
        &self.scope
    }

    /// Returns the approval record lineage, when human approval was required.
    pub fn approval_record_id(&self) -> Option<&ApprovalRecordId> {
        self.approval_record_id.as_ref()
    }

    /// Returns the issuance timestamp recorded on the decision.
    pub fn issued_at(&self) -> &str {
        &self.issued_at
    }

    /// Returns the constitutional citation lineage bound to this permit.
    pub fn citation(&self) -> &V25CitationContext {
        &self.citation
    }

    /// Returns the obligation references bound to this permit.
    pub fn obligation_refs(&self) -> &V25ControlObligationRefs {
        &self.obligation_refs
    }

    /// Returns the sealed runtime commit token.
    pub fn commit_token(&self) -> &CommitToken {
        &self.commit_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermitIssuanceError {
    DecisionContextMismatch,
    MethodDenied,
    AutonomyDenied,
    PlanNotPromotionCapable,
    ApprovalRequired,
    CitationIncomplete,
    ObligationRefsIncomplete,
    PromotionBlocked,
}

impl PermitIssuanceError {
    /// Returns a stable string discriminant for this permit issuance error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::DecisionContextMismatch => "decision_context_mismatch",
            Self::MethodDenied => "method_denied",
            Self::AutonomyDenied => "autonomy_denied",
            Self::PlanNotPromotionCapable => "plan_not_promotion_capable",
            Self::ApprovalRequired => "approval_required",
            Self::CitationIncomplete => "citation_incomplete",
            Self::ObligationRefsIncomplete => "obligation_refs_incomplete",
            Self::PromotionBlocked => "promotion_blocked",
        }
    }
}

impl fmt::Display for PermitIssuanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DecisionContextMismatch => {
                "policy decision lineage does not match the requested case/plan"
            }
            Self::MethodDenied => "policy denied the requested method",
            Self::AutonomyDenied => "policy denied the requested autonomy level",
            Self::PlanNotPromotionCapable => {
                "advisory-only or non-promotable plans cannot mint execution permits"
            }
            Self::ApprovalRequired => "matching approval is required before execution",
            Self::CitationIncomplete => "constitutional citation context is incomplete",
            Self::ObligationRefsIncomplete => "constitutional obligation refs are incomplete",
            Self::PromotionBlocked => "policy blocked promotion-capable execution",
        };
        f.write_str(message)
    }
}

impl std::error::Error for PermitIssuanceError {}

impl PolicyDecision {
    /// Mints a sealed execution permit when the policy decision still authorizes
    /// one promotable execution for the supplied case and plan.
    pub fn issue_execution_permit(
        &self,
        case: &VerificationCase,
        plan: &CheckPlan,
        approvals: &[ApprovalRecord],
    ) -> Result<ExecutionPermit, PermitIssuanceError> {
        if self.case_id != case.case_id || self.plan_id != plan.plan_id {
            return Err(PermitIssuanceError::DecisionContextMismatch);
        }
        if !self.method_allowed {
            return Err(PermitIssuanceError::MethodDenied);
        }
        if !self.autonomy_allowed {
            return Err(PermitIssuanceError::AutonomyDenied);
        }
        if plan.advisory_only || !plan.promotable_if_completed {
            return Err(PermitIssuanceError::PlanNotPromotionCapable);
        }

        let approval_record_id = if self.approval_required {
            Some(
                approvals
                    .iter()
                    .find(|approval| {
                        approval.policy_version == self.policy_version
                            && approval_matches(approval, case, plan, self.evaluated_at.as_str())
                    })
                    .map(|approval| approval.approval_record_id.clone())
                    .ok_or(PermitIssuanceError::ApprovalRequired)?,
            )
        } else {
            None
        };

        if !self.citation_status.is_complete() {
            return Err(PermitIssuanceError::CitationIncomplete);
        }
        if !self.obligation_refs_status.is_complete() {
            return Err(PermitIssuanceError::ObligationRefsIncomplete);
        }
        if !self.promotion_allowed {
            return Err(PermitIssuanceError::PromotionBlocked);
        }

        let execution_permit_id = ExecutionPermitId::generate();
        let commit_token = CommitToken {
            execution_permit_id: execution_permit_id.clone(),
            decision_id: self.decision_id.clone(),
            seal: seal::RuntimeSeal::new(),
        };

        Ok(ExecutionPermit {
            execution_permit_id,
            decision_id: self.decision_id.clone(),
            case_id: case.case_id.clone(),
            plan_id: plan.plan_id.clone(),
            scope: ExecutionPermitScope {
                namespace: case.region.namespace.clone(),
                target_key: case.region.target_key.clone(),
                method: plan.method,
                promotion_class: plan.promotion_class,
            },
            approval_record_id,
            issued_at: self.evaluated_at.clone(),
            citation: self.citation.clone(),
            obligation_refs: self.obligation_refs.clone(),
            commit_token,
        })
    }
}

impl From<&ExecutionPermit> for llm_tool_runtime::ToolExecutionPermit {
    fn from(permit: &ExecutionPermit) -> Self {
        llm_tool_runtime::ToolExecutionPermit::new(
            permit.execution_permit_id().clone(),
            permit.decision_id().clone(),
            permit.approval_record_id().cloned(),
            permit.scope().namespace(),
            permit.scope().target_key(),
        )
    }
}

mod seal {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RuntimeSeal(());

    impl RuntimeSeal {
        pub(super) fn new() -> Self {
            Self(())
        }
    }
}
