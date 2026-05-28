use scr_kernel::{ControlAction, ReasonCode, ScoreBps, ScrError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyModelV1 {
    pub policy: PolicyHeaderV1,
    pub action_precedence: BTreeMap<String, u16>,
    pub thresholds: PolicyThresholdsV1,
    pub minimum_actions: BTreeMap<String, String>,
    pub hard_rules: BTreeMap<String, HardRulePolicyV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyHeaderV1 {
    pub id: String,
    pub version: String,
    pub domain: String,
    pub algorithm_version: String,
    pub canonicalization: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyThresholdsV1 {
    pub autonomy_pressure: BTreeMap<String, u16>,
    pub verification_pressure: BTreeMap<String, u16>,
    pub repair_priority: BTreeMap<String, u16>,
    pub quarantine_pressure: BTreeMap<String, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardRulePolicyV1 {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_action: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPolicyV1 {
    pub model: PolicyModelV1,
    pub canonical_json: String,
    pub canonical_hash: String,
}

pub const SUPPORTED_HARD_RULE_IDS: &[&str] = &[
    "HR_SCHEMA_INVALID",
    "HR_AUTHORITY_MISSING",
    "HR_FORBIDDEN_PRODUCTION_TERM",
    "HR_UNKNOWN_OWNER_MUTATION",
    "HR_SOURCE_TRUTH_DRIFT",
    "HR_FALSE_COMPLETION_MISSING_TESTS",
    "HR_DESTRUCTIVE_MISSING_ROLLBACK",
];

pub const SUPPORTED_MINIMUM_ACTION_SIGNALS: &[&str] = &[
    "high_hazard",
    "low_hazard",
    "source_truth_drift",
    "false_completion_missing_tests",
    "destructive_missing_rollback",
    "forbidden_production_term",
    "invalid_schema",
    "authority_missing",
    "unknown_owner_mutation",
    "confirmed",
    "uncertain",
    "fixable",
    "false_completion",
    "missing_tests",
];

pub fn is_supported_minimum_action_signal(value: &str) -> bool {
    SUPPORTED_MINIMUM_ACTION_SIGNALS
        .iter()
        .any(|supported| supported == &value)
}

impl PolicyModelV1 {
    pub fn from_toml(source: &str) -> Result<Self, ScrError> {
        let model: Self =
            toml::from_str(source).map_err(|err| ScrError::PolicyParseFailed(err.to_string()))?;
        model.validate()?;
        Ok(model)
    }

    pub fn validate(&self) -> Result<(), ScrError> {
        require_text(&self.policy.id, "policy.id")?;
        require_text(&self.policy.version, "policy.version")?;
        require_text(&self.policy.domain, "policy.domain")?;
        require_text(&self.policy.algorithm_version, "policy.algorithm_version")?;
        if self.policy.canonicalization != "canonical-json-v1" {
            return Err(ScrError::PolicyValidationFailed(
                "policy.canonicalization must be canonical-json-v1".to_string(),
            ));
        }

        let mut seen_precedence = BTreeSet::new();
        for (action, precedence) in &self.action_precedence {
            parse_action(action)?;
            if !seen_precedence.insert(*precedence) {
                return Err(ScrError::PolicyValidationFailed(format!(
                    "duplicate action precedence: {precedence}"
                )));
            }
        }
        for action in ControlAction::all_names() {
            if !self.action_precedence.contains_key(*action) {
                return Err(ScrError::PolicyValidationFailed(format!(
                    "missing action precedence for {action}"
                )));
            }
        }

        validate_thresholds(&self.thresholds.autonomy_pressure)?;
        validate_thresholds(&self.thresholds.verification_pressure)?;
        validate_thresholds(&self.thresholds.repair_priority)?;
        validate_thresholds(&self.thresholds.quarantine_pressure)?;

        for (floor, action) in &self.minimum_actions {
            if !is_supported_minimum_action_signal(floor) {
                return Err(ScrError::PolicyValidationFailed(format!(
                    "unsupported minimum_actions key: {floor}"
                )));
            }
            require_text(floor, "minimum_actions key")?;
            parse_action(action)?;
        }

        for (rule_id, rule) in &self.hard_rules {
            require_text(rule_id, "hard_rules key")?;
            if let Some(action) = &rule.action {
                parse_action(action)?;
            }
            if let Some(action) = &rule.minimum_action {
                parse_action(action)?;
            }
            if rule.action.is_none() && rule.minimum_action.is_none() {
                return Err(ScrError::PolicyValidationFailed(format!(
                    "hard rule {rule_id} has no action or minimum_action"
                )));
            }
            ReasonCode::new(rule.reason.clone())?;
        }
        validate_hard_rule_registry(&self.hard_rules)?;
        validate_minimum_action_registry(&self.minimum_actions)?;
        Ok(())
    }

    pub fn canonicalize(self) -> Result<CanonicalPolicyV1, ScrError> {
        let value = serde_json::to_value(&self)
            .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
        let canonical_value = canonicalize_json_value(value);
        let canonical_json = serde_json::to_string_pretty(&canonical_value)
            .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
        let canonical_hash = blake3::hash(canonical_json.as_bytes()).to_hex().to_string();
        Ok(CanonicalPolicyV1 {
            model: self,
            canonical_json,
            canonical_hash,
        })
    }

    pub fn action_precedence(&self, action: &ControlAction) -> Result<u16, ScrError> {
        let name = action.name();
        self.action_precedence.get(name).copied().ok_or_else(|| {
            ScrError::PolicyValidationFailed(format!("missing precedence for {name}"))
        })
    }
}

pub fn parse_action(value: &str) -> Result<ControlAction, ScrError> {
    match value {
        "allow_with_receipt" => Ok(ControlAction::AllowWithReceipt),
        "backlog" => Ok(ControlAction::Backlog),
        "require_source_basis" => Ok(ControlAction::RequireSourceBasis),
        "require_verification" => Ok(ControlAction::RequireVerification),
        "require_approval" => Ok(ControlAction::RequireApproval),
        "generate_repair_packet" => Ok(ControlAction::GenerateRepairPacket),
        "require_owner_resolution" => Ok(ControlAction::RequireOwnerResolution),
        "block_mutation" => Ok(ControlAction::BlockMutation),
        "block_release" => Ok(ControlAction::BlockRelease),
        "quarantine_artifact" => Ok(ControlAction::QuarantineArtifact),
        other => Err(ScrError::PolicyValidationFailed(format!(
            "unknown action: {other}"
        ))),
    }
}

fn validate_thresholds(thresholds: &BTreeMap<String, u16>) -> Result<(), ScrError> {
    for (action, value) in thresholds {
        parse_action(action)?;
        ScoreBps::new(*value)?;
    }
    Ok(())
}

fn require_text(value: &str, field: &'static str) -> Result<(), ScrError> {
    if value.trim().is_empty() {
        Err(ScrError::PolicyValidationFailed(format!(
            "{field} must be non-empty"
        )))
    } else {
        Ok(())
    }
}

fn validate_hard_rule_registry(
    hard_rules: &BTreeMap<String, HardRulePolicyV1>,
) -> Result<(), ScrError> {
    let supported = SUPPORTED_HARD_RULE_IDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for rule_id in hard_rules.keys() {
        if !supported.contains(rule_id.as_str()) {
            return Err(ScrError::PolicyValidationFailed(format!(
                "unknown hard rule: {rule_id}"
            )));
        }
    }
    for rule_id in SUPPORTED_HARD_RULE_IDS {
        if !hard_rules.contains_key(*rule_id) {
            return Err(ScrError::PolicyValidationFailed(format!(
                "missing hard rule: {rule_id}"
            )));
        }
    }
    Ok(())
}

fn validate_minimum_action_registry(
    minimum_actions: &BTreeMap<String, String>,
) -> Result<(), ScrError> {
    for rule in minimum_actions.keys() {
        if !is_supported_minimum_action_signal(rule) {
            return Err(ScrError::PolicyValidationFailed(format!(
                "unsupported minimum_action signal: {rule}"
            )));
        }
    }
    Ok(())
}

pub fn canonicalize_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json_value(value)))
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut ordered = serde_json::Map::new();
            for (key, value) in entries {
                ordered.insert(key, value);
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(canonicalize_json_value).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_hard_rule_in_policy() {
        let mut policy =
            PolicyModelV1::from_toml(include_str!("../../../policies/audit_policy_v1.toml"))
                .unwrap();
        policy.hard_rules.insert(
            "HR_UNKNOWN".to_string(),
            HardRulePolicyV1 {
                enabled: true,
                action: None,
                minimum_action: Some("allow_with_receipt".to_string()),
                reason: "invalid".to_string(),
            },
        );
        assert_eq!(
            policy.validate().unwrap_err().kind(),
            "policy_validation_failed"
        );
    }

    #[test]
    fn rejects_unknown_minimum_action_signal() {
        let mut policy =
            PolicyModelV1::from_toml(include_str!("../../../policies/audit_policy_v1.toml"))
                .unwrap();
        policy.minimum_actions.insert(
            "does_not_exist".to_string(),
            "allow_with_receipt".to_string(),
        );
        assert_eq!(
            policy.validate().unwrap_err().kind(),
            "policy_validation_failed"
        );
    }

    #[test]
    fn rejects_unknown_action_name() {
        let mut policy =
            PolicyModelV1::from_toml(include_str!("../../../policies/audit_policy_v1.toml"))
                .unwrap();
        policy
            .action_precedence
            .insert("nonsense_action".to_string(), 999);
        policy
            .thresholds
            .autonomy_pressure
            .insert("nonsense_action".to_string(), 100);
        assert_eq!(
            policy.validate().unwrap_err().kind(),
            "policy_validation_failed"
        );
    }

    #[test]
    fn accepts_valid_model_textual() {
        let policy =
            PolicyModelV1::from_toml(include_str!("../../../policies/audit_policy_v1.toml"))
                .unwrap();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.policy.domain, "audit".to_string());
    }
}
