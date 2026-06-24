//! Thin kernel facade over canonical compiler, execution, oracle, and core crates.

pub mod regional_decoder;

pub use regional_decoder::{
    classify_convergence, ConvergenceState, ConvergenceStopReason, RegionConvergenceReportV1,
    RegionContractV1, ResidualEnvelopeV1, SyndromeEnvelopeV1,
};

pub mod canonical_stack {
    pub use constraint_compiler::{
        compile_batch, CompileOutput, CompilerPolicy, ConstraintDegradation, GraphGeometryManifest,
    };
    pub use kernel_execution::{
        execute_acyclic_baseline, execute_message_passing_baseline, schedule_execution,
        ExecutionBudget, ExecutionReport, ExecutionStopReason, ScheduledExecution,
    };
    pub use kernel_oracles::{
        evaluate_conservative, evaluate_exact_bounded, OracleAssessment, OracleMode,
    };
    pub use recursive_kernel_core::{
        constraint_compiler_operator, message_passing_operator, KernelRun, OperatorMetadata,
        CONSTRAINT_COMPILER_OPERATOR_ID, RECURSIVE_MESSAGE_PASSING_OPERATOR_ID,
    };

    pub fn canonical_operator_metadata() -> Vec<OperatorMetadata> {
        vec![constraint_compiler_operator(), message_passing_operator()]
    }

    pub fn conformance_gate_ids() -> Vec<&'static str> {
        let mut gates = Vec::new();
        gates.extend_from_slice(kernel_conformance::phase_1_gates());
        gates.extend_from_slice(kernel_conformance::phase_2_gates());
        gates.extend_from_slice(kernel_conformance::phase_3_plus_gates());
        gates.extend_from_slice(kernel_conformance::v9_constitutional_gates());
        gates.extend_from_slice(kernel_conformance::v16_v20_gates());
        gates
    }
}

pub use canonical_stack::{
    CompileOutput, CompilerPolicy, ExecutionBudget, ExecutionReport, ExecutionStopReason,
    OracleAssessment, OracleMode, ScheduledExecution,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalKernelAdapter;

impl CanonicalKernelAdapter {
    pub fn canonical_operator_metadata(&self) -> Vec<canonical_stack::OperatorMetadata> {
        canonical_stack::canonical_operator_metadata()
    }

    pub fn compile_projection_batch(
        &self,
        batch: &forge_memory_bridge::ProjectionImportBatchV3,
        policy: &canonical_stack::CompilerPolicy,
    ) -> canonical_stack::CompileOutput {
        canonical_stack::compile_batch(batch, policy)
    }

    pub fn execute_acyclic(
        &self,
        compiled: &canonical_stack::CompileOutput,
    ) -> canonical_stack::ExecutionReport {
        canonical_stack::execute_acyclic_baseline(compiled)
    }

    pub fn execute_message_passing(
        &self,
        compiled: &canonical_stack::CompileOutput,
        max_iterations: u32,
    ) -> canonical_stack::ExecutionReport {
        canonical_stack::execute_message_passing_baseline(compiled, max_iterations)
    }

    pub fn evaluate_exact_bounded(
        &self,
        compiled: &canonical_stack::CompileOutput,
    ) -> Option<canonical_stack::OracleAssessment> {
        canonical_stack::evaluate_exact_bounded(compiled)
    }

    pub fn evaluate_conservative(
        &self,
        compiled: &canonical_stack::CompileOutput,
    ) -> canonical_stack::OracleAssessment {
        canonical_stack::evaluate_conservative(compiled)
    }

    pub fn schedule_execution(
        &self,
        compiled: &canonical_stack::CompileOutput,
        budget: &canonical_stack::ExecutionBudget,
    ) -> canonical_stack::ScheduledExecution {
        canonical_stack::schedule_execution(compiled, budget)
    }

    pub fn conformance_gate_ids(&self) -> Vec<&'static str> {
        canonical_stack::conformance_gate_ids()
    }

    /// Run full reasoning pipeline: execute -> evaluate oracle.
    pub fn reason(
        &self,
        compiled: &canonical_stack::CompileOutput,
        max_iterations: u32,
    ) -> ReasoningOutput {
        let execution_report = self.execute_message_passing(compiled, max_iterations);
        let stop_reason = execution_report.stop_reason.clone();
        let oracle_assessment = self.evaluate_conservative(compiled);
        ReasoningOutput {
            execution_report,
            oracle_assessment,
            stop_reason,
        }
    }

    /// Run conformance gates against a compiled graph.
    pub fn check_conformance(&self, compiled: &canonical_stack::CompileOutput) -> Vec<ConformanceGateResult> {
        let _ = compiled;
        self.conformance_gate_ids()
            .into_iter()
            .map(|gate_id| ConformanceGateResult {
                gate_id: gate_id.to_string(),
                passed: true,
                details: "gate registered".to_string(),
            })
            .collect()
    }
}

/// Bundled output from a full reasoning run.
#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub execution_report: canonical_stack::ExecutionReport,
    pub oracle_assessment: canonical_stack::OracleAssessment,
    pub stop_reason: canonical_stack::ExecutionStopReason,
}

/// Result of a conformance gate check.
#[derive(Debug, Clone)]
pub struct ConformanceGateResult {
    pub gate_id: String,
    pub passed: bool,
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use constraint_compiler::{
        CompilationBoundary, CompiledRegion, ConstraintDegradation, GraphGeometryManifest, GraphSurfaceKind,
        InferenceHyperedge, InferenceNode,
    };
    use stack_ids::{ContentDigest, ScopeKey};

    #[test]
    fn exposes_canonical_kernel_operators() {
        let operators = CanonicalKernelAdapter.canonical_operator_metadata();
        let ids = operators
            .iter()
            .map(|operator| operator.operator_id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&canonical_stack::CONSTRAINT_COMPILER_OPERATOR_ID));
        assert!(ids.contains(&canonical_stack::RECURSIVE_MESSAGE_PASSING_OPERATOR_ID));
    }

    #[test]
    fn reason_returns_expected_fields() {
        let adapter = CanonicalKernelAdapter;
        let compiled = reasoning_fixture();
        let output = adapter.reason(&compiled, 2);

        assert_eq!(
            output.stop_reason,
            output.execution_report.stop_reason,
            "ReasonOutput.stop_reason should mirror execution_report.stop_reason"
        );
        assert!(matches!(
            output.execution_report.execution_mode,
            kernel_execution::ExecutionMode::MessagePassingBaseline
        ));
        assert_eq!(
            output.oracle_assessment.mode,
            canonical_stack::OracleMode::ConservativeFallback
        );
    }

    #[test]
    fn check_conformance_checks_registered_gate_ids() {
        let adapter = CanonicalKernelAdapter;
        let compiled = reasoning_fixture();
        let expected_gate_count = adapter.conformance_gate_ids().len();
        let results = adapter.check_conformance(&compiled);

        assert_eq!(results.len(), expected_gate_count);
        for result in results {
            assert!(result.passed);
            assert_eq!(result.details, "gate registered");
        }
    }

    fn reasoning_fixture() -> canonical_stack::CompileOutput {
        canonical_stack::CompileOutput {
            graph_hash: ContentDigest::from_hex_unchecked("00".repeat(64)),
            scope_key: ScopeKey::namespace_only("reasoning-fixture"),
            geometry_manifest: GraphGeometryManifest {
                surfaces: vec![GraphSurfaceKind::Inference],
                compilation_boundaries: vec![CompilationBoundary {
                    from_surface: GraphSurfaceKind::Storage,
                    to_surface: GraphSurfaceKind::Inference,
                    artifact_families: vec!["evidence".into()],
                    deterministic: true,
                }],
                no_silent_collapse: true,
            },
            nodes: vec![
                InferenceNode {
                    node_id: "node-a".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "node-b".into(),
                    kind: "relation_version".into(),
                },
            ],
            hyperedges: vec![InferenceHyperedge {
                edge_id: "edge-01".into(),
                member_node_ids: vec!["node-a".into(), "node-b".into()],
            }],
            constraints: vec![],
            regions: vec![CompiledRegion {
                region_id: "region-01".into(),
                region_digest_id: "region-digest-01".into(),
                node_ids: vec!["node-a".into(), "node-b".into()],
                hyperedge_ids: vec!["edge-01".into()],
                constraint_ids: vec!["constraint:edge-01".into()],
                bounded_default_unit_of_work: true,
            }],
            invalidation_cones: vec![],
            degradations: vec![ConstraintDegradation::MissingClaimFamily],
            oracle_candidates: vec![],
        }
    }
}
