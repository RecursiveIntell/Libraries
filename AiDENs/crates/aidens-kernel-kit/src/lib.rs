//! Thin kernel facade over canonical compiler, execution, oracle, and core crates.

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
