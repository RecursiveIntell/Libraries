# Phase 07 — Synthetic Tests and Benchmark Harness

## Objective

Add local tests and optional criterion benches for alpha gates.

## Required actions

1. Add MHA/MQA/GQA synthetic fixtures.
2. Add deterministic replay tests.
3. Add shape/layout rejection tests.
4. Add benchmark skeleton guarded by feature if needed.
5. Run full workspace tests.

## Acceptance gate

Synthetic gates pass and benchmark skeleton does not introduce release claims.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
