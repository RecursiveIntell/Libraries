# Codex Prompt — P03 TurnExecutorV1, provider/tool loop, and budgeted dispatch

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P03_TURN_EXECUTOR_TOOL_LOOP_AND_BUDGET.md`.

Implement P03 only. Do not start later passes.

## Goal

Replace one-shot provider completion with a budgeted turn executor that can expose tools, parse/receive tool calls, dispatch, and continue until final output or stop condition.

## Primary crates

- `aidens-runner`
- `aidens-provider-kit`
- `aidens-tool-kit`
- `aidens-budget-kit`
- `aidens-boundary-kit`
- `aidens-receipts`

## Required artifacts

- `TurnExecutionPlanV1`
- `TurnReceiptV1`
- `ToolCallRequestV1`
- `ToolCallResultV1`
- `StopRuleReceiptV1`
- `BudgetExhaustionReceiptV1`

## Acceptance gates

- A mock/provider fixture can request repo-read and receive the tool result before final answer.
- Every tool call has a receipt linked to run_id, attempt_id, tool_id, input digest, output digest, and outcome.
- Budget exhaustion returns a degraded/blocked final state with receipt; it does not loop forever or silently truncate.

## Forbidden shortcuts

- Do not handwave tool calls as prompt text without degraded receipt.
- Do not let a provider call a tool that was not exposed in ToolExposurePlanV1.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
