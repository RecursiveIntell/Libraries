//! Deterministic, nonauthorizing maintained-audit evaluation.
//!
//! The evaluator preserves the frozen experiment plan, every scheduled outcome,
//! and every incurred cost.  It may estimate and report; release authority stays
//! outside this module.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArmId {
    Baseline,
    Intervention,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedInputs {
    pub task_digest: String,
    pub source_digest: String,
    pub requirements_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceControls {
    pub max_provider_cost_microunits: u64,
    pub max_duration_ms: u64,
    pub max_specialists: u32,
}

impl ResourceControls {
    fn is_explicit(&self) -> bool {
        self.max_provider_cost_microunits > 0
            && self.max_duration_ms > 0
            && self.max_specialists > 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationArm {
    pub id: ArmId,
    pub admitted: AdmittedInputs,
    pub resources: ResourceControls,
    pub factors: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinguishedFactor {
    pub name: String,
    pub baseline_value: String,
    pub intervention_value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Estimand {
    MeanPairedQualityDifference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDesign {
    ControlledAblation,
    NaiveArmAverages,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledTrial {
    pub scheduled_trial_id: String,
    pub task_id: String,
    pub arm: ArmId,
}

impl ScheduledTrial {
    pub fn new(
        scheduled_trial_id: impl Into<String>,
        task_id: impl Into<String>,
        arm: ArmId,
    ) -> Self {
        Self {
            scheduled_trial_id: scheduled_trial_id.into(),
            task_id: task_id.into(),
            arm,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationPlan {
    pub baseline: EvaluationArm,
    pub intervention: EvaluationArm,
    pub distinguished_factor: DistinguishedFactor,
    pub estimand: Estimand,
    pub analysis_design: AnalysisDesign,
    pub scheduled_trials: Vec<ScheduledTrial>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvaluationError {
    TaskMismatch,
    SourceMismatch,
    RequirementsMismatch,
    ResourceControlsMismatch,
    InvalidResourceControls,
    DistinguishedFactorMismatch,
    UnmatchedFactors,
    InvalidArmIdentity,
    DuplicateScheduledTrial(String),
    DuplicateTrialRecord(String),
    UnknownScheduledTrial(String),
    MissingScheduledTrials(Vec<String>),
    InvalidInterval,
    ResourceCeilingExceeded(String),
}

impl EvaluationPlan {
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.baseline.id != ArmId::Baseline || self.intervention.id != ArmId::Intervention {
            return Err(EvaluationError::InvalidArmIdentity);
        }
        if self.baseline.admitted.task_digest != self.intervention.admitted.task_digest {
            return Err(EvaluationError::TaskMismatch);
        }
        if self.baseline.admitted.source_digest != self.intervention.admitted.source_digest {
            return Err(EvaluationError::SourceMismatch);
        }
        if self.baseline.admitted.requirements_digest
            != self.intervention.admitted.requirements_digest
        {
            return Err(EvaluationError::RequirementsMismatch);
        }
        if !self.baseline.resources.is_explicit() || !self.intervention.resources.is_explicit() {
            return Err(EvaluationError::InvalidResourceControls);
        }
        if self.baseline.resources != self.intervention.resources {
            return Err(EvaluationError::ResourceControlsMismatch);
        }

        let distinguished = &self.distinguished_factor;
        let baseline_value = self.baseline.factors.get(&distinguished.name);
        let intervention_value = self.intervention.factors.get(&distinguished.name);
        if distinguished.baseline_value == distinguished.intervention_value
            || baseline_value != Some(&distinguished.baseline_value)
            || intervention_value != Some(&distinguished.intervention_value)
        {
            return Err(EvaluationError::DistinguishedFactorMismatch);
        }

        let factor_names: BTreeSet<&String> = self
            .baseline
            .factors
            .keys()
            .chain(self.intervention.factors.keys())
            .collect();
        if factor_names.iter().any(|name| {
            name.as_str() != distinguished.name
                && self.baseline.factors.get(*name) != self.intervention.factors.get(*name)
        }) {
            return Err(EvaluationError::UnmatchedFactors);
        }
        if self.baseline.factors.len() != self.intervention.factors.len() {
            return Err(EvaluationError::UnmatchedFactors);
        }

        let mut scheduled_ids = BTreeSet::new();
        for trial in &self.scheduled_trials {
            if !scheduled_ids.insert(trial.scheduled_trial_id.clone()) {
                return Err(EvaluationError::DuplicateScheduledTrial(
                    trial.scheduled_trial_id.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialCost {
    pub provider_cost_microunits: u64,
    pub duration_ms: u64,
}

impl TrialCost {
    fn zero() -> Self {
        Self {
            provider_cost_microunits: 0,
            duration_ms: 0,
        }
    }

    fn add_assign(&mut self, other: &Self) {
        self.provider_cost_microunits = self
            .provider_cost_microunits
            .saturating_add(other.provider_cost_microunits);
        self.duration_ms = self.duration_ms.saturating_add(other.duration_ms);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialFailure {
    Timeout,
    BudgetExhausted,
    ProviderFailure,
    InvalidOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TrialOutcome {
    Succeeded { quality: f64 },
    Failed { failure: TrialFailure },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialRecord {
    pub scheduled_trial_id: String,
    pub outcome: TrialOutcome,
    pub cost: TrialCost,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialAggregate {
    pub scheduled_count: usize,
    pub succeeded_count: usize,
    pub failed_count: usize,
    pub total_cost: TrialCost,
    pub trials: Vec<TrialRecord>,
}

pub fn aggregate_trials(
    plan: &EvaluationPlan,
    records: Vec<TrialRecord>,
) -> Result<TrialAggregate, EvaluationError> {
    plan.validate()?;

    let schedule: BTreeMap<&str, &ScheduledTrial> = plan
        .scheduled_trials
        .iter()
        .map(|trial| (trial.scheduled_trial_id.as_str(), trial))
        .collect();
    let mut indexed = BTreeMap::new();
    for record in records {
        if !schedule.contains_key(record.scheduled_trial_id.as_str()) {
            return Err(EvaluationError::UnknownScheduledTrial(
                record.scheduled_trial_id,
            ));
        }
        let id = record.scheduled_trial_id.clone();
        if indexed.insert(id.clone(), record).is_some() {
            return Err(EvaluationError::DuplicateTrialRecord(id));
        }
    }

    let missing: Vec<String> = plan
        .scheduled_trials
        .iter()
        .filter(|scheduled| !indexed.contains_key(&scheduled.scheduled_trial_id))
        .map(|scheduled| scheduled.scheduled_trial_id.clone())
        .collect();
    if !missing.is_empty() {
        return Err(EvaluationError::MissingScheduledTrials(missing));
    }

    let mut ordered = Vec::with_capacity(plan.scheduled_trials.len());
    let mut total_cost = TrialCost::zero();
    let mut succeeded_count = 0;
    let mut failed_count = 0;
    for scheduled in &plan.scheduled_trials {
        let Some(record) = indexed.remove(&scheduled.scheduled_trial_id) else {
            return Err(EvaluationError::MissingScheduledTrials(vec![scheduled
                .scheduled_trial_id
                .clone()]));
        };
        let resources = match scheduled.arm {
            ArmId::Baseline => &plan.baseline.resources,
            ArmId::Intervention => &plan.intervention.resources,
        };
        if matches!(record.outcome, TrialOutcome::Succeeded { .. })
            && (record.cost.provider_cost_microunits > resources.max_provider_cost_microunits
                || record.cost.duration_ms > resources.max_duration_ms)
        {
            return Err(EvaluationError::ResourceCeilingExceeded(
                record.scheduled_trial_id,
            ));
        }
        total_cost.add_assign(&record.cost);
        match record.outcome {
            TrialOutcome::Succeeded { .. } => succeeded_count += 1,
            TrialOutcome::Failed { .. } => failed_count += 1,
        }
        ordered.push(record);
    }

    Ok(TrialAggregate {
        scheduled_count: plan.scheduled_trials.len(),
        succeeded_count,
        failed_count,
        total_cost,
        trials: ordered,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
    pub confidence_level: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectEstimate {
    pub estimand: Estimand,
    pub point_estimate: f64,
    pub matched_pair_count: usize,
    pub confidence_interval: Interval,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AttributionResult {
    Estimated(EffectEstimate),
    NotIdentified { reason: String },
}

pub fn estimate_marginal_effect(
    plan: &EvaluationPlan,
    aggregate: &TrialAggregate,
) -> Result<AttributionResult, EvaluationError> {
    plan.validate()?;
    if plan.analysis_design != AnalysisDesign::ControlledAblation {
        return Ok(AttributionResult::NotIdentified {
            reason: "naive arm averages do not identify marginal factor benefit".into(),
        });
    }

    let aggregate = aggregate_trials(plan, aggregate.trials.clone())?;
    let scheduled: BTreeMap<&str, &ScheduledTrial> = plan
        .scheduled_trials
        .iter()
        .map(|trial| (trial.scheduled_trial_id.as_str(), trial))
        .collect();
    let mut paired: BTreeMap<&str, [Option<f64>; 2]> = BTreeMap::new();
    for record in &aggregate.trials {
        let Some(scheduled_trial) = scheduled.get(record.scheduled_trial_id.as_str()) else {
            return Err(EvaluationError::UnknownScheduledTrial(
                record.scheduled_trial_id.clone(),
            ));
        };
        let TrialOutcome::Succeeded { quality } = record.outcome else {
            continue;
        };
        let slot = paired.entry(&scheduled_trial.task_id).or_default();
        match scheduled_trial.arm {
            ArmId::Baseline => slot[0] = Some(quality),
            ArmId::Intervention => slot[1] = Some(quality),
        }
    }

    let differences: Vec<f64> = paired
        .values()
        .filter_map(|pair| Some(pair[1]? - pair[0]?))
        .collect();
    if differences.len() < 2 {
        return Ok(AttributionResult::NotIdentified {
            reason: "at least two complete successful matched trial pairs are required".into(),
        });
    }

    let count = differences.len();
    let point_estimate = differences.iter().sum::<f64>() / count as f64;
    let half_width = if count > 1 {
        let variance = differences
            .iter()
            .map(|difference| (difference - point_estimate).powi(2))
            .sum::<f64>()
            / (count - 1) as f64;
        1.96 * (variance / count as f64).sqrt()
    } else {
        0.0
    };

    Ok(AttributionResult::Estimated(EffectEstimate {
        estimand: plan.estimand,
        point_estimate,
        matched_pair_count: count,
        confidence_interval: Interval {
            lower: point_estimate - half_width,
            upper: point_estimate + half_width,
            confidence_level: 0.95,
        },
    }))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityDecisionRule {
    pub minimum_acceptable_effect: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionOutcome {
    Recommended,
    Rejected,
    Inconclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseAuthority {
    NotGrantedByEvaluation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub outcome: PromotionOutcome,
    pub promotion: bool,
    pub release_authority: ReleaseAuthority,
    pub interval: Interval,
    pub rule: QualityDecisionRule,
}

pub fn evaluate_quality_interval(
    rule: QualityDecisionRule,
    interval: Interval,
) -> Result<QualityAssessment, EvaluationError> {
    if !interval.lower.is_finite()
        || !interval.upper.is_finite()
        || !interval.confidence_level.is_finite()
        || interval.lower > interval.upper
        || !(0.0..=1.0).contains(&interval.confidence_level)
        || interval.confidence_level == 0.0
        || !rule.minimum_acceptable_effect.is_finite()
    {
        return Err(EvaluationError::InvalidInterval);
    }

    let (outcome, promotion) = if interval.lower >= rule.minimum_acceptable_effect {
        (PromotionOutcome::Recommended, true)
    } else if interval.upper < rule.minimum_acceptable_effect {
        (PromotionOutcome::Rejected, false)
    } else {
        (PromotionOutcome::Inconclusive, false)
    };

    Ok(QualityAssessment {
        outcome,
        promotion,
        release_authority: ReleaseAuthority::NotGrantedByEvaluation,
        interval,
        rule,
    })
}
