# Libraries Hostile Remediation — Hermes Orchestration Pack

A self-contained pack for Hermes to coordinate isolated Codex agents through the hostile
remediation of `RecursiveIntell/Libraries`.

## Operator entrypoint

1. Attach this directory or the ZIP archive to Hermes.
2. Paste `OPERATOR_PASTE_FIRST.md`.
3. Provide the checkout path; the default assumed path is `~/Coding/Libraries`.
4. Hermes verifies the pack and captures a read-only baseline before source edits.

## Contents

- source-bound audit basis and issue matrix;
- Hermes master prompt and orchestration state machine;
- phased work orders, dependency DAG, file locks, merge protocol;
- Codex implementer/reviewer/migration/conformance prompts;
- acceptance, validation, rollback, final-state, and auditor contracts;
- machine-readable workstreams, validation matrix, state/handoff/receipt schemas;
- executable standard-library-only tooling for receipts, workspace inventory, ID authority,
  placeholder codec detection, evidence consistency, lint inheritance, and claims provenance.

## Nonclaims

This pack is not a patch and does not prove the repository builds. The source audit was
connector-backed static inspection. Current files, tests, logs, and source-bound receipts
outrank this pack when they differ.

## Read order

1. `00_START_HERE.md`
2. `01_SOURCE_BASIS.md`
3. `02_HERMES_MASTER_PROMPT.md`
4. `03_ORCHESTRATION_RUNBOOK.md`
5. `07_GLOBAL_GUARDRAILS.md`
6. `04_EXECUTION_PLAN.md`
7. `05_ISSUE_MATRIX.json`
8. assigned phase work order
