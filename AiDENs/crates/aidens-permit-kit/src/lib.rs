//! Approval and permit model.

use aidens_contracts::{
    ApprovalDecisionV1, ApprovalRequestV1, ArtifactId, CanonicalToolSideEffectClass, PermitGrantV1,
    PermitUseReportV1,
};
use std::collections::BTreeSet;

const UNKNOWN_PERMIT_SCOPE_TOKEN: &str = "unknown";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermitDecisionV1 {
    Allow,
    Deny(String),
    RequiresApproval,
}

pub type PermitV1 = PermitGrantV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermitCheckContextV1 {
    pub tool_id: String,
    pub risk_class: CanonicalToolSideEffectClass,
    pub sandbox_root: String,
    pub run_id: Option<ArtifactId>,
    pub attempt_id: Option<ArtifactId>,
}

impl PermitCheckContextV1 {
    pub fn new(
        tool_id: impl Into<String>,
        risk_class: CanonicalToolSideEffectClass,
        sandbox_root: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            risk_class,
            sandbox_root: sandbox_root.into(),
            run_id: None,
            attempt_id: None,
        }
    }

    pub fn with_run_attempt(
        mut self,
        run_id: Option<ArtifactId>,
        attempt_id: Option<ArtifactId>,
    ) -> Self {
        self.run_id = run_id;
        self.attempt_id = attempt_id;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PermitPolicyV1 {
    granted_risks: BTreeSet<CanonicalToolSideEffectClass>,
    grants: Vec<PermitGrantV1>,
}

impl PermitPolicyV1 {
    pub fn with_permit(mut self, permit: &PermitV1) -> Self {
        self.granted_risks.insert(permit.risk_class.clone());
        self.grants.push(permit.clone());
        self
    }

    pub fn with_grant(mut self, grant: PermitGrantV1) -> Self {
        self.granted_risks.insert(grant.risk_class.clone());
        self.grants.push(grant);
        self
    }

    pub fn decision_for_risk(&self, risk: &CanonicalToolSideEffectClass) -> PermitDecisionV1 {
        if matches!(risk, CanonicalToolSideEffectClass::ReadOnly)
            || self.granted_risks.contains(risk)
        {
            PermitDecisionV1::Allow
        } else {
            PermitDecisionV1::RequiresApproval
        }
    }

    pub fn decision_for_context(&self, context: &PermitCheckContextV1) -> PermitDecisionV1 {
        if matches!(context.risk_class, CanonicalToolSideEffectClass::ReadOnly) {
            return PermitDecisionV1::Allow;
        }
        if self.grant_for_context(context).is_some() {
            PermitDecisionV1::Allow
        } else {
            PermitDecisionV1::RequiresApproval
        }
    }

    pub fn grant_for_risk(&self, risk: &CanonicalToolSideEffectClass) -> Option<&PermitGrantV1> {
        self.grants.iter().find(|grant| &grant.risk_class == risk)
    }

    pub fn grant_for_context(&self, context: &PermitCheckContextV1) -> Option<&PermitGrantV1> {
        self.grants.iter().find(|grant| {
            grant.matches_scope(
                &context.risk_class,
                &context.tool_id,
                &context.sandbox_root,
                context.run_id.as_ref(),
                context.attempt_id.as_ref(),
            )
        })
    }

    pub fn permit_use_receipt_for_context(
        &self,
        context: &PermitCheckContextV1,
    ) -> Option<PermitUseReportV1> {
        self.grant_for_context(context).map(|grant| {
            PermitUseReportV1::allowed(
                grant,
                context.tool_id.clone(),
                context.sandbox_root.clone(),
                context.run_id.clone(),
                context.attempt_id.clone(),
            )
        })
    }

    pub fn approval_request_for_tool(
        &self,
        tool_id: impl Into<String>,
        risk: CanonicalToolSideEffectClass,
        scope: impl Into<String>,
    ) -> Option<ApprovalRequestV1> {
        if self.decision_for_risk(&risk) == PermitDecisionV1::RequiresApproval {
            Some(ApprovalRequestV1::new(
                tool_id,
                risk,
                scope,
                "side-effect tool requires explicit permit",
            ))
        } else {
            None
        }
    }

    pub fn approval_request_for_context(
        &self,
        context: &PermitCheckContextV1,
    ) -> Option<ApprovalRequestV1> {
        if self.decision_for_context(context) == PermitDecisionV1::RequiresApproval {
            let mut request = ApprovalRequestV1::scoped(
                context.tool_id.clone(),
                context.risk_class.clone(),
                context.sandbox_root.clone(),
                "side-effect tool requires explicit scoped permit",
            );
            request.run_id = context.run_id.clone();
            request.attempt_id = context.attempt_id.clone();
            if context.run_id.is_some() || context.attempt_id.is_some() {
                request.scope = format!(
                    "{};run={};attempt={}",
                    request.scope,
                    context
                        .run_id
                        .as_ref()
                        .map(|id| id.as_str())
                        .unwrap_or(UNKNOWN_PERMIT_SCOPE_TOKEN),
                    context
                        .attempt_id
                        .as_ref()
                        .map(|id| id.as_str())
                        .unwrap_or(UNKNOWN_PERMIT_SCOPE_TOKEN)
                );
            }
            Some(request)
        } else {
            None
        }
    }

    pub fn deny_request(
        &self,
        request_id: ArtifactId,
        decided_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> ApprovalDecisionV1 {
        ApprovalDecisionV1::denied(request_id, decided_by, reason)
    }
}

pub fn default_decision(risk: &CanonicalToolSideEffectClass) -> PermitDecisionV1 {
    match risk {
        CanonicalToolSideEffectClass::ReadOnly => PermitDecisionV1::Allow,
        _ => PermitDecisionV1::RequiresApproval,
    }
}

pub fn requires_permit(risk: &CanonicalToolSideEffectClass) -> bool {
    !matches!(risk, CanonicalToolSideEffectClass::ReadOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_requires_approval_by_default() {
        assert_eq!(
            default_decision(&CanonicalToolSideEffectClass::Write),
            PermitDecisionV1::RequiresApproval
        );
    }

    #[test]
    fn explicit_permit_allows_matching_risk() {
        let permit = PermitV1::new(CanonicalToolSideEffectClass::Admin, "repo", "test");
        let policy = PermitPolicyV1::default().with_permit(&permit);

        assert_eq!(
            policy.decision_for_risk(&CanonicalToolSideEffectClass::Admin),
            PermitDecisionV1::Allow
        );
        assert_eq!(
            policy.decision_for_risk(&CanonicalToolSideEffectClass::Write),
            PermitDecisionV1::RequiresApproval
        );
        assert_eq!(
            policy
                .grant_for_risk(&CanonicalToolSideEffectClass::Admin)
                .map(|grant| grant.permit_id.clone()),
            Some(permit.permit_id)
        );
    }

    #[test]
    fn scoped_permit_requires_matching_tool_and_sandbox() {
        let permit = PermitV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:file-write:1",
            "/repo",
            "test",
        );
        let policy = PermitPolicyV1::default().with_permit(&permit);
        let matching = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        );
        let wrong_tool = PermitCheckContextV1::new(
            "aidens:shell:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        );
        let wrong_root = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "/other",
        );

        assert_eq!(
            policy.decision_for_context(&matching),
            PermitDecisionV1::Allow
        );
        assert_eq!(
            policy.decision_for_context(&wrong_tool),
            PermitDecisionV1::RequiresApproval
        );
        assert_eq!(
            policy.decision_for_context(&wrong_root),
            PermitDecisionV1::RequiresApproval
        );
        let receipt = policy
            .permit_use_receipt_for_context(&matching)
            .expect("matching grant emits use receipt");
        assert!(receipt.allowed);
        assert_eq!(receipt.permit_id, permit.permit_id);
    }

    #[test]
    fn approval_request_for_context_marks_missing_ids_explicitly() {
        let policy = PermitPolicyV1::default();
        let context = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "repo",
        )
        .with_run_attempt(Some(ArtifactId::new("run-id:example")), None);

        let request = policy
            .approval_request_for_context(&context)
            .expect("write requests permit approval by default");

        assert!(request.scope.contains("run=run-id:example"));
        assert!(request.scope.contains("attempt=unknown"));
        assert!(request.scope.contains("tool=aidens:file-write:1"));
        assert!(!request.scope.contains("run=*"));
        assert!(!request.scope.contains("attempt=*"));
    }

    #[test]
    fn side_effect_risks_request_approval() {
        let policy = PermitPolicyV1::default();
        let request = policy
            .approval_request_for_tool(
                "aidens:file-write:1",
                CanonicalToolSideEffectClass::Write,
                "repo",
            )
            .expect("file-write requires approval");

        assert_eq!(request.tool_id, "aidens:file-write:1");
        assert_eq!(request.risk_class, CanonicalToolSideEffectClass::Write);
        assert!(requires_permit(&CanonicalToolSideEffectClass::Admin));
    }

    #[test]
    fn default_permit_policy_matches_reference_interpreter() {
        for risk_class in aidens_testkit::all_risk_classes() {
            let case = aidens_testkit::reference_permit_case(risk_class.clone());
            let decision = match default_decision(&risk_class) {
                PermitDecisionV1::Allow => "allow",
                PermitDecisionV1::RequiresApproval => "requires-approval",
                PermitDecisionV1::Deny(_) => "deny",
            };
            let reason_codes = if requires_permit(&risk_class) {
                vec!["approval-required"]
            } else {
                vec!["read-only-risk"]
            };
            let actual = serde_json::json!({
                "risk_class": aidens_testkit::json_string(&risk_class),
                "permit_required": requires_permit(&risk_class),
                "decision": decision,
                "reason_codes": reason_codes
            });
            let report = aidens_testkit::compare_case_to_actual(
                &case,
                "aidens-permit-kit::default_decision",
                actual,
            );

            assert!(
                report.passed,
                "{}",
                report
                    .findings
                    .iter()
                    .map(|finding| finding.human_diff.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }
}
