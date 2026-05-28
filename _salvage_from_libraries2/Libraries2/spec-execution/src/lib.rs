//! Typed spec and proof surface crate for generated artifact families.
//!
//! The crate keeps the historical compatibility name, but it does not claim
//! a full execution runtime. It publishes spec/proof schemas and bounded
//! evaluation artifacts for later admission elsewhere.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ContentDigest, GeneratedConformanceCorpusId, GeneratedInterpreterBundleId,
    GeneratedMigrationPlanId, GeneratedSchemaBundleId, HumanVetoBundleId, MetaChallengeBundleId,
    NormativeAstId, ProofEvaluationReceiptId, ProofObligationSetId, SelfHostingBuildReceiptId,
    SpecBundleId,
};

pub use stack_ids::SurfaceStatus;

pub const SPEC_BUNDLE_V1_SCHEMA: &str = "spec_bundle_v1";
pub const NORMATIVE_AST_V1_SCHEMA: &str = "normative_ast_v1";
pub const GENERATED_SCHEMA_BUNDLE_V1_SCHEMA: &str = "generated_schema_bundle_v1";
pub const GENERATED_INTERPRETER_BUNDLE_V1_SCHEMA: &str = "generated_interpreter_bundle_v1";
pub const GENERATED_CONFORMANCE_CORPUS_V1_SCHEMA: &str = "generated_conformance_corpus_v1";
pub const GENERATED_MIGRATION_PLAN_V1_SCHEMA: &str = "generated_migration_plan_v1";
pub const PROOF_OBLIGATION_SET_V1_SCHEMA: &str = "proof_obligation_set_v1";
pub const PROOF_EVALUATION_RECEIPT_V1_SCHEMA: &str = "proof_evaluation_receipt_v1";
pub const HUMAN_VETO_BUNDLE_V1_SCHEMA: &str = "human_veto_bundle_v1";
pub const META_CHALLENGE_BUNDLE_V1_SCHEMA: &str = "meta_challenge_bundle_v1";
pub const SELF_HOSTING_BUILD_RECEIPT_V1_SCHEMA: &str = "self_hosting_build_receipt_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SpecBundleV1 {
    pub schema_version: String,
    pub spec_bundle_id: SpecBundleId,
    pub spec_name: String,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NormativeAstNodeV1 {
    pub node_id: String,
    pub kind: String,
    pub normative_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NormativeASTV1 {
    pub schema_version: String,
    pub normative_ast_id: NormativeAstId,
    pub spec_bundle_id: SpecBundleId,
    #[serde(default)]
    pub nodes: Vec<NormativeAstNodeV1>,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedSchemaFileV1 {
    pub path: String,
    pub digest: ContentDigest,
    #[serde(default)]
    pub source_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedAdmissionState {
    AdvisoryOnly,
    NonAdmitted,
    HumanVetoed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedSchemaBundleV1 {
    pub schema_version: String,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub spec_bundle_id: SpecBundleId,
    pub normative_ast_id: NormativeAstId,
    pub proof_obligation_set_id: ProofObligationSetId,
    pub schema_family: String,
    #[serde(default)]
    pub generated_files: Vec<GeneratedSchemaFileV1>,
    pub admission_state: GeneratedAdmissionState,
    pub advisory_only: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedInterpreterBundleV1 {
    pub schema_version: String,
    pub generated_interpreter_bundle_id: GeneratedInterpreterBundleId,
    pub spec_bundle_id: SpecBundleId,
    pub normative_ast_id: NormativeAstId,
    pub proof_obligation_set_id: ProofObligationSetId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub horizon_only: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedConformanceCorpusV1 {
    pub schema_version: String,
    pub generated_conformance_corpus_id: GeneratedConformanceCorpusId,
    pub spec_bundle_id: SpecBundleId,
    pub normative_ast_id: NormativeAstId,
    pub proof_obligation_set_id: ProofObligationSetId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub horizon_only: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedMigrationPlanV1 {
    pub schema_version: String,
    pub generated_migration_plan_id: GeneratedMigrationPlanId,
    pub spec_bundle_id: SpecBundleId,
    pub normative_ast_id: NormativeAstId,
    pub proof_obligation_set_id: ProofObligationSetId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub horizon_only: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofObligationV1 {
    pub obligation_id: String,
    pub description: String,
    pub satisfied: bool,
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofObligationSetV1 {
    pub schema_version: String,
    pub proof_obligation_set_id: ProofObligationSetId,
    pub spec_bundle_id: SpecBundleId,
    pub normative_ast_id: NormativeAstId,
    #[serde(default)]
    pub obligations: Vec<ProofObligationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofEvaluationReceiptV1 {
    pub schema_version: String,
    pub proof_evaluation_receipt_id: ProofEvaluationReceiptId,
    pub proof_obligation_set_id: ProofObligationSetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_schema_bundle_id: Option<GeneratedSchemaBundleId>,
    pub satisfied_count: usize,
    #[serde(default)]
    pub unsatisfied_blocking_obligations: Vec<String>,
    pub admission_allowed: bool,
    pub advisory_only: bool,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HumanVetoBundleV1 {
    pub schema_version: String,
    pub human_veto_bundle_id: HumanVetoBundleId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MetaChallengeBundleV1 {
    pub schema_version: String,
    pub meta_challenge_bundle_id: MetaChallengeBundleId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub challenge_summary: String,
    pub horizon_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedSurfaceGovernanceState {
    AdvisoryBaseline,
    ChallengePending,
    HumanVetoed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SelfHostingBuildReceiptV1 {
    pub schema_version: String,
    pub self_hosting_build_receipt_id: SelfHostingBuildReceiptId,
    pub spec_bundle_id: SpecBundleId,
    pub generated_schema_bundle_id: GeneratedSchemaBundleId,
    pub generated_interpreter_bundle_id: GeneratedInterpreterBundleId,
    pub generated_conformance_corpus_id: GeneratedConformanceCorpusId,
    pub generated_migration_plan_id: GeneratedMigrationPlanId,
    pub proof_evaluation_receipt_id: ProofEvaluationReceiptId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_veto_bundle_id: Option<HumanVetoBundleId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_challenge_bundle_id: Option<MetaChallengeBundleId>,
    pub governance_state: GeneratedSurfaceGovernanceState,
    pub rollback_required: bool,
    pub admission_allowed: bool,
    pub advisory_only: bool,
    #[serde(default)]
    pub notes: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCompanionBundles {
    pub schema_bundle: GeneratedSchemaBundleV1,
    pub interpreter_bundle: GeneratedInterpreterBundleV1,
    pub conformance_corpus: GeneratedConformanceCorpusV1,
    pub migration_plan: GeneratedMigrationPlanV1,
    pub proof_evaluation_receipt: ProofEvaluationReceiptV1,
    pub self_hosting_build_receipt: SelfHostingBuildReceiptV1,
}

pub fn generate_schema_bundle(
    spec: &SpecBundleV1,
    ast: &NormativeASTV1,
    obligations: &ProofObligationSetV1,
    schema_family: impl Into<String>,
    generated_at: impl Into<String>,
) -> (GeneratedSchemaBundleV1, ProofEvaluationReceiptV1) {
    let generated_at = generated_at.into();
    let evaluated_at = generated_at.clone();
    let schema_family = schema_family.into();
    let blocking = obligations
        .obligations
        .iter()
        .filter(|obligation| obligation.blocking && !obligation.satisfied)
        .map(|obligation| obligation.obligation_id.clone())
        .collect::<Vec<_>>();
    let file_payload = serde_json::json!({
        "spec_bundle_id": spec.spec_bundle_id,
        "normative_ast_id": ast.normative_ast_id,
        "source_node_ids": ast.nodes.iter().map(|node| node.node_id.clone()).collect::<Vec<_>>(),
    });
    let file = GeneratedSchemaFileV1 {
        path: format!("generated/{schema_family}/schema.json"),
        digest: ContentDigest::compute_json(&file_payload)
            .expect("schema file payload should be digestible"),
        source_node_ids: ast.nodes.iter().map(|node| node.node_id.clone()).collect(),
    };

    let bundle = GeneratedSchemaBundleV1 {
        schema_version: GENERATED_SCHEMA_BUNDLE_V1_SCHEMA.into(),
        generated_schema_bundle_id: GeneratedSchemaBundleId::generate(),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        proof_obligation_set_id: obligations.proof_obligation_set_id.clone(),
        schema_family,
        generated_files: vec![file],
        admission_state: GeneratedAdmissionState::AdvisoryOnly,
        advisory_only: true,
        generated_at,
    };

    let receipt = ProofEvaluationReceiptV1 {
        schema_version: PROOF_EVALUATION_RECEIPT_V1_SCHEMA.into(),
        proof_evaluation_receipt_id: ProofEvaluationReceiptId::generate(),
        proof_obligation_set_id: obligations.proof_obligation_set_id.clone(),
        generated_schema_bundle_id: Some(bundle.generated_schema_bundle_id.clone()),
        satisfied_count: obligations
            .obligations
            .iter()
            .filter(|obligation| obligation.satisfied)
            .count(),
        unsatisfied_blocking_obligations: blocking.clone(),
        admission_allowed: blocking.is_empty(),
        advisory_only: true,
        evaluated_at,
    };

    (bundle, receipt)
}

pub fn generate_companion_bundles(
    spec: &SpecBundleV1,
    ast: &NormativeASTV1,
    obligations: &ProofObligationSetV1,
    schema_family: impl Into<String>,
    generated_at: impl Into<String>,
) -> GeneratedCompanionBundles {
    let generated_at = generated_at.into();
    let (schema_bundle, proof_evaluation_receipt) =
        generate_schema_bundle(spec, ast, obligations, schema_family, generated_at.clone());
    let interpreter_bundle = GeneratedInterpreterBundleV1 {
        schema_version: GENERATED_INTERPRETER_BUNDLE_V1_SCHEMA.into(),
        generated_interpreter_bundle_id: GeneratedInterpreterBundleId::generate(),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        proof_obligation_set_id: obligations.proof_obligation_set_id.clone(),
        generated_schema_bundle_id: schema_bundle.generated_schema_bundle_id.clone(),
        horizon_only: true,
        generated_at: generated_at.clone(),
    };
    let conformance_corpus = GeneratedConformanceCorpusV1 {
        schema_version: GENERATED_CONFORMANCE_CORPUS_V1_SCHEMA.into(),
        generated_conformance_corpus_id: GeneratedConformanceCorpusId::generate(),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        proof_obligation_set_id: obligations.proof_obligation_set_id.clone(),
        generated_schema_bundle_id: schema_bundle.generated_schema_bundle_id.clone(),
        horizon_only: true,
        generated_at: generated_at.clone(),
    };
    let migration_plan = GeneratedMigrationPlanV1 {
        schema_version: GENERATED_MIGRATION_PLAN_V1_SCHEMA.into(),
        generated_migration_plan_id: GeneratedMigrationPlanId::generate(),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        normative_ast_id: ast.normative_ast_id.clone(),
        proof_obligation_set_id: obligations.proof_obligation_set_id.clone(),
        generated_schema_bundle_id: schema_bundle.generated_schema_bundle_id.clone(),
        horizon_only: true,
        generated_at: generated_at.clone(),
    };
    let self_hosting_build_receipt = establish_veto_challenge_baseline(
        spec,
        &schema_bundle,
        &interpreter_bundle,
        &conformance_corpus,
        &migration_plan,
        &proof_evaluation_receipt,
        None,
        None,
        generated_at.clone(),
    );

    GeneratedCompanionBundles {
        schema_bundle,
        interpreter_bundle,
        conformance_corpus,
        migration_plan,
        proof_evaluation_receipt,
        self_hosting_build_receipt,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn establish_veto_challenge_baseline(
    spec: &SpecBundleV1,
    schema_bundle: &GeneratedSchemaBundleV1,
    interpreter_bundle: &GeneratedInterpreterBundleV1,
    conformance_corpus: &GeneratedConformanceCorpusV1,
    migration_plan: &GeneratedMigrationPlanV1,
    proof_evaluation_receipt: &ProofEvaluationReceiptV1,
    human_veto: Option<&HumanVetoBundleV1>,
    meta_challenge: Option<&MetaChallengeBundleV1>,
    generated_at: impl Into<String>,
) -> SelfHostingBuildReceiptV1 {
    let governance_state = if human_veto.is_some() {
        GeneratedSurfaceGovernanceState::HumanVetoed
    } else if meta_challenge.is_some() {
        GeneratedSurfaceGovernanceState::ChallengePending
    } else {
        GeneratedSurfaceGovernanceState::AdvisoryBaseline
    };
    let admission_allowed = proof_evaluation_receipt.admission_allowed && human_veto.is_none();

    SelfHostingBuildReceiptV1 {
        schema_version: SELF_HOSTING_BUILD_RECEIPT_V1_SCHEMA.into(),
        self_hosting_build_receipt_id: SelfHostingBuildReceiptId::generate(),
        spec_bundle_id: spec.spec_bundle_id.clone(),
        generated_schema_bundle_id: schema_bundle.generated_schema_bundle_id.clone(),
        generated_interpreter_bundle_id: interpreter_bundle.generated_interpreter_bundle_id.clone(),
        generated_conformance_corpus_id: conformance_corpus.generated_conformance_corpus_id.clone(),
        generated_migration_plan_id: migration_plan.generated_migration_plan_id.clone(),
        proof_evaluation_receipt_id: proof_evaluation_receipt.proof_evaluation_receipt_id.clone(),
        human_veto_bundle_id: human_veto.map(|bundle| bundle.human_veto_bundle_id.clone()),
        meta_challenge_bundle_id: meta_challenge
            .map(|bundle| bundle.meta_challenge_bundle_id.clone()),
        governance_state,
        rollback_required: human_veto.is_some() || meta_challenge.is_some(),
        admission_allowed,
        advisory_only: true,
        notes: vec![
            "generated surfaces remain non-admitted until proof obligations pass and a human lane accepts them".into(),
            "self-hosting status does not override veto, challenge, or rollback obligations".into(),
        ],
        generated_at: generated_at.into(),
    }
}
