# Phase 00 — Preflight Hostile Audit

## Goal
Build the current truth inventory before touching code.

## Required actions

1. Inspect current repo state and package reports.
2. Run the existing P22 verifier and record whether it passes in the working repo.
3. Run package extraction replay if a package is available.
4. Inventory all Pxx/Pyy-marked artifacts outside `docs/codex-runs/archive/`.
5. Inventory `z.py`, `zip.py`, verifier scripts, package modes, current-run allowlists, and script-reference checks.
6. Inventory actual AiDENs product surfaces and identify the smallest high-ROI capability slice to build in P23.

## Required evidence

- `target/p23/audit/phase00_inventory.json`
- `target/p23/audit/phase00_pxx_artifact_inventory.csv`
- `target/p23/audit/phase00_zpy_inventory.md`
- `target/p23/audit/phase00_capability_gap.md`
- `handoffs/p23/PHASE_00_REPORT.md`

## Acceptance gate

Do not edit implementation until the report explicitly states:

- known P22 package replay failure status,
- all active Pxx/Pyy artifacts and their intended classification,
- exact `z.py` problems to fix,
- capability slice chosen for P23.
