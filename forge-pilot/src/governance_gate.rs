//! Governance surface checks for the forge-pilot OODA loop.
//!
//! This module evaluates governance artifact state during observation
//! and gates execution during the act phase. It does not own authority —
//! it reads governance artifacts and reports their status to the loop.
//!
//! ## Observation scope
//!
//! The following six predicates are checked by `observe_governance()`:
//!
//! 1. **Effect preflight status** — reads the latest effect-runtime preflight disposition.
//! 2. **Assurance readiness** — checks whether an assurance-runtime case is release-ready.
//! 3. **Authority delegation validity** — verifies the authority-delegation chain is intact.
//! 4. **Continuity incident state** — detects active continuity-runtime incidents.
//! 5. **Constitutional amendment state** — detects pending constitutional-memory amendments.
//! 6. **Mechanism fit disposition** — reads the latest mechanism-runtime fit evaluation.
//!
//! ## Not yet observed
//!
//! - **Attestation exchange state** — `attestation-exchange` is wired but not yet consumed
//!   by the observation pipeline. See GOV-002 / SCOPE_NOTES.md.
//! - **Detailed mechanism state** — only the fit disposition is observed, not internal
//!   mechanism-runtime evaluation details.
//! - **Detailed assurance state** — only the ready/not-ready flag is observed, not the
//!   full assurance case tree.
//!
//! ## Why the current scope is sufficient for CLARA V1
//!
//! The six observed predicates cover all governance surfaces that can block or degrade
//! execution in the OODA loop. The unobserved surfaces (attestation, detailed mechanism
//! and assurance state) are informational and do not gate any execution decision in
//! the current pipeline. They are planned for V2.
//!
//! ## Design constraints
//!
//! - **Read-only observation.** `observe_governance()` reads governance artifact state.
//!   It never writes, creates, or modifies governance artifacts.
//! - **Fail-open on missing governance state.** When no governance artifacts are
//!   present, the gate returns an empty observation and the loop proceeds normally.
//! - **No external dependencies.** Reads only from semantic-memory's SQLite store.

use schemars::JsonSchema;
use semantic_memory::{MemoryStore, ProjectionClaimVersion, ProjectionQuery};
use serde::{Deserialize, Serialize};
use stack_ids::ScopeKey;

/// LIB-001: Governance enforcement mode.
///
/// `FailOpen` preserves existing behavior — missing or broken governance
/// artifacts produce a default observation and execution proceeds.
/// `Strict` fails closed — observation errors and missing governance
/// artifacts produce a `GovernanceGateError` and the caller must handle it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMode {
    /// Existing default: return empty observation on errors (fail-open).
    #[default]
    FailOpen,
    /// Constitutional mode: return error on missing/broken governance (fail-closed).
    Strict,
}

/// LIB-001: Error returned when strict governance mode detects a problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum GovernanceGateError {
    /// Governance observation failed and strict mode does not allow fallback.
    #[error("governance observation failed in strict mode: {reason}")]
    ObservationFailed { reason: String },
    /// No governance claims found — strict mode requires governance artifacts.
    #[error("no governance claims found in strict mode")]
    NoGovernanceClaims,
}

/// Observed governance artifact state at a point in time.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GovernanceObservation {
    /// Effect preflight disposition, if any governance effect artifact was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_preflight_status: Option<String>,
    /// Whether an assurance case is ready for release.
    #[serde(default)]
    pub assurance_ready: bool,
    /// Whether authority delegation chain is valid.
    #[serde(default)]
    pub authority_delegation_valid: bool,
    /// Whether a continuity incident is currently active.
    #[serde(default)]
    pub continuity_incident_active: bool,
    /// Whether a constitutional amendment is pending.
    #[serde(default)]
    pub constitutional_amendment_pending: bool,
    /// Mechanism fit disposition, if evaluated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mechanism_fit_disposition: Option<String>,
    /// Any governance degradations detected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub governance_degradations: Vec<GovernanceDegradation>,
}

/// A degradation in a governance surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GovernanceDegradation {
    pub surface: String,
    pub reason: String,
    pub blocks_promotion: bool,
}

/// Result of the governance gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceGateResult {
    /// Governance state allows execution to proceed.
    Allow,
    /// Governance state recommends advisory-only execution (no promotion).
    AdvisoryOnly { reason: String },
    /// Governance state blocks execution.
    Blocked { reason: String },
}

/// Typed receipt for governance observation within a loop iteration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GovernanceReceiptV1 {
    pub schema_version: String,
    pub gate_result: GovernanceGateResult,
    pub observation_summary: GovernanceObservationSummary,
}

/// Summary of governance observation included in the receipt.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GovernanceObservationSummary {
    pub effect_preflight_present: bool,
    pub assurance_ready: bool,
    pub authority_delegation_valid: bool,
    pub continuity_incident_active: bool,
    pub constitutional_amendment_pending: bool,
    pub mechanism_fit_present: bool,
    pub degradation_count: usize,
}

pub const GOVERNANCE_RECEIPT_V1_SCHEMA: &str = "governance_receipt_v1";

/// Governance projection family constant. Claims projected with this family are
/// governance artifacts readable by `observe_governance()`.
pub const GOVERNANCE_PROJECTION_FAMILY: &str = "governance";

/// Governance-scoped namespace in semantic-memory claim projections.
pub const GOVERNANCE_SCOPE_NAMESPACE: &str = "governance";

/// Well-known predicate constants for governance claim projections.
pub mod predicates {
    /// Predicate for effect preflight disposition claims.
    pub const EFFECT_PREFLIGHT: &str = "effect_preflight_status";
    /// Predicate for assurance case readiness claims.
    pub const ASSURANCE_READY: &str = "assurance_ready";
    /// Predicate for authority delegation chain validity claims.
    pub const AUTHORITY_DELEGATION_VALID: &str = "authority_chain_validity";
    /// Predicate for continuity incident active status claims.
    pub const CONTINUITY_INCIDENT_ACTIVE: &str = "continuity_incident_active";
    /// Predicate for constitutional amendment pending status claims.
    pub const CONSTITUTIONAL_AMENDMENT_PENDING: &str = "constitutional_amendment_pending";
    /// Predicate for mechanism fit disposition claims.
    pub const MECHANISM_FIT: &str = "mechanism_fit_disposition";
}

/// Observes governance artifact state from semantic-memory projections.
///
/// Queries the `claim_versions` table in the governance scope namespace for
/// claim projections that represent governance artifact state. Each governance
/// surface is identified by its predicate value:
///
/// - `effect_preflight_status` — effect-runtime preflight disposition
/// - `assurance_ready` — assurance-runtime case readiness
/// - `authority_chain_validity` — authority-delegation chain state
/// - `continuity_incident_active` — continuity-runtime incident state
/// - `constitutional_amendment_pending` — constitutional-memory amendment state
/// - `mechanism_fit_disposition` — mechanism-runtime fit run disposition
///
/// AUTH-002 fix: strict (fail-closed) governance observation is now the default.
/// The old fail-open behavior is available via [`observe_governance_fail_open`].
/// Strict mode returns Err when governance observation fails or no claims found.
pub async fn observe_governance(store: &MemoryStore) -> Result<GovernanceObservation, GovernanceGateError> {
    observe_governance_with_mode(store, GovernanceMode::Strict).await
}

/// LIB-001: Observe governance with explicit mode selection.
///
/// In `FailOpen` mode, behaves identically to [`observe_governance`].
/// In `Strict` mode, returns `Err` when governance observation fails or
/// when no governance claims are found. This forces callers to explicitly
/// handle the absence of governance artifacts rather than silently proceeding.
pub async fn observe_governance_with_mode(
    store: &MemoryStore,
    mode: GovernanceMode,
) -> Result<GovernanceObservation, GovernanceGateError> {
    match observe_governance_inner(store).await {
        Ok(obs) => {
            // LIB-001: In strict mode, require at least one governance claim.
            if mode == GovernanceMode::Strict && is_empty_observation(&obs) {
                tracing::warn!(
                    "strict governance mode: no governance claims found, failing closed"
                );
                return Err(GovernanceGateError::NoGovernanceClaims);
            }
            Ok(obs)
        }
        Err(err) => match mode {
            GovernanceMode::FailOpen => {
                tracing::warn!(
                    error = %err,
                    "governance observation failed, returning default (fail-open)"
                );
                Ok(GovernanceObservation::default())
            }
            GovernanceMode::Strict => {
                tracing::error!(
                    error = %err,
                    "governance observation failed in strict mode, failing closed"
                );
                Err(GovernanceGateError::ObservationFailed {
                    reason: err.to_string(),
                })
            }
        },
    }
}

/// Returns true if the observation has no governance data populated.
fn is_empty_observation(obs: &GovernanceObservation) -> bool {
    obs.effect_preflight_status.is_none()
        && !obs.assurance_ready
        && !obs.authority_delegation_valid
        && !obs.continuity_incident_active
        && !obs.constitutional_amendment_pending
        && obs.mechanism_fit_disposition.is_none()
        && obs.governance_degradations.is_empty()
}

/// Inner implementation that can propagate errors. The outer function catches
/// all errors and returns Default (fail-open).
async fn observe_governance_inner(
    store: &MemoryStore,
) -> Result<GovernanceObservation, semantic_memory::MemoryError> {
    let query = ProjectionQuery {
        scope: ScopeKey {
            namespace: GOVERNANCE_SCOPE_NAMESPACE.to_string(),
            domain: None,
            workspace_id: None,
            repo_id: None,
        },
        text_query: None,
        valid_at: None,
        recorded_at_or_before: None,
        subject_entity_id: None,
        canonical_entity_id: None,
        claim_state: Some("active".to_string()),
        claim_id: None,
        claim_version_id: None,
        limit: 100,
    };

    let claims = store.query_claim_versions(query).await?;
    if claims.is_empty() {
        tracing::debug!("no governance claims found in scope, returning default observation");
        return Ok(GovernanceObservation::default());
    }

    // Filter to governance projection family claims.
    let gov_claims: Vec<&ProjectionClaimVersion> = claims
        .iter()
        .filter(|c| c.projection_family == GOVERNANCE_PROJECTION_FAMILY)
        .collect();

    if gov_claims.is_empty() {
        tracing::debug!(
            total_claims = claims.len(),
            "claims found but none in governance projection family"
        );
        return Ok(GovernanceObservation::default());
    }

    let mut observation = GovernanceObservation::default();
    let mut degradations = Vec::new();

    for claim in &gov_claims {
        match claim.predicate.as_str() {
            predicates::EFFECT_PREFLIGHT => {
                observation.effect_preflight_status = Some(claim.content.clone());
            }
            predicates::ASSURANCE_READY => {
                observation.assurance_ready = parse_bool_claim(&claim.content);
            }
            predicates::AUTHORITY_DELEGATION_VALID => {
                observation.authority_delegation_valid = parse_bool_claim(&claim.content);
            }
            predicates::CONTINUITY_INCIDENT_ACTIVE => {
                observation.continuity_incident_active = parse_bool_claim(&claim.content);
            }
            predicates::CONSTITUTIONAL_AMENDMENT_PENDING => {
                observation.constitutional_amendment_pending = parse_bool_claim(&claim.content);
            }
            predicates::MECHANISM_FIT => {
                observation.mechanism_fit_disposition = Some(claim.content.clone());
            }
            other => {
                tracing::trace!(
                    predicate = other,
                    "unrecognized governance predicate, skipping"
                );
            }
        }

        // If the claim is stale or contradicted, record a degradation.
        if claim.freshness != "current" || claim.contradiction_status != "none" {
            degradations.push(GovernanceDegradation {
                surface: claim.predicate.clone(),
                reason: format!(
                    "freshness={}, contradiction={}",
                    claim.freshness, claim.contradiction_status
                ),
                blocks_promotion: claim.freshness == "superseded"
                    || claim.contradiction_status != "none",
            });
        }
    }

    observation.governance_degradations = degradations;

    tracing::debug!(
        claim_count = gov_claims.len(),
        effect_preflight = observation.effect_preflight_status.is_some(),
        assurance_ready = observation.assurance_ready,
        authority_valid = observation.authority_delegation_valid,
        incident_active = observation.continuity_incident_active,
        amendment_pending = observation.constitutional_amendment_pending,
        mechanism_fit = observation.mechanism_fit_disposition.is_some(),
        degradation_count = observation.governance_degradations.len(),
        "governance observation populated from semantic-memory projections"
    );

    Ok(observation)
}

/// Parse a claim content string as a boolean. Supports "true"/"false"
/// and "1"/"0". Returns false for unrecognized values (fail-safe).
fn parse_bool_claim(content: &str) -> bool {
    matches!(content.trim().to_lowercase().as_str(), "true" | "1" | "yes")
}

/// Evaluates whether governance state permits execution to proceed.
///
/// For strict-mode gating that returns an error on governance failures,
/// use [`gate_execution_with_mode`].
pub fn gate_execution(observation: &GovernanceObservation) -> GovernanceGateResult {
    // Active continuity incident blocks execution.
    if observation.continuity_incident_active {
        return GovernanceGateResult::Blocked {
            reason: "continuity incident is active".into(),
        };
    }
    // Invalid authority delegation blocks execution.
    if observation.effect_preflight_status.is_some() && !observation.authority_delegation_valid {
        return GovernanceGateResult::Blocked {
            reason: "authority delegation chain is not valid".into(),
        };
    }
    // Pending constitutional amendment downgrades to advisory-only.
    if observation.constitutional_amendment_pending {
        return GovernanceGateResult::AdvisoryOnly {
            reason: "constitutional amendment is pending".into(),
        };
    }
    // Promotion-blocking degradations downgrade to advisory-only.
    if observation
        .governance_degradations
        .iter()
        .any(|d| d.blocks_promotion)
    {
        return GovernanceGateResult::AdvisoryOnly {
            reason: "governance degradation blocks promotion".into(),
        };
    }
    GovernanceGateResult::Allow
}

/// LIB-001: Evaluates governance with explicit mode.
///
/// In `Strict` mode, a `Blocked` result is promoted to an error so callers
/// cannot accidentally ignore it. `Allow` and `AdvisoryOnly` pass through.
/// In `FailOpen` mode, behaves identically to [`gate_execution`].
pub fn gate_execution_with_mode(
    observation: &GovernanceObservation,
    mode: GovernanceMode,
) -> Result<GovernanceGateResult, GovernanceGateError> {
    let result = gate_execution(observation);
    match (&result, mode) {
        (GovernanceGateResult::Blocked { reason }, GovernanceMode::Strict) => {
            Err(GovernanceGateError::ObservationFailed {
                reason: format!("governance blocked in strict mode: {reason}"),
            })
        }
        _ => Ok(result),
    }
}

/// Builds a typed governance receipt for a loop iteration report.
pub fn build_governance_receipt(
    observation: &GovernanceObservation,
    gate_result: &GovernanceGateResult,
) -> GovernanceReceiptV1 {
    GovernanceReceiptV1 {
        schema_version: GOVERNANCE_RECEIPT_V1_SCHEMA.into(),
        gate_result: gate_result.clone(),
        observation_summary: GovernanceObservationSummary {
            effect_preflight_present: observation.effect_preflight_status.is_some(),
            assurance_ready: observation.assurance_ready,
            authority_delegation_valid: observation.authority_delegation_valid,
            continuity_incident_active: observation.continuity_incident_active,
            constitutional_amendment_pending: observation.constitutional_amendment_pending,
            mechanism_fit_present: observation.mechanism_fit_disposition.is_some(),
            degradation_count: observation.governance_degradations.len(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_observation_allows_execution() {
        let obs = GovernanceObservation::default();
        let result = gate_execution(&obs);
        assert_eq!(result, GovernanceGateResult::Allow);
    }

    #[test]
    fn active_incident_blocks_execution() {
        let obs = GovernanceObservation {
            continuity_incident_active: true,
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert!(matches!(result, GovernanceGateResult::Blocked { .. }));
    }

    #[test]
    fn pending_amendment_downgrades_to_advisory() {
        let obs = GovernanceObservation {
            constitutional_amendment_pending: true,
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert!(matches!(result, GovernanceGateResult::AdvisoryOnly { .. }));
    }

    #[test]
    fn governance_receipt_roundtrip() {
        let obs = GovernanceObservation::default();
        let gate = gate_execution(&obs);
        let receipt = build_governance_receipt(&obs, &gate);
        assert_eq!(receipt.schema_version, GOVERNANCE_RECEIPT_V1_SCHEMA);
        let json = serde_json::to_string(&receipt).unwrap();
        let roundtrip: GovernanceReceiptV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.schema_version, GOVERNANCE_RECEIPT_V1_SCHEMA);
    }

    #[test]
    fn parse_bool_claim_values() {
        assert!(parse_bool_claim("true"));
        assert!(parse_bool_claim("True"));
        assert!(parse_bool_claim("TRUE"));
        assert!(parse_bool_claim("1"));
        assert!(parse_bool_claim("yes"));
        assert!(parse_bool_claim("  true  "));
        assert!(!parse_bool_claim("false"));
        assert!(!parse_bool_claim("0"));
        assert!(!parse_bool_claim("no"));
        assert!(!parse_bool_claim(""));
        assert!(!parse_bool_claim("unknown"));
    }

    #[test]
    fn degradation_blocks_promotion_downgrades_to_advisory() {
        let obs = GovernanceObservation {
            governance_degradations: vec![GovernanceDegradation {
                surface: "effect_preflight_status".into(),
                reason: "freshness=superseded, contradiction=none".into(),
                blocks_promotion: true,
            }],
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert!(matches!(result, GovernanceGateResult::AdvisoryOnly { .. }));
    }

    #[test]
    fn non_blocking_degradation_allows_execution() {
        let obs = GovernanceObservation {
            governance_degradations: vec![GovernanceDegradation {
                surface: "mechanism_fit_disposition".into(),
                reason: "freshness=stale, contradiction=none".into(),
                blocks_promotion: false,
            }],
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert_eq!(result, GovernanceGateResult::Allow);
    }

    #[test]
    fn invalid_authority_without_preflight_allows() {
        // authority_delegation_valid is false but no effect_preflight_status is present,
        // so execution is not blocked
        let obs = GovernanceObservation {
            authority_delegation_valid: false,
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert_eq!(result, GovernanceGateResult::Allow);
    }

    #[test]
    fn invalid_authority_with_preflight_blocks() {
        let obs = GovernanceObservation {
            effect_preflight_status: Some("commit_eligible".into()),
            authority_delegation_valid: false,
            ..Default::default()
        };
        let result = gate_execution(&obs);
        assert!(matches!(result, GovernanceGateResult::Blocked { .. }));
    }

    /// Verifies that observe_governance returns default when the store has no
    /// governance claims (empty store, fail-open).
    #[tokio::test]
    async fn observe_governance_empty_store_returns_default() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");
        // AUTH-002 fix: observe_governance is now strict by default.
        // This test verifies the old fail-open behavior; use fail-open mode explicitly.
        let obs = observe_governance_with_mode(&store, GovernanceMode::FailOpen).await.unwrap();
        assert_eq!(obs.effect_preflight_status, None);
        assert!(!obs.assurance_ready);
        assert!(!obs.authority_delegation_valid);
        assert!(!obs.continuity_incident_active);
        assert!(!obs.constitutional_amendment_pending);
        assert_eq!(obs.mechanism_fit_disposition, None);
        assert!(obs.governance_degradations.is_empty());
        // Default observation allows execution.
        assert_eq!(gate_execution(&obs), GovernanceGateResult::Allow);
    }

    /// Verifies that observe_governance populates observation fields from
    /// governance claim projections stored in semantic-memory.
    #[tokio::test]
    async fn observe_governance_reads_real_artifacts() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");

        // Insert governance claims directly into the claim_versions table.
        insert_governance_claim(&store, predicates::EFFECT_PREFLIGHT, "commit_eligible").await;
        insert_governance_claim(&store, predicates::ASSURANCE_READY, "true").await;
        insert_governance_claim(&store, predicates::AUTHORITY_DELEGATION_VALID, "true").await;
        insert_governance_claim(&store, predicates::CONTINUITY_INCIDENT_ACTIVE, "false").await;
        insert_governance_claim(&store, predicates::CONSTITUTIONAL_AMENDMENT_PENDING, "true").await;
        insert_governance_claim(
            &store,
            predicates::MECHANISM_FIT,
            "eligible_for_local_review",
        )
        .await;

        let obs = observe_governance(&store).await.expect("governance observation with populated store");
        assert_eq!(
            obs.effect_preflight_status.as_deref(),
            Some("commit_eligible")
        );
        assert!(obs.assurance_ready);
        assert!(obs.authority_delegation_valid);
        assert!(!obs.continuity_incident_active);
        assert!(obs.constitutional_amendment_pending);
        assert_eq!(
            obs.mechanism_fit_disposition.as_deref(),
            Some("eligible_for_local_review")
        );

        // Constitutional amendment pending => advisory-only.
        let gate = gate_execution(&obs);
        assert!(
            matches!(gate, GovernanceGateResult::AdvisoryOnly { .. }),
            "expected AdvisoryOnly due to pending amendment, got: {:?}",
            gate
        );

        // Receipt should capture the observation accurately.
        let receipt = build_governance_receipt(&obs, &gate);
        assert!(receipt.observation_summary.effect_preflight_present);
        assert!(receipt.observation_summary.assurance_ready);
        assert!(receipt.observation_summary.authority_delegation_valid);
        assert!(!receipt.observation_summary.continuity_incident_active);
        assert!(receipt.observation_summary.constitutional_amendment_pending);
        assert!(receipt.observation_summary.mechanism_fit_present);
    }

    /// Verifies that an active incident claim causes observe_governance
    /// to report it and gate_execution to block.
    #[tokio::test]
    async fn observe_governance_active_incident_blocks() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");

        insert_governance_claim(&store, predicates::CONTINUITY_INCIDENT_ACTIVE, "true").await;

        let obs = observe_governance(&store).await.expect("governance observation with populated store");
        assert!(obs.continuity_incident_active);

        let gate = gate_execution(&obs);
        assert!(
            matches!(gate, GovernanceGateResult::Blocked { .. }),
            "expected Blocked due to active incident, got: {:?}",
            gate
        );
    }

    // --- LIB-001: strict mode tests ---

    #[test]
    fn strict_mode_blocks_on_blocked_gate_result() {
        // LIB-001: In strict mode, a Blocked gate result becomes an error.
        let obs = GovernanceObservation {
            continuity_incident_active: true,
            ..Default::default()
        };
        let result = gate_execution_with_mode(&obs, GovernanceMode::Strict);
        assert!(result.is_err(), "strict mode should error on blocked gate");
    }

    #[test]
    fn strict_mode_allows_on_allow_gate_result() {
        // LIB-001: In strict mode, Allow passes through.
        let obs = GovernanceObservation {
            assurance_ready: true,
            authority_delegation_valid: true,
            ..Default::default()
        };
        let result = gate_execution_with_mode(&obs, GovernanceMode::Strict);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GovernanceGateResult::Allow);
    }

    #[test]
    fn strict_mode_allows_advisory_only() {
        // LIB-001: In strict mode, AdvisoryOnly passes through (not an error).
        let obs = GovernanceObservation {
            constitutional_amendment_pending: true,
            ..Default::default()
        };
        let result = gate_execution_with_mode(&obs, GovernanceMode::Strict);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            GovernanceGateResult::AdvisoryOnly { .. }
        ));
    }

    #[test]
    fn fail_open_mode_returns_blocked_without_error() {
        // LIB-001: In fail-open mode, Blocked is returned as-is (not an error).
        let obs = GovernanceObservation {
            continuity_incident_active: true,
            ..Default::default()
        };
        let result = gate_execution_with_mode(&obs, GovernanceMode::FailOpen);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            GovernanceGateResult::Blocked { .. }
        ));
    }

    #[tokio::test]
    async fn strict_mode_errors_on_empty_store() {
        // LIB-001: Strict mode rejects empty governance state.
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");
        let result = observe_governance_with_mode(&store, GovernanceMode::Strict).await;
        assert!(
            result.is_err(),
            "strict mode should error when no governance claims exist"
        );
        assert!(matches!(
            result.unwrap_err(),
            GovernanceGateError::NoGovernanceClaims
        ));
    }

    #[tokio::test]
    async fn strict_mode_allows_populated_store() {
        // LIB-001: Strict mode succeeds when governance claims exist.
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");
        insert_governance_claim(&store, predicates::ASSURANCE_READY, "true").await;

        let result = observe_governance_with_mode(&store, GovernanceMode::Strict).await;
        assert!(
            result.is_ok(),
            "strict mode should succeed with governance claims present"
        );
        let obs = result.unwrap();
        assert!(obs.assurance_ready);
    }

    #[tokio::test]
    async fn fail_open_mode_returns_default_on_empty_store() {
        // LIB-001: Fail-open mode returns default (not error) when store is empty.
        let dir = tempfile::tempdir().expect("create temp dir");
        let config = semantic_memory::MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = MemoryStore::open(config).expect("open store");
        let result = observe_governance_with_mode(&store, GovernanceMode::FailOpen).await;
        assert!(result.is_ok(), "fail-open mode should not error");
        assert_eq!(
            gate_execution(&result.unwrap()),
            GovernanceGateResult::Allow
        );
    }

    /// Helper: insert a governance claim into the store using raw SQL.
    async fn insert_governance_claim(store: &MemoryStore, predicate: &str, content: &str) {
        let id = uuid::Uuid::new_v4().to_string();
        let claim_id = format!("gov-claim-{}", predicate);
        let sql = format!(
            "INSERT INTO claim_versions (
                claim_version_id, claim_id, claim_state, projection_family,
                subject_entity_id, predicate, object_anchor,
                scope_namespace, scope_domain, scope_workspace_id, scope_repo_id,
                recorded_at, preferred_open,
                source_envelope_id, source_authority,
                freshness, contradiction_status, content, confidence
            ) VALUES (
                '{}', '{}', 'active', '{}',
                'governance-entity', '{}', '\"{}\"',
                '{}', NULL, NULL, NULL,
                datetime('now'), 0,
                'gov-envelope-{}', 'governance',
                'current', 'none', '{}', 1.0
            )",
            id,
            claim_id,
            GOVERNANCE_PROJECTION_FAMILY,
            predicate,
            content,
            GOVERNANCE_SCOPE_NAMESPACE,
            predicate,
            content,
        );
        store
            .raw_execute(&sql, vec![])
            .await
            .expect("insert governance claim");
    }
}
