//! Coding-agent profile expansion.

use aidens_contracts::{
    AiDENsAppPlanV1, CanonicalToolSideEffectClass, MemoryModeV1, ReportLevelV1, RiskDisclosureV1,
};

pub const PROFILE_ID: &str = "coding-agent";
pub const SUPPORT_TIER: &str = "supported-local";
pub const NON_GOAL: &str = "not production cloud or broad autonomy";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCodingStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for ProfileCodingStatus {
    fn default() -> Self {
        Self {
            enabled: true,
            note: format!(
                "{PROFILE_ID}: {SUPPORT_TIER}; risky actions require explicit permits; {NON_GOAL}"
            ),
        }
    }
}

pub fn coding_agent_plan(app_id: impl Into<String>) -> AiDENsAppPlanV1 {
    AiDENsAppPlanV1 {
        app_id: app_id.into(),
        profile_id: PROFILE_ID.into(),
        provider_required: true,
        memory_mode: MemoryModeV1::Optional,
        receipt_level: ReportLevelV1::Full,
        dangerous_auto_approval: false,
        risk_disclosures: [
            (
                CanonicalToolSideEffectClass::Write,
                "file writes require an explicit permit",
            ),
            (
                CanonicalToolSideEffectClass::Admin,
                "shell execution requires an explicit permit",
            ),
            (
                CanonicalToolSideEffectClass::Analysis,
                "network access requires an explicit permit",
            ),
        ]
        .into_iter()
        .map(|(risk_class, reason)| RiskDisclosureV1 {
            risk_class,
            granted_by_default: false,
            permit_required: true,
            reason: reason.into(),
        })
        .collect(),
        enabled_tool_bundles: vec![
            "repo-read".into(),
            "repo-list".into(),
            "file-stat".into(),
            "repo-search".into(),
            "patch-propose".into(),
            "patch-apply".into(),
            "run-checks".into(),
        ],
        disabled_tool_bundles: vec![
            "shell-auto".into(),
            "network-auto".into(),
            "file-write-auto".into(),
            "dangerous-auto-approval".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coding_profile_does_not_auto_grant_dangerous_capabilities() {
        let plan = coding_agent_plan("agent");
        assert!(!plan.dangerous_auto_approval);
        assert!(plan
            .risk_disclosures
            .iter()
            .all(|risk| !risk.granted_by_default && risk.permit_required));
        assert!(plan.validate().is_ok());
        let status = ProfileCodingStatus::default();
        assert!(status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status.note.contains("explicit permits"));
    }
}
