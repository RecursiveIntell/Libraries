# Phase 01 — Scope Freeze and Final File-Tree Plan

## Objective

Translate bundle specs into a repo-specific final plan without implementation drift.

## Required actions

1. Read all docs in `docs/`.
2. Decide whether repo is empty or existing workspace.
3. Produce `.codex-runs/<run-id>/source_inventory.md`.
4. Produce `.codex-runs/<run-id>/phase_plan.md`.
5. Confirm out-of-scope items are not planned.

## Acceptance gate

Plan names exact crates/files and explicitly excludes governor/app integrations.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
