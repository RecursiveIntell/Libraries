//! Audit fixture adapter for SCR-P0A.
//!
//! This crate maps deterministic fixture cases into SCR evaluation inputs. It
//! does not read external systems or decide truth for the referenced evidence.

use schemars::JsonSchema;
use scr_kernel::{
    ControlEvaluationInputV1, Domain, ExternalArtifactRef, ProposedAction, RequestedEffect,
    ScrError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditFixtureCaseV1 {
    pub schema_version: String,
    pub case_id: String,
    pub domain: Domain,
    pub proposed_action: ProposedAction,
    pub requested_effect: RequestedEffect,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<ExternalArtifactRef>,
}

impl AuditFixtureCaseV1 {
    pub const SCHEMA_VERSION: &'static str = "audit_fixture_case_v1";

    pub fn validate(&self) -> Result<(), ScrError> {
        if self.schema_version != Self::SCHEMA_VERSION {
            return Err(ScrError::InvalidSchemaVersion {
                artifact: "AuditFixtureCaseV1",
                expected: Self::SCHEMA_VERSION,
                found: self.schema_version.clone(),
            });
        }
        if self.case_id.trim().is_empty() {
            return Err(ScrError::MissingField("case_id"));
        }
        for signal in &self.signals {
            if signal.trim().is_empty() {
                return Err(ScrError::MissingField("signals"));
            }
        }
        for evidence_ref in &self.evidence_refs {
            evidence_ref.validate()?;
        }
        Ok(())
    }

    pub fn into_input(self) -> Result<ControlEvaluationInputV1, ScrError> {
        self.validate()?;
        let mut evidence_refs = self.evidence_refs;
        for signal in self.signals {
            evidence_refs.push(ExternalArtifactRef::new("signal", signal)?);
        }
        Ok(ControlEvaluationInputV1 {
            schema_version: ControlEvaluationInputV1::SCHEMA_VERSION.to_string(),
            input_id: self.case_id.clone(),
            actor_ref: ExternalArtifactRef::with_owner_hint(
                "actor_ref",
                "fixture_operator",
                "fixture",
            )?,
            permit_ref: ExternalArtifactRef::with_owner_hint(
                "permit_ref",
                "fixture_permit",
                "fixture",
            )?,
            subject_ref: ExternalArtifactRef::with_owner_hint(
                "subject_ref",
                self.case_id,
                "fixture",
            )?,
            domain: self.domain,
            proposed_action: self.proposed_action,
            requested_effect: self.requested_effect,
            evidence_refs,
            environment_ref: ExternalArtifactRef::with_owner_hint(
                "environment_ref",
                "fixture_environment",
                "fixture",
            )?,
            valid_time_basis: "2026-05-13T00:00:00Z".to_string(),
            recorded_time: "2026-05-13T00:00:00Z".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_preserves_signals_as_external_refs() {
        let case = AuditFixtureCaseV1 {
            schema_version: AuditFixtureCaseV1::SCHEMA_VERSION.to_string(),
            case_id: "case_001".to_string(),
            domain: Domain::Audit,
            proposed_action: ProposedAction::Analyze,
            requested_effect: RequestedEffect::AdvisoryOnly,
            signals: vec!["source_truth_drift".to_string()],
            evidence_refs: Vec::new(),
        };

        let input = case.into_input().unwrap();

        assert!(input
            .evidence_refs
            .iter()
            .any(|item| item.ref_kind == "signal" && item.ref_value == "source_truth_drift"));
    }
}
