-- Draft additive v14 persistence scaffold.

CREATE TABLE IF NOT EXISTS intervention_bundles (
    intervention_id           TEXT PRIMARY KEY,
    episode_id                TEXT,
    outcome_schema_id         TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS experiment_cases (
    experiment_case_id        TEXT PRIMARY KEY,
    intervention_id           TEXT NOT NULL,
    cohort_contract_id        TEXT NOT NULL,
    comparability_matrix_id   TEXT,
    refuter_suite_id          TEXT,
    decision_trace_id         TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS decision_traces (
    decision_trace_id         TEXT PRIMARY KEY,
    experiment_case_id        TEXT,
    counterfactual_slice_id   TEXT,
    rollout_decision_id       TEXT,
    rollback_decision_id      TEXT,
    artifact_json             TEXT NOT NULL,
    recorded_at               TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_v14_exp_intervention ON experiment_cases(intervention_id);
CREATE INDEX IF NOT EXISTS idx_v14_trace_exp ON decision_traces(experiment_case_id);
