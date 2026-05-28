# Codex Prompt — P05 Durable execution evidence ledger, receipt store, and transactional outbox

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P05_DURABLE_EXECUTION_EVIDENCE_LEDGER_AND_OUTBOX.md`.

Implement P05 only. Do not start later passes.

## Goal

Replace in-memory-only receipts with durable append-only execution evidence and same-transaction outbox semantics.

## Primary crates

- `aidens-receipts`
- `aidens-contracts`
- `aidens-cli`
- `aidens-runner`
- `aidens-daemon-kit`

## Required artifacts

- canonical library receipt log/sink
- `PoisonReceiptRecordV1`
- `ExecutionLineageGraphV1`

## Acceptance gates

- A run can be inspected after process restart via durable store.
- Receipt digests are stable across pretty/compact JSON representation.
- Every provider/tool/boundary/permit failure has an emitted receipt or explicit test proving no durable store was configured and receipt-level is minimal.

## Forbidden shortcuts

- Do not keep “durable” evidence only in Vec/Mutex.
- Do not export receipts by scanning mutable tables without outbox identity.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
