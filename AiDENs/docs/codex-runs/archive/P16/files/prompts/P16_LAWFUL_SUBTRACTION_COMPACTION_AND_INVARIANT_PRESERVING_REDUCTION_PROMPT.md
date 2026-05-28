# Codex Prompt — P16 Lawful subtraction, compaction, and invariant-preserving reduction

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P16_LAWFUL_SUBTRACTION_COMPACTION_AND_INVARIANT_PRESERVING_REDUCTION.md`.

Implement P16 only. Do not start later passes.

## Goal

Add certifying reduction operators that remove, compact, summarize, or forget while preserving declared invariants and history budgets.

## Primary crates

- `aidens-kernel-kit`
- `aidens-memory-kit`
- `aidens-repair-kit`
- `aidens-contracts`
- `aidens-receipts`

## Required artifacts

- `SubtractionPlanV1`
- `SupportCoreV1`
- `RemovalFrontierV1`
- `InvariantBudgetV1`
- `CompactionReceiptV1`
- `HistoryPreservationReportV1`

## Acceptance gates

- Subtraction cannot delete support needed by accepted claim unless claim is superseded/quarantined first.
- Compaction emits receipt and history preservation report with before/after digests.
- As-of queries remain correct under declared history budget after compaction.

## Forbidden shortcuts

- Do not call destructive deletion “subtraction”.
- Do not compact receipts that are required for audit without explicit retention policy and approval.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
