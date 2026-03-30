//! Typed mechanism/theory surface crate for schema publication and bounded fit evaluation.
//!
//! `mechanism-runtime` keeps the compatibility name, but its actual role is narrow:
//! it publishes artifact families plus small helper evaluators and does not implement
//! autonomous orchestration or promotion authority.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   mechanism bundle publication status, theory version advisory state, and fit run dispositions
//!   from semantic-memory projections.
//! - **forge-pilot act phase:** fit run dispositions (eligible for local review vs. blocked)
//!   and simulation contract advisory flags constrain whether mechanism promotions proceed.
//! - **verification-control:** theory refuter suite and rollout stability report case types
//!   are affected; fit run blocking dispositions trigger verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of mechanism
//!   bundle state and fit run evaluation dispositions.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`MechanismBundleV1`] | `mechanism_bundle_v1` | this crate |
//! | [`TheoryVersionV1`] | `theory_version_v1` | this crate |
//! | [`TheoryLibraryV1`] | `theory_library_v1` | this crate |
//! | [`HypothesisLibraryV1`] | `hypothesis_library_v1` | this crate |
//! | [`SimulationContractV1`] | `simulation_contract_v1` | this crate |
//! | [`FitRunV1`] | `fit_run_v1` | this crate |
//! | [`TheoryRefuterSuiteV1`] | `theory_refuter_suite_v1` | this crate |
//! | [`RolloutStabilityReportV1`] | `rollout_stability_report_v1` | this crate |

pub mod error;
pub use error::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    FitRunId, HypothesisLibraryId, MechanismBundleId, RolloutStabilityReportId,
    SimulationContractId, SurfaceStatus, TheoryLibraryId, TheoryRefuterSuiteId, TheoryVersionId,
};

pub const MECHANISM_BUNDLE_V1_SCHEMA: &str = "mechanism_bundle_v1";
pub const THEORY_VERSION_V1_SCHEMA: &str = "theory_version_v1";
pub const THEORY_LIBRARY_V1_SCHEMA: &str = "theory_library_v1";
pub const HYPOTHESIS_LIBRARY_V1_SCHEMA: &str = "hypothesis_library_v1";
pub const SIMULATION_CONTRACT_V1_SCHEMA: &str = "simulation_contract_v1";
pub const FIT_RUN_V1_SCHEMA: &str = "fit_run_v1";
pub const THEORY_REFUTER_SUITE_V1_SCHEMA: &str = "theory_refuter_suite_v1";
pub const ROLLOUT_STABILITY_REPORT_V1_SCHEMA: &str = "rollout_stability_report_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MechanismBundleV1 {
    pub schema_version: String,
    pub mechanism_bundle_id: MechanismBundleId,
    pub mechanism_name: String,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TheoryVersionV1 {
    pub schema_version: String,
    pub theory_version_id: TheoryVersionId,
    pub mechanism_bundle_id: MechanismBundleId,
    pub theory_name: String,
    pub semantic_version: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TheoryLibraryV1 {
    pub schema_version: String,
    pub theory_library_id: TheoryLibraryId,
    #[serde(default)]
    pub theory_version_ids: Vec<TheoryVersionId>,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HypothesisLibraryV1 {
    pub schema_version: String,
    pub hypothesis_library_id: HypothesisLibraryId,
    #[serde(default)]
    pub hypothesis_refs: Vec<String>,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SimulationContractV1 {
    pub schema_version: String,
    pub simulation_contract_id: SimulationContractId,
    pub mechanism_bundle_id: MechanismBundleId,
    pub target_outcome: String,
    #[serde(default)]
    pub required_inputs: Vec<String>,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FitDisposition {
    AdvisoryFitOnly,
    PromotionBlockedMissingRefuter,
    PromotionBlockedFailingRefuter,
    PromotionBlockedStabilityRisk,
    EligibleForLocalReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FitRunV1 {
    pub schema_version: String,
    pub fit_run_id: FitRunId,
    pub mechanism_bundle_id: MechanismBundleId,
    pub theory_version_id: TheoryVersionId,
    pub simulation_contract_id: SimulationContractId,
    pub fit_score: f64,
    pub disposition: FitDisposition,
    pub advisory_only: bool,
    pub degraded: bool,
    pub refuter_ready: bool,
    pub stability_clear: bool,
    pub theory_refuter_suite_id: TheoryRefuterSuiteId,
    pub rollout_stability_report_id: RolloutStabilityReportId,
    #[serde(default)]
    pub notes: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TheoryRefuterSuiteV1 {
    pub schema_version: String,
    pub theory_refuter_suite_id: TheoryRefuterSuiteId,
    pub theory_version_id: TheoryVersionId,
    #[serde(default)]
    pub required_refuters: Vec<String>,
    #[serde(default)]
    pub available_refuters: Vec<String>,
    #[serde(default)]
    pub failing_refuters: Vec<String>,
    pub horizon_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RolloutStabilityReportV1 {
    pub schema_version: String,
    pub rollout_stability_report_id: RolloutStabilityReportId,
    pub theory_version_id: TheoryVersionId,
    pub invariance_summary: String,
    pub advisory_only: bool,
    pub locally_reviewable: bool,
    #[serde(default)]
    pub blocking_findings: Vec<String>,
}

pub fn evaluate_fit_run(
    mechanism: &MechanismBundleV1,
    theory: &TheoryVersionV1,
    contract: &SimulationContractV1,
    refuter_suite: &TheoryRefuterSuiteV1,
    stability_report: &RolloutStabilityReportV1,
    fit_score: f64,
    generated_at: impl Into<String>,
) -> FitRunV1 {
    let missing_required_refuters = refuter_suite
        .required_refuters
        .iter()
        .filter(|required| !refuter_suite.available_refuters.contains(required))
        .cloned()
        .collect::<Vec<_>>();
    let refuter_ready = missing_required_refuters.is_empty()
        && refuter_suite.failing_refuters.is_empty()
        && !refuter_suite.horizon_only;
    let stability_clear = !stability_report.advisory_only
        && stability_report.locally_reviewable
        && stability_report.blocking_findings.is_empty();

    let disposition = if !refuter_ready && !refuter_suite.failing_refuters.is_empty() {
        FitDisposition::PromotionBlockedFailingRefuter
    } else if !refuter_ready {
        FitDisposition::PromotionBlockedMissingRefuter
    } else if !stability_clear {
        FitDisposition::PromotionBlockedStabilityRisk
    } else if fit_score >= 0.8 && !theory.advisory_only && !contract.advisory_only {
        FitDisposition::EligibleForLocalReview
    } else {
        FitDisposition::AdvisoryFitOnly
    };

    let mut notes = Vec::new();
    if !missing_required_refuters.is_empty() {
        notes.push(format!(
            "missing required refuters: {}",
            missing_required_refuters.join(", ")
        ));
    }
    if !refuter_suite.failing_refuters.is_empty() {
        notes.push(format!(
            "failing refuters remain visible: {}",
            refuter_suite.failing_refuters.join(", ")
        ));
    }
    if !stability_report.blocking_findings.is_empty() {
        notes.push(format!(
            "stability blockers: {}",
            stability_report.blocking_findings.join(", ")
        ));
    }
    if matches!(disposition, FitDisposition::EligibleForLocalReview) {
        notes
            .push("fit run is eligible for local review only; no authority is created here".into());
    } else if notes.is_empty() {
        notes.push("fit run remains advisory until typed refuter and stability gates clear".into());
    }

    FitRunV1 {
        schema_version: FIT_RUN_V1_SCHEMA.into(),
        fit_run_id: FitRunId::generate(),
        mechanism_bundle_id: mechanism.mechanism_bundle_id.clone(),
        theory_version_id: theory.theory_version_id.clone(),
        simulation_contract_id: contract.simulation_contract_id.clone(),
        fit_score,
        disposition,
        advisory_only: disposition != FitDisposition::EligibleForLocalReview,
        degraded: disposition != FitDisposition::EligibleForLocalReview,
        refuter_ready,
        stability_clear,
        theory_refuter_suite_id: refuter_suite.theory_refuter_suite_id.clone(),
        rollout_stability_report_id: stability_report.rollout_stability_report_id.clone(),
        notes,
        generated_at: generated_at.into(),
    }
}
