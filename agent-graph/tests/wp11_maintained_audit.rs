use agent_graph::maintained_audit::{
    aggregate_trials, estimate_marginal_effect, evaluate_quality_interval, AdmittedInputs,
    AnalysisDesign, ArmId, AttributionResult, DistinguishedFactor, Estimand, EvaluationArm,
    EvaluationError, EvaluationPlan, Interval, PromotionOutcome, QualityDecisionRule,
    ReleaseAuthority, ResourceControls, ScheduledTrial, TrialCost, TrialFailure, TrialOutcome,
    TrialRecord,
};
use std::collections::BTreeMap;

fn resources() -> ResourceControls {
    ResourceControls {
        max_provider_cost_microunits: 50_000,
        max_duration_ms: 30_000,
        max_specialists: 4,
    }
}

fn factors(specialist: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("model".into(), "fixed-model-revision".into()),
        ("specialist".into(), specialist.into()),
        ("temperature".into(), "0".into()),
    ])
}

fn arm(id: ArmId, specialist: &str) -> EvaluationArm {
    EvaluationArm {
        id,
        admitted: AdmittedInputs {
            task_digest: "sha256:task".into(),
            source_digest: "sha256:source".into(),
            requirements_digest: "sha256:requirements".into(),
        },
        resources: resources(),
        factors: factors(specialist),
    }
}

fn schedule() -> Vec<ScheduledTrial> {
    vec![
        ScheduledTrial::new("task-a-baseline", "task-a", ArmId::Baseline),
        ScheduledTrial::new("task-a-intervention", "task-a", ArmId::Intervention),
        ScheduledTrial::new("task-b-baseline", "task-b", ArmId::Baseline),
        ScheduledTrial::new("task-b-intervention", "task-b", ArmId::Intervention),
    ]
}

fn plan(design: AnalysisDesign) -> EvaluationPlan {
    EvaluationPlan {
        baseline: arm(ArmId::Baseline, "absent"),
        intervention: arm(ArmId::Intervention, "present"),
        distinguished_factor: DistinguishedFactor {
            name: "specialist".into(),
            baseline_value: "absent".into(),
            intervention_value: "present".into(),
        },
        estimand: Estimand::MeanPairedQualityDifference,
        analysis_design: design,
        scheduled_trials: schedule(),
    }
}

fn success(id: &str, quality: f64, cost: u64) -> TrialRecord {
    TrialRecord {
        scheduled_trial_id: id.into(),
        outcome: TrialOutcome::Succeeded { quality },
        cost: TrialCost {
            provider_cost_microunits: cost,
            duration_ms: cost * 10,
        },
    }
}

#[test]
fn eval_01_requires_matched_admitted_arms_controls_and_one_distinguished_factor() {
    let valid = plan(AnalysisDesign::ControlledAblation);
    assert_eq!(valid.validate(), Ok(()));

    let mut mismatched_task = valid.clone();
    mismatched_task.intervention.admitted.task_digest = "sha256:other-task".into();
    assert_eq!(
        mismatched_task.validate(),
        Err(EvaluationError::TaskMismatch)
    );

    let mut mismatched_source = valid.clone();
    mismatched_source.intervention.admitted.source_digest = "sha256:other-source".into();
    assert_eq!(
        mismatched_source.validate(),
        Err(EvaluationError::SourceMismatch)
    );

    let mut mismatched_requirements = valid.clone();
    mismatched_requirements
        .intervention
        .admitted
        .requirements_digest = "sha256:other-requirements".into();
    assert_eq!(
        mismatched_requirements.validate(),
        Err(EvaluationError::RequirementsMismatch)
    );

    let mut mismatched_resources = valid.clone();
    mismatched_resources.intervention.resources.max_specialists += 1;
    assert_eq!(
        mismatched_resources.validate(),
        Err(EvaluationError::ResourceControlsMismatch)
    );

    let mut absent_controls = valid.clone();
    absent_controls.baseline.resources.max_duration_ms = 0;
    assert_eq!(
        absent_controls.validate(),
        Err(EvaluationError::InvalidResourceControls)
    );

    let mut no_distinguished_difference = valid.clone();
    no_distinguished_difference
        .intervention
        .factors
        .insert("specialist".into(), "absent".into());
    assert_eq!(
        no_distinguished_difference.validate(),
        Err(EvaluationError::DistinguishedFactorMismatch)
    );

    let mut second_difference = valid;
    second_difference
        .intervention
        .factors
        .insert("temperature".into(), "1".into());
    assert_eq!(
        second_difference.validate(),
        Err(EvaluationError::UnmatchedFactors)
    );
}

#[test]
fn eval_03_retains_every_scheduled_failure_and_its_cost_in_aggregates() {
    let plan = plan(AnalysisDesign::ControlledAblation);
    let records = vec![
        success("task-b-intervention", 0.91, 800),
        TrialRecord {
            scheduled_trial_id: "task-a-intervention".into(),
            outcome: TrialOutcome::Failed {
                failure: TrialFailure::Timeout,
            },
            cost: TrialCost {
                provider_cost_microunits: 700,
                duration_ms: 30_000,
            },
        },
        success("task-a-baseline", 0.80, 500),
        TrialRecord {
            scheduled_trial_id: "task-b-baseline".into(),
            outcome: TrialOutcome::Failed {
                failure: TrialFailure::BudgetExhausted,
            },
            cost: TrialCost {
                provider_cost_microunits: 600,
                duration_ms: 12_000,
            },
        },
    ];

    let aggregate = aggregate_trials(&plan, records).unwrap();
    assert_eq!(aggregate.scheduled_count, 4);
    assert_eq!(aggregate.trials.len(), 4);
    assert_eq!(aggregate.succeeded_count, 2);
    assert_eq!(aggregate.failed_count, 2);
    assert_eq!(aggregate.total_cost.provider_cost_microunits, 2_600);
    assert_eq!(aggregate.total_cost.duration_ms, 55_000);
    assert_eq!(
        aggregate
            .trials
            .iter()
            .map(|trial| trial.scheduled_trial_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "task-a-baseline",
            "task-a-intervention",
            "task-b-baseline",
            "task-b-intervention"
        ]
    );
    assert!(matches!(
        aggregate.trials[1].outcome,
        TrialOutcome::Failed {
            failure: TrialFailure::Timeout
        }
    ));
    assert!(matches!(
        aggregate.trials[2].outcome,
        TrialOutcome::Failed {
            failure: TrialFailure::BudgetExhausted
        }
    ));

    let missing_failure = vec![
        success("task-a-baseline", 0.80, 500),
        success("task-b-intervention", 0.91, 800),
    ];
    assert_eq!(
        aggregate_trials(&plan, missing_failure),
        Err(EvaluationError::MissingScheduledTrials(vec![
            "task-a-intervention".into(),
            "task-b-baseline".into(),
        ]))
    );
}

#[test]
fn eval_04_only_controlled_matched_ablation_identifies_specialist_effect() {
    let controlled = plan(AnalysisDesign::ControlledAblation);
    let records = vec![
        success("task-a-baseline", 0.60, 100),
        success("task-a-intervention", 0.80, 100),
        success("task-b-baseline", 0.70, 100),
        success("task-b-intervention", 0.80, 100),
    ];
    let aggregate = aggregate_trials(&controlled, records.clone()).unwrap();
    let estimate = estimate_marginal_effect(&controlled, &aggregate).unwrap();
    let AttributionResult::Estimated(estimate) = estimate else {
        panic!("controlled matched ablation must identify the estimand");
    };
    assert_eq!(estimate.estimand, Estimand::MeanPairedQualityDifference);
    assert!((estimate.point_estimate - 0.15).abs() < 1e-12);
    assert_eq!(estimate.matched_pair_count, 2);
    assert_eq!(estimate.confidence_interval.confidence_level, 0.95);

    let naive = plan(AnalysisDesign::NaiveArmAverages);
    let naive_aggregate = aggregate_trials(&naive, records).unwrap();
    assert_eq!(
        estimate_marginal_effect(&naive, &naive_aggregate),
        Ok(AttributionResult::NotIdentified {
            reason: "naive arm averages do not identify marginal factor benefit".into(),
        })
    );

    let mut unmatched = controlled;
    unmatched
        .intervention
        .factors
        .insert("model".into(), "different-model".into());
    assert_eq!(
        estimate_marginal_effect(&unmatched, &aggregate),
        Err(EvaluationError::UnmatchedFactors)
    );
}

#[test]
fn eval_05_crossing_quality_interval_is_inconclusive_and_nonauthorizing() {
    let assessment = evaluate_quality_interval(
        QualityDecisionRule {
            minimum_acceptable_effect: -0.02,
        },
        Interval {
            lower: -0.05,
            upper: 0.03,
            confidence_level: 0.95,
        },
    )
    .unwrap();

    assert_eq!(assessment.outcome, PromotionOutcome::Inconclusive);
    assert!(!assessment.promotion);
    assert_eq!(
        assessment.release_authority,
        ReleaseAuthority::NotGrantedByEvaluation
    );
}

#[test]
fn pr11_single_pair_has_no_identified_interval() {
    let mut plan = plan(AnalysisDesign::ControlledAblation);
    plan.scheduled_trials.truncate(2);
    let aggregate = aggregate_trials(
        &plan,
        vec![
            success("task-a-baseline", 0.1, 1),
            success("task-a-intervention", 0.9, 1),
        ],
    )
    .unwrap();
    assert!(matches!(
        estimate_marginal_effect(&plan, &aggregate).unwrap(),
        AttributionResult::NotIdentified { .. }
    ));
}

#[test]
fn pr11_success_over_either_resource_ceiling_is_rejected() {
    let mut plan = plan(AnalysisDesign::ControlledAblation);
    plan.scheduled_trials.truncate(2);
    for cost in [
        TrialCost {
            provider_cost_microunits: 50_001,
            duration_ms: 1,
        },
        TrialCost {
            provider_cost_microunits: 1,
            duration_ms: 30_001,
        },
    ] {
        let mut bad = success("task-a-intervention", 0.9, 1);
        bad.cost = cost;
        assert!(matches!(
            aggregate_trials(&plan, vec![success("task-a-baseline", 0.1, 1), bad]),
            Err(EvaluationError::ResourceCeilingExceeded(_))
        ));
    }
    let mut exact = success("task-a-intervention", 0.9, 1);
    exact.cost = TrialCost {
        provider_cost_microunits: 50_000,
        duration_ms: 30_000,
    };
    assert!(aggregate_trials(&plan, vec![success("task-a-baseline", 0.1, 1), exact]).is_ok());
}
