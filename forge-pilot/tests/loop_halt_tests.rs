mod common;

use common::{base_loop_config, open_forge_store, open_memory_store, resources, tempdir};
use forge_pilot::{HaltReason, LoopRunner, PILOT_LOOP_RECEIPT_V1_SCHEMA};
use knowledge_runtime::Scope;

#[tokio::test]
async fn loop_runner_stops_when_time_budget_is_exhausted() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-time-budget");
    let mut config = base_loop_config(scope.clone());
    config.time_budget_secs = 0;

    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    assert_eq!(report.halt_reason, HaltReason::TimeBudgetExhausted);
    assert_eq!(report.receipt.schema_version, PILOT_LOOP_RECEIPT_V1_SCHEMA);
    assert!(report.receipt.non_authoritative);
    assert_eq!(report.receipt.budget.workload_class, "orchestration");
    assert_eq!(report.receipt.budget.time_budget_secs, 0);
    assert_eq!(report.receipt.halt_reason, HaltReason::TimeBudgetExhausted);
    assert_eq!(report.receipt.iterations_completed, 0);
}
