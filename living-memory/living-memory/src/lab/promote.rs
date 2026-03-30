use crate::config::ForgeConfig;
use crate::error::{ForgeError, ForgeResult};
use crate::lab::emitters::AlgebraSpec;
use crate::lab::evaluate::ScoreVector;
use crate::lab::evidence::{AssessmentCategory, ContradictionState, SampleSupport};
use crate::store::db::EvalRunRow;
use crate::store::ForgeStore;

/// BasisVersion — a promoted, immutable algebra version.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BasisVersion {
    pub version_id: String,
    pub candidate_id: String,
    pub frozen_spec: AlgebraSpec,
    pub bounds: ParameterBounds,
    pub checksum: String,
    pub cea_fingerprint: Option<CausalFingerprint>,
    pub promoted_at: String,
}

/// Locked min/max per parameter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterBounds {
    pub k_min: f64,
    pub k_max: f64,
    pub evidence_budget_min: usize,
    pub evidence_budget_max: usize,
    pub token_budget_min: usize,
    pub token_budget_max: usize,
}

impl Default for ParameterBounds {
    fn default() -> Self {
        Self {
            k_min: 1.0,
            k_max: 20.0,
            evidence_budget_min: 1,
            evidence_budget_max: 20,
            token_budget_min: 500,
            token_budget_max: 4000,
        }
    }
}

/// Causal fingerprint — frozen CEA fingerprint for drift detection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CausalFingerprint {
    pub dominant_edge_hashes: Vec<String>,
    pub checksum: String,
}

/// Graduation contract criteria.
pub struct GraduationContract {
    pub min_suite_pass_rate: f64,
    pub min_weighted_improvement: f64,
    pub max_stability_variance: f64,
    pub max_invariant_violations: usize,
    pub max_causal_drift: f64,
}

impl GraduationContract {
    pub fn from_config(config: &ForgeConfig) -> Self {
        Self {
            min_suite_pass_rate: config.lab.promotion_min_suite_pass_rate,
            min_weighted_improvement: config.lab.promotion_min_weighted_improvement,
            max_stability_variance: config.lab.archive.stability_variance_threshold,
            max_invariant_violations: 0,
            max_causal_drift: config.cea.causal_drift_warning_threshold,
        }
    }
}

#[derive(Debug)]
struct CandidatePromotionSnapshot {
    average_suite_pass_rate: f64,
    average_weighted_total: f64,
    stability_variance: f64,
    invariant_violations: usize,
    cea_fingerprint: Option<CausalFingerprint>,
}

/// Promote a candidate to a BasisVersion.
pub fn promote(
    store: &ForgeStore,
    candidate_id: &str,
    config: &ForgeConfig,
) -> ForgeResult<BasisVersion> {
    let contract = GraduationContract::from_config(config);
    let snapshot = candidate_promotion_snapshot(store, candidate_id, config)?;
    enforce_graduation_contract(store, candidate_id, &contract, &snapshot, config)?;

    // Get candidate spec
    let spec_json = store.get_candidate_spec(candidate_id)?;
    let spec: AlgebraSpec = serde_json::from_str(&spec_json)?;

    // Compute checksum
    let bounds = ParameterBounds::default();
    let bounds_json = serde_json::to_string(&bounds)?;
    let invariants_json = "{}"; // invariants are code-enforced, not data-enforced

    let content = format!("{spec_json}{bounds_json}{invariants_json}");
    let checksum = blake3::hash(content.as_bytes()).to_hex().to_string();

    // Assign version_id
    let count = store.count_promotions()?;
    let version_id = format!("v{:04}", count + 1);

    // Compute CEA fingerprint if CEA is enabled
    let cea_fingerprint = snapshot.cea_fingerprint.clone();

    let cea_fingerprint_json = cea_fingerprint
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    let now = chrono::Utc::now().to_rfc3339();

    // Transaction: insert promotion + update candidate status atomically.
    // If either step fails, neither takes effect.
    store.with_transaction(|tx| {
        tx.execute(
            "INSERT INTO promotions (version_id, candidate_id, frozen_spec_json, bounds_json, invariants_json, checksum, cea_fingerprint_json, promoted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![&version_id, candidate_id, &spec_json, &bounds_json, invariants_json, &checksum, cea_fingerprint_json.as_deref(), &now],
        )?;

        tx.execute(
            "UPDATE candidates SET status = ?1 WHERE candidate_id = ?2",
            rusqlite::params!["promoted", candidate_id],
        )?;

        Ok(())
    })?;

    Ok(BasisVersion {
        version_id,
        candidate_id: candidate_id.to_string(),
        frozen_spec: spec,
        bounds,
        checksum,
        cea_fingerprint,
        promoted_at: now,
    })
}

fn enforce_graduation_contract(
    store: &ForgeStore,
    candidate_id: &str,
    contract: &GraduationContract,
    snapshot: &CandidatePromotionSnapshot,
    config: &ForgeConfig,
) -> ForgeResult<()> {
    if snapshot.average_suite_pass_rate < contract.min_suite_pass_rate {
        return Err(ForgeError::PromotionFailed {
            criterion: "suite_pass_rate".into(),
            value: format!(
                "{:.4} < {:.4}",
                snapshot.average_suite_pass_rate, contract.min_suite_pass_rate
            ),
        });
    }

    let baseline_average = latest_baseline_weighted_total(store, candidate_id, config)?;
    let weighted_improvement = snapshot.average_weighted_total - baseline_average;
    if weighted_improvement < contract.min_weighted_improvement {
        return Err(ForgeError::PromotionFailed {
            criterion: "weighted_improvement".into(),
            value: format!(
                "{:.4} < {:.4} (candidate_avg={:.4}, baseline_avg={:.4})",
                weighted_improvement,
                contract.min_weighted_improvement,
                snapshot.average_weighted_total,
                baseline_average
            ),
        });
    }

    if snapshot.stability_variance > contract.max_stability_variance {
        return Err(ForgeError::PromotionFailed {
            criterion: "stability_variance".into(),
            value: format!(
                "{:.4} > {:.4}",
                snapshot.stability_variance, contract.max_stability_variance
            ),
        });
    }

    if snapshot.invariant_violations > contract.max_invariant_violations {
        return Err(ForgeError::PromotionFailed {
            criterion: "invariant_violations".into(),
            value: snapshot.invariant_violations.to_string(),
        });
    }

    enforce_verification_evidence(store, candidate_id)?;

    if config.cea.enabled {
        enforce_causal_drift(store, contract, snapshot)?;
    }

    Ok(())
}

fn candidate_promotion_snapshot(
    store: &ForgeStore,
    candidate_id: &str,
    config: &ForgeConfig,
) -> ForgeResult<CandidatePromotionSnapshot> {
    let runs = store.get_eval_runs_for_candidate(candidate_id)?;
    if runs.is_empty() {
        return Err(ForgeError::PromotionFailed {
            criterion: "eval_runs".into(),
            value: "0".into(),
        });
    }

    let mut suite_sum = 0.0_f64;
    let mut weighted_scores = Vec::with_capacity(runs.len());
    let mut invariant_violations = 0usize;

    for run in &runs {
        let scores: ScoreVector = serde_json::from_str(&run.scores_json)?;
        suite_sum += scores.correctness;
        weighted_scores.push(scores.weighted_total);
        invariant_violations += violation_count(run)?;
    }

    let average_suite_pass_rate = suite_sum / runs.len() as f64;
    let average_weighted_total = weighted_scores.iter().sum::<f64>() / weighted_scores.len() as f64;
    let stability_variance = variance(&weighted_scores);
    let cea_fingerprint = if config.cea.enabled {
        compute_causal_fingerprint(&runs)
    } else {
        None
    };

    Ok(CandidatePromotionSnapshot {
        average_suite_pass_rate,
        average_weighted_total,
        stability_variance,
        invariant_violations,
        cea_fingerprint,
    })
}

fn latest_baseline_weighted_total(
    store: &ForgeStore,
    candidate_id: &str,
    config: &ForgeConfig,
) -> ForgeResult<f64> {
    let Some(baseline) = store.get_latest_promotion()? else {
        return Ok(0.0);
    };

    if baseline.candidate_id == candidate_id {
        return Ok(0.0);
    }

    let baseline_snapshot = candidate_promotion_snapshot(store, &baseline.candidate_id, config)?;
    Ok(baseline_snapshot.average_weighted_total)
}

fn enforce_verification_evidence(store: &ForgeStore, candidate_id: &str) -> ForgeResult<()> {
    let Some(bundle) = store.get_latest_evidence_bundle_for_candidate(candidate_id)? else {
        return Err(ForgeError::PromotionFailed {
            criterion: "verification_evidence".into(),
            value: "missing".into(),
        });
    };

    let bundle = bundle.local_bundle()?;

    let Some(assessment) = bundle.assessment else {
        return Err(ForgeError::PromotionFailed {
            criterion: "verification_assessment".into(),
            value: "missing".into(),
        });
    };

    if !matches!(
        assessment.reproducibility,
        AssessmentCategory::Strong | AssessmentCategory::Adequate
    ) {
        return Err(ForgeError::PromotionFailed {
            criterion: "reproducibility".into(),
            value: format!("{:?}", assessment.reproducibility),
        });
    }

    if assessment.isolation != AssessmentCategory::Strong {
        return Err(ForgeError::PromotionFailed {
            criterion: "isolation".into(),
            value: format!("{:?}", assessment.isolation),
        });
    }

    if assessment.contradiction_state != ContradictionState::Clean {
        return Err(ForgeError::PromotionFailed {
            criterion: "contradiction_state".into(),
            value: format!("{:?}", assessment.contradiction_state),
        });
    }

    if assessment.sample_support == SampleSupport::Insufficient {
        return Err(ForgeError::PromotionFailed {
            criterion: "sample_support".into(),
            value: format!("{:?}", assessment.sample_support),
        });
    }

    Ok(())
}

fn enforce_causal_drift(
    store: &ForgeStore,
    contract: &GraduationContract,
    snapshot: &CandidatePromotionSnapshot,
) -> ForgeResult<()> {
    let Some(candidate) = snapshot.cea_fingerprint.as_ref() else {
        return Ok(());
    };
    let Some(baseline) = store.get_latest_promotion()? else {
        return Ok(());
    };
    let Some(baseline_json) = baseline.cea_fingerprint_json.as_deref() else {
        return Ok(());
    };
    let baseline_fingerprint: CausalFingerprint = serde_json::from_str(baseline_json)?;
    let drift = causal_drift(candidate, &baseline_fingerprint);
    if drift > contract.max_causal_drift {
        return Err(ForgeError::PromotionFailed {
            criterion: "causal_drift".into(),
            value: format!("{:.4} > {:.4}", drift, contract.max_causal_drift),
        });
    }
    Ok(())
}

fn compute_causal_fingerprint(runs: &[EvalRunRow]) -> Option<CausalFingerprint> {
    let mut edge_hashes: Vec<String> = runs.iter().filter_map(|r| r.cea_run_hash.clone()).collect();
    if edge_hashes.is_empty() {
        return None;
    }

    edge_hashes.sort();
    edge_hashes.truncate(20);

    let fp_content = edge_hashes.join(",");
    let fp_checksum = blake3::hash(fp_content.as_bytes()).to_hex().to_string();

    Some(CausalFingerprint {
        dominant_edge_hashes: edge_hashes,
        checksum: fp_checksum,
    })
}

fn causal_drift(candidate: &CausalFingerprint, baseline: &CausalFingerprint) -> f64 {
    let candidate: std::collections::BTreeSet<_> = candidate
        .dominant_edge_hashes
        .iter()
        .map(String::as_str)
        .collect();
    let baseline: std::collections::BTreeSet<_> = baseline
        .dominant_edge_hashes
        .iter()
        .map(String::as_str)
        .collect();

    if candidate.is_empty() && baseline.is_empty() {
        return 0.0;
    }

    let intersection = candidate.intersection(&baseline).count() as f64;
    let denominator = candidate.len().max(baseline.len()) as f64;
    if denominator == 0.0 {
        0.0
    } else {
        1.0 - (intersection / denominator)
    }
}

fn violation_count(run: &EvalRunRow) -> ForgeResult<usize> {
    let value: serde_json::Value = serde_json::from_str(&run.violations_json)?;
    Ok(value.as_array().map_or(0, Vec::len))
}

fn variance(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64
}
