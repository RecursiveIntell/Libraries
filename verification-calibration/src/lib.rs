use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{CalibrationSnapshotId, NuisanceStateId, RegionId};

pub const CALIBRATION_SNAPSHOT_V1_SCHEMA: &str = "calibration_snapshot_v1";
pub const NUISANCE_STATE_ARTIFACT_V1_SCHEMA: &str = "nuisance_state_artifact_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CalibrationSnapshot {
    pub schema_version: String,
    pub snapshot_id: CalibrationSnapshotId,
    pub case_id: stack_ids::VerificationCaseId,
    pub recorded_at: String,
    pub comparability_available: bool,
    pub oracle_calibrated: bool,
    pub abstention_threshold_micros: u64,
    pub observed_risk_micros: u64,
    #[serde(default)]
    pub drift_markers: Vec<String>,
    pub forces_advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NuisanceStateArtifact {
    pub schema_version: String,
    pub nuisance_state_id: NuisanceStateId,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_id: Option<RegionId>,
    pub family: String,
    pub markers: Vec<String>,
    pub semantic_change_confounded: bool,
    pub operator_families: Vec<String>,
}

impl CalibrationSnapshot {
    pub fn evaluate(
        case_id: stack_ids::VerificationCaseId,
        recorded_at: impl Into<String>,
        comparability_available: bool,
        oracle_calibrated: bool,
        abstention_threshold_micros: u64,
        observed_risk_micros: u64,
        drift_markers: Vec<String>,
    ) -> Self {
        let forces_advisory_only = !comparability_available
            || !oracle_calibrated
            || observed_risk_micros >= abstention_threshold_micros
            || !drift_markers.is_empty();
        Self {
            schema_version: CALIBRATION_SNAPSHOT_V1_SCHEMA.into(),
            snapshot_id: CalibrationSnapshotId::generate(),
            case_id,
            recorded_at: recorded_at.into(),
            comparability_available,
            oracle_calibrated,
            abstention_threshold_micros,
            observed_risk_micros,
            drift_markers,
            forces_advisory_only,
        }
    }
}

impl NuisanceStateArtifact {
    pub fn new(
        recorded_at: impl Into<String>,
        region_id: Option<RegionId>,
        family: impl Into<String>,
        markers: Vec<String>,
        semantic_change_confounded: bool,
        operator_families: Vec<String>,
    ) -> Self {
        Self {
            schema_version: NUISANCE_STATE_ARTIFACT_V1_SCHEMA.into(),
            nuisance_state_id: NuisanceStateId::generate(),
            recorded_at: recorded_at.into(),
            region_id,
            family: family.into(),
            markers,
            semantic_change_confounded,
            operator_families,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_comparability_forces_advisory_only() {
        let snapshot = CalibrationSnapshot::evaluate(
            stack_ids::VerificationCaseId::new("case-1"),
            "2026-03-12T00:00:00Z",
            false,
            true,
            500_000,
            100_000,
            vec![],
        );
        assert!(snapshot.forces_advisory_only);
    }

    #[test]
    fn nuisance_state_artifact_is_serializable_and_region_bound() {
        let artifact = NuisanceStateArtifact::new(
            "2026-03-13T00:00:00Z",
            Some(RegionId::new("region:demo")),
            "retrieval_widening_shift",
            vec!["widened_scope".into(), "provider_retry_storm".into()],
            true,
            vec!["kernel_execution".into(), "kernel_oracles".into()],
        );

        assert_eq!(artifact.schema_version, NUISANCE_STATE_ARTIFACT_V1_SCHEMA);
        assert_eq!(artifact.region_id, Some(RegionId::new("region:demo")));
        assert!(artifact.semantic_change_confounded);
    }
}
