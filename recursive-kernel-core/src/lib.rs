use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{ConstraintId, KernelRunId, OperatorId, OracleSliceId, ScopeKey, SyndromeId};

pub const CONSTRAINT_COMPILER_OPERATOR_ID: &str = "rik.constraint_compiler.v1";
pub const RECURSIVE_MESSAGE_PASSING_OPERATOR_ID: &str = "rik.message_passing.v1";

pub const CONSTRAINT_COMPILER_OPERATOR_NAME: &str = "constraint_compiler";
pub const RECURSIVE_MESSAGE_PASSING_OPERATOR_NAME: &str = "recursive_message_passing";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExactnessClass {
    Exact,
    Conservative,
    Approximate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConvergenceKind {
    AcyclicSinglePass,
    ResidualBounded,
    FixedPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StopRule {
    MaxIterations { limit: u32 },
    ResidualBelow { micros: u64 },
    AcyclicCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DegradationBehavior {
    Blocked,
    AdvisoryOnly,
    ConservativeFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactAuthorityClass {
    NonAuthoritativeDerived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperatorContract {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub exactness: ExactnessClass,
    pub convergence: ConvergenceKind,
    pub stop_rule: StopRule,
    pub degradation: DegradationBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperatorMetadata {
    pub operator_id: OperatorId,
    pub name: String,
    pub contract: OperatorContract,
}

impl OperatorMetadata {
    /// Validates the minimal operator metadata contract exposed to downstream schedulers.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("operator name must not be empty".into());
        }
        if self.contract.inputs.is_empty() {
            return Err("operator inputs must not be empty".into());
        }
        if self.contract.outputs.is_empty() {
            return Err("operator outputs must not be empty".into());
        }
        Ok(())
    }
}

/// Returns the canonical metadata for the bounded constraint-compiler operator.
pub fn constraint_compiler_operator() -> OperatorMetadata {
    OperatorMetadata {
        operator_id: OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
        name: CONSTRAINT_COMPILER_OPERATOR_NAME.to_string(),
        contract: OperatorContract {
            inputs: vec!["projection_import_batch_v3".into()],
            outputs: vec!["inference_graph".into()],
            exactness: ExactnessClass::Exact,
            convergence: ConvergenceKind::AcyclicSinglePass,
            stop_rule: StopRule::AcyclicCompletion,
            degradation: DegradationBehavior::AdvisoryOnly,
        },
    }
}

/// Returns the canonical metadata for the bounded recursive message-passing operator.
pub fn message_passing_operator() -> OperatorMetadata {
    OperatorMetadata {
        operator_id: OperatorId::new(RECURSIVE_MESSAGE_PASSING_OPERATOR_ID),
        name: RECURSIVE_MESSAGE_PASSING_OPERATOR_NAME.to_string(),
        contract: OperatorContract {
            inputs: vec!["inference_graph".into()],
            outputs: vec!["execution_report".into()],
            exactness: ExactnessClass::Conservative,
            convergence: ConvergenceKind::FixedPoint,
            stop_rule: StopRule::ResidualBelow { micros: 1_000 },
            degradation: DegradationBehavior::AdvisoryOnly,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KernelRun {
    pub run_id: KernelRunId,
    pub policy_version: String,
    pub scope_key: ScopeKey,
    pub operator_ids: Vec<OperatorId>,
    pub oracle_slice_id: Option<OracleSliceId>,
    pub degradation_active: bool,
}

impl KernelRun {
    /// Marks every kernel run artifact as derived rather than directly authoritative.
    pub fn authority_class(&self) -> ArtifactAuthorityClass {
        ArtifactAuthorityClass::NonAuthoritativeDerived
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintUnit {
    pub constraint_id: ConstraintId,
    pub kind: String,
    pub variable_ids: Vec<String>,
    pub operator_id: OperatorId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Syndrome {
    pub syndrome_id: SyndromeId,
    pub signature: String,
    pub blocked_by_degradation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KernelArtifactKind {
    Residual,
    Syndrome,
    Witness,
    Certificate,
    CalibrationReport,
    KernelRefutationResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResidualArtifact {
    pub constraint_id: ConstraintId,
    pub residual_micros: u64,
    pub monotone_nonincreasing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WitnessArtifact {
    pub witness_id: String,
    pub target_node_id: String,
    pub supporting_constraint_ids: Vec<ConstraintId>,
    pub belief_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CertificateArtifact {
    pub certificate_id: String,
    pub certified_node_id: String,
    pub satisfied_constraint_ids: Vec<ConstraintId>,
    pub oracle_slice_id: Option<OracleSliceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CalibrationReport {
    pub nuisance_node_ids: Vec<String>,
    pub caveats: Vec<String>,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KernelRefutationOutcome {
    FlipWitnessFound,
    NoFlipFoundWithinBudget,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PerturbationWitness {
    pub changed_node_ids: Vec<String>,
    pub changed_constraint_ids: Vec<ConstraintId>,
    pub budget_used: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KernelRefutationResult {
    pub target_node_id: String,
    pub outcome: KernelRefutationOutcome,
    pub witness: Option<PerturbationWitness>,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_contract_requires_inputs_outputs_and_stop_rule() {
        let operator = constraint_compiler_operator();
        assert_eq!(
            operator.operator_id.as_str(),
            CONSTRAINT_COMPILER_OPERATOR_ID
        );
        assert_eq!(operator.name, CONSTRAINT_COMPILER_OPERATOR_NAME);
        assert_eq!(operator.contract.inputs, vec!["projection_import_batch_v3"]);
        assert_eq!(operator.contract.outputs, vec!["inference_graph"]);
        assert!(matches!(operator.contract.exactness, ExactnessClass::Exact));
        operator.validate().unwrap();
    }

    #[test]
    fn kernel_artifacts_remain_non_authoritative() {
        let run = KernelRun {
            run_id: KernelRunId::new("run-1"),
            policy_version: "policy-1".into(),
            scope_key: ScopeKey::namespace_only("ns"),
            operator_ids: vec![OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID)],
            oracle_slice_id: None,
            degradation_active: false,
        };

        assert_eq!(
            run.authority_class(),
            ArtifactAuthorityClass::NonAuthoritativeDerived
        );
    }

    #[test]
    fn shared_artifact_schemas_are_serializable() {
        let witness = WitnessArtifact {
            witness_id: "wit-1".into(),
            target_node_id: "node-1".into(),
            supporting_constraint_ids: vec![ConstraintId::new("1")],
            belief_micros: 750_000,
        };
        let encoded = serde_json::to_string(&witness).unwrap();
        assert!(encoded.contains("wit-1"));
    }
}
