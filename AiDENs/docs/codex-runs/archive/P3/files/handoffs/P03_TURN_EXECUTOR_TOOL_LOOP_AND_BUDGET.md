# P03 Turn Executor, Tool Loop, And Budget Handoff

## Scope

Implemented P03 only. Later pass work remains deferred.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-budget-kit/src/lib.rs`
- `crates/aidens-runner/Cargo.toml`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-runner/tests/next_turn_executor.rs`
- `tests/fixtures/p03/turn_execution_plan_v1.json`
- `tests/fixtures/p03/turn_receipt_v1.json`
- `tests/fixtures/p03/tool_call_request_v1.json`
- `tests/fixtures/p03/tool_call_result_v1.json`
- `tests/fixtures/p03/stop_rule_receipt_v1.json`
- `tests/fixtures/p03/budget_exhaustion_receipt_v1.json`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `STATUS.md`

## Tests Added

- Contract coverage for P03 artifact deserialization and tool invocation receipt linkage.
- Budget coverage for cumulative tool-call limits and explicit turn deadlines.
- Runner coverage for mock provider repo-read tool loop execution.
- Runner coverage for budget exhaustion returning a blocked/degraded turn instead of looping.
- Runner coverage for blocking unexposed tool calls before dispatch.

## Commands Run

- `cargo check -p aidens-contracts -p aidens-provider-kit -p aidens-budget-kit -p aidens-tool-kit -p aidens-runner`
- `cargo fmt --all`
- `cargo test -p aidens-contracts -p aidens-provider-kit -p aidens-budget-kit -p aidens-runner`
- `cargo test -p aidens-runner`
- `cargo test -p aidens-contracts`
- `cargo test -p aidens-tool-kit`
- `cargo test -p aidens-app-kit`
- `cargo test -p aidens-cli`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Final required gate is recorded in `STATUS.md`.

## Blockers

None for P03 acceptance.

Deferred by build order:

- Durable receipt persistence remains P05 work.
- Tool permit lifecycle hardening remains P04 work.
- Boundary parser hardening beyond explicit degraded parser fallback remains P06 work.

## Next-Pass Readiness

P04 can start from an explicit `TurnExecutorV1` path with typed turn plans, tool call requests/results, invocation receipts, stop-rule receipts, and budget exhaustion receipts. Tool calls are now blocked unless listed in the current `ToolExposurePlanV1`, which gives P04 a concrete enforcement surface for permits and lifecycle policy.
