//! Mechanism display and equivalence reports.
//!
//! AiDENs exposes report envelopes only; mechanism/kernel semantics remain delegated to sibling canonical crates.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MechanismKindV1 {
    Symbolic,
    Sparse,
    Neural,
    Hybrid,
    SimulatorBacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TheoryVersionStateV1 {
    Candidate,
    Fitted,
    Refuted,
    Superseded,
    Published,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FitRunOutcomeV1 {
    Succeeded,
    Failed,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum InvarianceVerdictV1 {
    Invariant,
    Variant,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TheoryRefuterOutcomeV1 {
    NotRun,
    NoRefutationFound,
    Refuted,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EquivalenceDispositionV1 {
    Distinct,
    AliasAdmitted,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MechanismReportDraftV1 {
    pub mechanism_kind: MechanismKindV1,
    pub name: String,
    pub version_label: String,
    pub variables: Vec<String>,
    pub assumptions: Vec<String>,
    pub causal_claims: Vec<String>,
    pub mechanism_program: Vec<String>,
    pub parameters: BTreeMap<String, serde_json::Value>,
    pub replay_recipe: String,
}

impl MechanismReportDraftV1 {
    pub fn new(
        mechanism_kind: MechanismKindV1,
        name: impl Into<String>,
        version_label: impl Into<String>,
        replay_recipe: impl Into<String>,
    ) -> Self {
        Self {
            mechanism_kind,
            name: name.into(),
            version_label: version_label.into(),
            variables: Vec::new(),
            assumptions: Vec::new(),
            causal_claims: Vec::new(),
            mechanism_program: Vec::new(),
            parameters: BTreeMap::new(),
            replay_recipe: replay_recipe.into(),
        }
    }

    pub fn with_variables(mut self, variables: Vec<String>) -> Self {
        self.variables = variables;
        self
    }

    pub fn with_assumptions(mut self, assumptions: Vec<String>) -> Self {
        self.assumptions = assumptions;
        self
    }

    pub fn with_causal_claims(mut self, causal_claims: Vec<String>) -> Self {
        self.causal_claims = causal_claims;
        self
    }

    pub fn with_program(mut self, mechanism_program: Vec<String>) -> Self {
        self.mechanism_program = mechanism_program;
        self
    }

    pub fn with_parameters(mut self, parameters: BTreeMap<String, serde_json::Value>) -> Self {
        self.parameters = parameters;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MechanismReportV1 {
    pub mechanism_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub mechanism_kind: MechanismKindV1,
    pub name: String,
    pub version_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hypothesis_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanism_program: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, serde_json::Value>,
    pub structural_signature_digest: DisplayDigestV1,
    pub observational_signature_digest: DisplayDigestV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulator_contract_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_artifact_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_weight_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notebook_reference: Option<String>,
    pub replay_recipe: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl MechanismReportV1 {
    pub fn new(draft: MechanismReportDraftV1) -> Self {
        let mechanism_kind = draft.mechanism_kind;
        let name = draft.name;
        let version_label = draft.version_label;
        let variables = sorted_unique_strings(draft.variables);
        let assumptions = sorted_unique_strings(draft.assumptions);
        let causal_claims = sorted_unique_strings(draft.causal_claims);
        let mechanism_program = draft.mechanism_program;
        let parameters = draft.parameters;
        let replay_recipe = draft.replay_recipe;
        let structural_payload = serde_json::json!({
            "mechanism_kind": mechanism_kind,
            "name": &name,
            "version_label": &version_label,
            "variables": &variables,
            "assumptions": &assumptions,
            "causal_claims": &causal_claims,
            "mechanism_program": &mechanism_program,
            "parameters": &parameters,
        });
        let observational_payload = serde_json::json!({
            "name": &name,
            "variables": &variables,
            "mechanism_program": &mechanism_program,
        });
        Self {
            mechanism_id: local_artifact_id_from_stack_digest(
                "mechanism-report",
                &display_json_string(&structural_payload),
            ),
            kind: ArtifactKindV1::MechanismReport,
            mechanism_kind,
            name,
            version_label,
            hypothesis_id: None,
            variables,
            assumptions,
            causal_claims,
            mechanism_program,
            parameters,
            structural_signature_digest: DisplayDigestV1::for_json_value(&structural_payload),
            observational_signature_digest: DisplayDigestV1::for_json_value(&observational_payload),
            simulator_contract_id: None,
            source_artifact_ids: Vec::new(),
            evidence_ids: Vec::new(),
            raw_weight_digest: None,
            notebook_reference: None,
            replay_recipe,
            reason_codes: vec![
                "mechanism-is-versioned-artifact".into(),
                "raw-model-weights-alone-are-insufficient".into(),
            ],
            created_at: Utc::now(),
        }
    }

    pub fn with_hypothesis(mut self, hypothesis_id: ArtifactId) -> Self {
        self.hypothesis_id = Some(hypothesis_id);
        self
    }

    pub fn with_simulator_contract(mut self, simulator_contract_id: ArtifactId) -> Self {
        self.simulator_contract_id = Some(simulator_contract_id);
        self.reason_codes
            .push("simulator-contract-linked-for-replay".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_evidence(mut self, evidence_ids: Vec<ArtifactId>) -> Self {
        self.evidence_ids = sorted_unique_artifact_ids(evidence_ids);
        self
    }

    pub fn with_source_artifacts(mut self, source_artifact_ids: Vec<ArtifactId>) -> Self {
        self.source_artifact_ids = sorted_unique_artifact_ids(source_artifact_ids);
        self
    }

    pub fn with_raw_weight_digest(mut self, raw_weight_digest: impl Into<String>) -> Self {
        self.raw_weight_digest = Some(raw_weight_digest.into());
        self.reason_codes
            .push("raw-weights-linked-but-not-sole-artifact".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn is_replayable(&self) -> bool {
        !self.replay_recipe.trim().is_empty()
            && (!self.mechanism_program.is_empty() || self.simulator_contract_id.is_some())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimulatorContractV1 {
    pub contract_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub name: String,
    pub backend: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub deterministic: bool,
    pub seed_policy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_command: Vec<String>,
    pub simulator_digest: DisplayDigestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl SimulatorContractV1 {
    pub fn new(
        name: impl Into<String>,
        backend: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        replay_command: Vec<String>,
    ) -> Self {
        let name = name.into();
        let backend = backend.into();
        let digest_payload = serde_json::json!({
            "name": &name,
            "backend": &backend,
            "input_schema": &input_schema,
            "output_schema": &output_schema,
            "replay_command": &replay_command,
        });
        Self {
            contract_id: local_artifact_id_from_stack_digest(
                "simulator-contract",
                &display_json_string(&digest_payload),
            ),
            kind: ArtifactKindV1::SimulatorContract,
            name,
            backend,
            input_schema,
            output_schema,
            deterministic: true,
            seed_policy: "fixed-seed-required-for-replay".into(),
            replay_command,
            simulator_digest: DisplayDigestV1::for_json_value(&digest_payload),
            reason_codes: vec!["simulator-contract-defines-replay-boundary".into()],
            created_at: Utc::now(),
        }
    }

    pub fn nondeterministic(mut self, seed_policy: impl Into<String>) -> Self {
        self.deterministic = false;
        self.seed_policy = seed_policy.into();
        self.reason_codes
            .push("nondeterminism-disclosed-by-contract".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn is_replayable(&self) -> bool {
        !self.replay_command.is_empty() && (self.deterministic || self.seed_policy.contains("seed"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FitRunReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub mechanism_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theory_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulator_contract_id: Option<ArtifactId>,
    pub dataset_digest: DisplayDigestV1,
    pub parameter_digest: DisplayDigestV1,
    pub output_digest: DisplayDigestV1,
    pub outcome: FitRunOutcomeV1,
    pub objective: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub score_is_verification: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_command: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub fitted_at: DateTime<Utc>,
}

impl FitRunReportV1 {
    pub fn new(
        mechanism_id: ArtifactId,
        simulator_contract_id: Option<ArtifactId>,
        dataset: serde_json::Value,
        parameters: serde_json::Value,
        output: serde_json::Value,
        objective: impl Into<String>,
        score: Option<f64>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("fit-run-report"),
            kind: ArtifactKindV1::FitRunReport,
            mechanism_id,
            theory_id: None,
            simulator_contract_id,
            dataset_digest: DisplayDigestV1::for_json_value(&dataset),
            parameter_digest: DisplayDigestV1::for_json_value(&parameters),
            output_digest: DisplayDigestV1::for_json_value(&output),
            outcome: FitRunOutcomeV1::Succeeded,
            objective: objective.into(),
            score,
            score_is_verification: false,
            seed: None,
            replay_command: Vec::new(),
            run_id: None,
            attempt_id: None,
            source_receipt_ids: Vec::new(),
            reason_codes: vec![
                "fit-run-recorded-as-evidence".into(),
                "score-is-not-verification".into(),
            ],
            fitted_at: Utc::now(),
        }
    }

    pub fn for_theory(mut self, theory_id: ArtifactId) -> Self {
        self.theory_id = Some(theory_id);
        self
    }

    pub fn with_execution_context(mut self, context: &AidensRunContextV1) -> Self {
        self.run_id = Some(context.run_id.clone());
        self.attempt_id = Some(context.attempt_id.clone());
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_replay_command(mut self, replay_command: Vec<String>) -> Self {
        self.replay_command = replay_command;
        self
    }

    pub fn with_source_receipts(mut self, source_receipt_ids: Vec<ArtifactId>) -> Self {
        self.source_receipt_ids = sorted_unique_artifact_ids(source_receipt_ids);
        self
    }

    pub fn failed(mut self, reason: impl Into<String>) -> Self {
        self.outcome = FitRunOutcomeV1::Failed;
        self.reason_codes.push(reason.into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn is_replayable(&self) -> bool {
        self.outcome == FitRunOutcomeV1::Succeeded
            && !self.replay_command.is_empty()
            && self.simulator_contract_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InvarianceCheckV1 {
    pub check_label: String,
    pub environment: String,
    pub perturbation: String,
    pub metric_delta: f64,
    pub verdict: InvarianceVerdictV1,
}

impl InvarianceCheckV1 {
    pub fn new(
        check_label: impl Into<String>,
        environment: impl Into<String>,
        perturbation: impl Into<String>,
        metric_delta: f64,
        tolerance: f64,
    ) -> Self {
        let verdict = if metric_delta.abs() <= tolerance {
            InvarianceVerdictV1::Invariant
        } else {
            InvarianceVerdictV1::Variant
        };
        Self {
            check_label: check_label.into(),
            environment: environment.into(),
            perturbation: perturbation.into(),
            metric_delta,
            verdict,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InvarianceReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub mechanism_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theory_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit_run_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perturbations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<InvarianceCheckV1>,
    pub verdict: InvarianceVerdictV1,
    pub observational_equivalence_only: bool,
    pub causal_identification_claimed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl InvarianceReportV1 {
    pub fn new(
        mechanism_id: ArtifactId,
        theory_id: Option<ArtifactId>,
        fit_run_receipt_id: Option<ArtifactId>,
        checks: Vec<InvarianceCheckV1>,
    ) -> Self {
        let environments = sorted_unique_strings(
            checks
                .iter()
                .map(|check| check.environment.clone())
                .collect(),
        );
        let perturbations = sorted_unique_strings(
            checks
                .iter()
                .map(|check| check.perturbation.clone())
                .collect(),
        );
        let verdict = if checks.is_empty() {
            InvarianceVerdictV1::Inconclusive
        } else if checks
            .iter()
            .all(|check| check.verdict == InvarianceVerdictV1::Invariant)
        {
            InvarianceVerdictV1::Invariant
        } else if checks
            .iter()
            .any(|check| check.verdict == InvarianceVerdictV1::Variant)
        {
            InvarianceVerdictV1::Variant
        } else {
            InvarianceVerdictV1::Inconclusive
        };
        Self {
            report_id: display_only_unstable_id("invariance-report"),
            kind: ArtifactKindV1::InvarianceReport,
            mechanism_id,
            theory_id,
            fit_run_receipt_id,
            environments,
            perturbations,
            checks,
            verdict,
            observational_equivalence_only: true,
            causal_identification_claimed: false,
            reason_codes: vec![
                "invariance-tested-as-refuter-evidence".into(),
                "observational-equivalence-is-not-causal-identification".into(),
            ],
            recorded_at: Utc::now(),
        }
    }

    pub fn with_causal_identification(mut self, reason: impl Into<String>) -> Self {
        self.observational_equivalence_only = false;
        self.causal_identification_claimed = self.verdict == InvarianceVerdictV1::Invariant;
        self.reason_codes.push(reason.into());
        self.reason_codes
            .push("causal-identification-claim-explicit".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn supports_causal_identification(&self) -> bool {
        self.causal_identification_claimed
            && !self.observational_equivalence_only
            && self.verdict == InvarianceVerdictV1::Invariant
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MechanismEquivalenceDecisionV1 {
    pub decision_id: ArtifactId,
    pub left_mechanism_id: ArtifactId,
    pub right_mechanism_id: ArtifactId,
    pub disposition: EquivalenceDispositionV1,
    pub observationally_equivalent: bool,
    pub causally_identified_same: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_by_artifact_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl MechanismEquivalenceDecisionV1 {
    pub fn distinct(
        left_mechanism_id: ArtifactId,
        right_mechanism_id: ArtifactId,
        observationally_equivalent: bool,
    ) -> Self {
        Self {
            decision_id: display_only_unstable_id("mechanism-equivalence-decision"),
            left_mechanism_id,
            right_mechanism_id,
            disposition: EquivalenceDispositionV1::Distinct,
            observationally_equivalent,
            causally_identified_same: false,
            admitted_by_artifact_id: None,
            reason_codes: vec![
                "equivalent-observations-do-not-collapse-mechanisms".into(),
                "causal-identification-not-established".into(),
            ],
        }
    }

    pub fn alias_admitted(
        left_mechanism_id: ArtifactId,
        right_mechanism_id: ArtifactId,
        admitted_by_artifact_id: ArtifactId,
    ) -> Self {
        Self {
            decision_id: display_only_unstable_id("mechanism-equivalence-decision"),
            left_mechanism_id,
            right_mechanism_id,
            disposition: EquivalenceDispositionV1::AliasAdmitted,
            observationally_equivalent: true,
            causally_identified_same: true,
            admitted_by_artifact_id: Some(admitted_by_artifact_id),
            reason_codes: vec!["explicit-equivalence-alias-admitted".into()],
        }
    }

    pub fn permits_alias(&self, left: &ArtifactId, right: &ArtifactId) -> bool {
        self.disposition == EquivalenceDispositionV1::AliasAdmitted
            && self.admitted_by_artifact_id.is_some()
            && self.causally_identified_same
            && ((&self.left_mechanism_id == left && &self.right_mechanism_id == right)
                || (&self.left_mechanism_id == right && &self.right_mechanism_id == left))
    }
}
