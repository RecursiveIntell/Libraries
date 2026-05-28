//! Material-operation operator contracts and effect registry.
//!
//! Contracts here constrain AiDENs-local orchestration actions. They do not
//! authorize new provider, memory, proof, or kernel truth ownership.

use super::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorEffectV1 {
    ReadsTruth,
    ProjectsTruth,
    ProposesInference,
    EmitsReceipt,
    WidensView,
    RepairsState,
    SubtractsStructure,
    ChangesPromotion,
    ChangesSchema,
    CrossesTrustBoundary,
    AffectsFutureExecution,
    AffectsUserAgency,
    ReadsRepository,
    WritesRepository,
    RunsCommand,
    GeneratesPackage,
    ValidatesPackage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperatorContractV1 {
    pub operator_id: String,
    pub owner_plane: String,
    pub input_families: Vec<String>,
    pub output_families: Vec<String>,
    pub allowed_effects: BTreeSet<OperatorEffectV1>,
    pub forbidden_effects: BTreeSet<OperatorEffectV1>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub proof_obligations: Vec<String>,
    pub boundary_profile: String,
    pub replay_requirements: Vec<String>,
    pub failure_taxonomy: Vec<String>,
    pub human_approval: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl OperatorContractV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operator_id: impl Into<String>,
        owner_plane: impl Into<String>,
        input_families: Vec<&str>,
        output_families: Vec<&str>,
        allowed_effects: BTreeSet<OperatorEffectV1>,
        forbidden_effects: BTreeSet<OperatorEffectV1>,
        human_approval: impl Into<String>,
    ) -> Self {
        Self {
            operator_id: operator_id.into(),
            owner_plane: owner_plane.into(),
            input_families: input_families.into_iter().map(str::to_string).collect(),
            output_families: output_families.into_iter().map(str::to_string).collect(),
            allowed_effects,
            forbidden_effects,
            preconditions: vec![
                "input-manifest-present".into(),
                "execution-context-present".into(),
            ],
            postconditions: vec![
                "output-manifest-present".into(),
                "invocation-receipt-present".into(),
            ],
            proof_obligations: vec!["proof-or-debt-state-declared".into()],
            boundary_profile: "p28-v11a-strict-json-boundary-profile".into(),
            replay_requirements: vec![
                "stable-input-digests".into(),
                "receipt-bearing-transition".into(),
            ],
            failure_taxonomy: vec![
                "invalid-input".into(),
                "policy-denied".into(),
                "tool-failed".into(),
                "proof-debt".into(),
                "degraded-output".into(),
            ],
            human_approval: human_approval.into(),
            reason_codes: vec!["operator-contract-declared".into()],
        }
    }

    pub fn permits_effects(&self, effects: &BTreeSet<OperatorEffectV1>) -> bool {
        effects.is_subset(&self.allowed_effects) && self.forbidden_effects.is_disjoint(effects)
    }

    pub fn failure_taxonomy_is_finite(&self) -> bool {
        self.failure_taxonomy
            .iter()
            .all(|class| MATERIAL_OPERATOR_FAILURE_CLASSES.contains(&class.as_str()))
    }
}

pub const MATERIAL_OPERATOR_FAILURE_CLASSES: &[&str] = &[
    "invalid-input",
    "policy-denied",
    "tool-failed",
    "proof-debt",
    "degraded-output",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MaterialOperationRegistryV1 {
    pub registry_id: ArtifactId,
    pub contracts: BTreeMap<String, OperatorContractV1>,
    pub generated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl MaterialOperationRegistryV1 {
    pub fn new(contracts: Vec<OperatorContractV1>) -> Self {
        let mut map = BTreeMap::new();
        for contract in contracts {
            map.insert(contract.operator_id.clone(), contract);
        }
        let material = map.keys().cloned().collect::<Vec<_>>().join("|");
        Self {
            registry_id: generated_artifact_id_from_material(
                "material-operation-registry",
                &material,
            ),
            contracts: map,
            generated_at: Utc::now(),
            reason_codes: vec!["material-operation-registry-declared".into()],
        }
    }

    pub fn contract(&self, operator_id: &str) -> Option<&OperatorContractV1> {
        self.contracts.get(operator_id)
    }

    pub fn authorize_effects(
        &self,
        operator_id: &str,
        effects: BTreeSet<OperatorEffectV1>,
    ) -> Result<(), String> {
        let contract = self
            .contract(operator_id)
            .ok_or_else(|| format!("operator contract missing: {operator_id}"))?;
        if contract.permits_effects(&effects) {
            Ok(())
        } else {
            Err(format!(
                "operator effect not declared or forbidden: {operator_id}"
            ))
        }
    }

    pub fn authorize_material_invocation(
        &self,
        operator_id: &str,
        effects: BTreeSet<OperatorEffectV1>,
        execution_context: &ExecutionContextEnvelopeV1,
        input_manifest: &ArtifactManifestV1,
        output_manifest: &ArtifactManifestV1,
        receipt_refs: &[ArtifactId],
    ) -> Result<(), String> {
        let contract = self
            .contract(operator_id)
            .ok_or_else(|| format!("operator contract missing: {operator_id}"))?;
        if !contract.failure_taxonomy_is_finite() {
            return Err(format!(
                "operator failure taxonomy is not release-gated: {operator_id}"
            ));
        }
        if !execution_context.terminal_budget_is_enforced() {
            return Err(format!(
                "operator terminal state missing budget enforcement: {operator_id}"
            ));
        }
        self.authorize_effects(operator_id, effects)?;
        if !input_manifest.complete() || !output_manifest.complete() {
            return Err(format!(
                "operator manifests incomplete or opaque refs present: {operator_id}"
            ));
        }
        if receipt_refs.is_empty() {
            return Err(format!(
                "operator invocation missing material receipts: {operator_id}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperationConformanceReportV1 {
    pub report_id: ArtifactId,
    pub registry_id: ArtifactId,
    pub checked_operator_ids: Vec<String>,
    pub missing_operator_ids: Vec<String>,
    pub effect_violations: Vec<String>,
    pub passed: bool,
    pub generated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl OperationConformanceReportV1 {
    pub fn for_required_operators(
        registry: &MaterialOperationRegistryV1,
        required_operator_ids: &[&str],
    ) -> Self {
        let missing_operator_ids = required_operator_ids
            .iter()
            .filter(|operator_id| !registry.contracts.contains_key(**operator_id))
            .map(|operator_id| (*operator_id).to_string())
            .collect::<Vec<_>>();
        let effect_violations = registry
            .contracts
            .values()
            .filter(|contract| {
                !contract
                    .forbidden_effects
                    .is_disjoint(&contract.allowed_effects)
                    || !contract.failure_taxonomy_is_finite()
            })
            .map(|contract| contract.operator_id.clone())
            .collect::<Vec<_>>();
        let passed = missing_operator_ids.is_empty() && effect_violations.is_empty();
        Self {
            report_id: generated_artifact_id_from_material(
                "operation-conformance-report",
                &format!(
                    "{}|{}",
                    registry.registry_id.0,
                    required_operator_ids.join("|")
                ),
            ),
            registry_id: registry.registry_id.clone(),
            checked_operator_ids: required_operator_ids
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            missing_operator_ids,
            effect_violations,
            passed,
            generated_at: Utc::now(),
            reason_codes: if passed {
                vec!["operation-contracts-conformant".into()]
            } else {
                vec!["operation-contracts-missing-or-invalid".into()]
            },
        }
    }
}

pub fn p28_required_operator_ids() -> Vec<&'static str> {
    vec![
        "aidens.agent.validate",
        "aidens.agent.doctor",
        "aidens.runner.turn",
        "aidens.provider.route",
        "aidens.tool.repo_read",
        "aidens.tool.repo_list",
        "aidens.tool.file_stat",
        "aidens.tool.repo_search",
        "aidens.tool.patch_propose",
        "aidens.tool.patch_apply",
        "aidens.tool.run_checks",
        "aidens.package.generate",
        "aidens.package.validate",
        "aidens.package.self_replay",
        "aidens.report.final_done",
    ]
}

pub fn p28_declared_material_operation_registry() -> MaterialOperationRegistryV1 {
    use OperatorEffectV1::*;
    let read = BTreeSet::from([ReadsRepository, EmitsReceipt]);
    let validate = BTreeSet::from([EmitsReceipt, AffectsFutureExecution]);
    let run = BTreeSet::from([EmitsReceipt, AffectsFutureExecution, AffectsUserAgency]);
    let write = BTreeSet::from([WritesRepository, EmitsReceipt, AffectsFutureExecution]);
    let command = BTreeSet::from([RunsCommand, EmitsReceipt, AffectsFutureExecution]);
    let package = BTreeSet::from([GeneratesPackage, EmitsReceipt]);
    let package_validate = BTreeSet::from([ValidatesPackage, EmitsReceipt]);
    let forbidden_cloud = BTreeSet::from([CrossesTrustBoundary, ChangesSchema, SubtractsStructure]);
    MaterialOperationRegistryV1::new(vec![
        OperatorContractV1::new(
            "aidens.agent.validate",
            "runtime",
            vec!["agent-spec"],
            vec!["agent-validation-report"],
            validate.clone(),
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.agent.doctor",
            "runtime",
            vec!["agent-spec"],
            vec!["doctor-report"],
            validate.clone(),
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.runner.turn",
            "runtime",
            vec!["agent-spec", "task"],
            vec!["run-report", "run-bundle"],
            run,
            forbidden_cloud.clone(),
            "operator-supervised",
        ),
        OperatorContractV1::new(
            "aidens.provider.route",
            "tool-execution",
            vec!["provider-policy"],
            vec!["provider-route-report"],
            BTreeSet::from([EmitsReceipt]),
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.repo_read",
            "tool-execution",
            vec!["repo-path"],
            vec!["repo-read-receipt"],
            read.clone(),
            BTreeSet::from([WritesRepository, RunsCommand, CrossesTrustBoundary]),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.repo_list",
            "tool-execution",
            vec!["repo-path"],
            vec!["repo-list-receipt"],
            read.clone(),
            BTreeSet::from([WritesRepository, RunsCommand, CrossesTrustBoundary]),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.file_stat",
            "tool-execution",
            vec!["repo-path"],
            vec!["file-stat-receipt"],
            read.clone(),
            BTreeSet::from([WritesRepository, RunsCommand, CrossesTrustBoundary]),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.repo_search",
            "tool-execution",
            vec!["repo-query"],
            vec!["repo-search-receipt"],
            read,
            BTreeSet::from([WritesRepository, RunsCommand, CrossesTrustBoundary]),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.patch_propose",
            "tool-execution",
            vec!["patch-diff"],
            vec!["patch-proposal"],
            BTreeSet::from([ProposesInference, EmitsReceipt]),
            BTreeSet::from([WritesRepository, RunsCommand, CrossesTrustBoundary]),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.tool.patch_apply",
            "tool-execution",
            vec!["patch-diff", "permit"],
            vec!["patch-apply-receipt"],
            write,
            BTreeSet::from([RunsCommand, CrossesTrustBoundary]),
            "required",
        ),
        OperatorContractV1::new(
            "aidens.tool.run_checks",
            "tool-execution",
            vec!["command", "permit"],
            vec!["command-run-receipt"],
            command,
            BTreeSet::from([WritesRepository, CrossesTrustBoundary]),
            "required",
        ),
        OperatorContractV1::new(
            "aidens.package.generate",
            "package",
            vec!["repo-tree"],
            vec!["package-manifest"],
            package,
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.package.validate",
            "package",
            vec!["package-manifest"],
            vec!["package-validation-report"],
            package_validate.clone(),
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.package.self_replay",
            "package",
            vec!["package"],
            vec!["self-replay-receipt"],
            package_validate,
            forbidden_cloud.clone(),
            "not-required",
        ),
        OperatorContractV1::new(
            "aidens.report.final_done",
            "governance",
            vec!["phase-reports", "receipts"],
            vec!["final-report"],
            BTreeSet::from([EmitsReceipt, ChangesPromotion]),
            BTreeSet::from([CrossesTrustBoundary, WritesRepository, RunsCommand]),
            "operator-reviewed",
        ),
    ])
}
