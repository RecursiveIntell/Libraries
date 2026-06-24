//! P17 — Attested exchange, federation, and external artifact admission.
//!
//! External artifacts can influence but not overwrite local truth.

use serde::{Deserialize, Serialize};

/// Admission decision for an external artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionDecisionV1 {
    pub decision_id: String,
    pub artifact_id: String,
    pub source_runtime: String,
    pub decision: AdmissionDisposition,
    pub rationale: String,
    pub trust_root: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionDisposition {
    Admitted,
    Quarantined,
    Rejected,
}

/// Federation adapter — manages external artifact admission.
pub struct FederationAdapter;

impl FederationAdapter {
    pub fn admit_external_artifact(
        artifact_id: &str,
        source_runtime: &str,
        trust_root: &str,
    ) -> AdmissionDecisionV1 {
        // External artifacts are always quarantined first, never auto-admitted
        AdmissionDecisionV1 {
            decision_id: format!("ad:{}:{}", artifact_id, chrono::Utc::now().timestamp()),
            artifact_id: artifact_id.to_string(),
            source_runtime: source_runtime.to_string(),
            decision: AdmissionDisposition::Quarantined,
            rationale: "External artifacts require explicit admission review".to_string(),
            trust_root: trust_root.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_artifact_admission_cannot_overwrite_local_truth() {
        let decision = FederationAdapter::admit_external_artifact(
            "ext:001",
            "remote-runtime",
            "trust-root-1",
        );
        assert_eq!(decision.decision, AdmissionDisposition::Quarantined);
        assert!(decision.rationale.contains("require explicit admission"));
    }

    #[test]
    fn admission_decision_has_trust_root() {
        let decision = FederationAdapter::admit_external_artifact("ext:001", "remote", "tr1");
        assert!(!decision.trust_root.is_empty());
    }
}