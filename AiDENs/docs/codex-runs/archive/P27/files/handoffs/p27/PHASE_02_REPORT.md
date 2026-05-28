# P27 Phase Report

## Phase

- Phase ID: 02
- Phase title: Active run truth surface normalization
- Date: 2026-05-04T22:54:52Z

## Scope

- Intended work: make active current-run docs agree on P27 and classify P24/P25/P26 material as historical evidence, not active doctrine.
- Issue IDs in scope: `P27-003`.
- Explicit non-goals: no capability work, no package self-replay proof, no ownership scanner repair, no root Markdown archive sweep, no support-claim widening, no canonical-owner boundary changes.

## Files inspected

- `prompts/phases/P27_PHASE_02_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_02_BEFORE_PHASE_03.md`
- `handoffs/p27/PHASE_01_REPORT.md`
- `scripts/assert_p27_current_run_truth.py`
- `scripts/assert_p27_agents_md_current.py`
- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`

## Files changed

- `README.md`
- `STATUS.md`
- `handoffs/p27/PHASE_02_REPORT.md`

## Changes made

- Updated `STATUS.md` to record that `P27-001` closed in Phase 01 while full verifier success remains blocked by `P27-004`.
- Updated `STATUS.md` to record `P27-003` as closed in Phase 02.
- Added a short phase evidence ledger to `STATUS.md`.
- Updated `README.md` opening seams so it no longer describes stale active docs or stale `AGENTS.md` doctrine as current facts.
- Preserved P24/P25/P26 references only as prior/historical evidence.

## Commands run

| Command | Result | Log |
|---|---|---|
| `python3 scripts/assert_p27_current_run_truth.py .` before edit | pass | `target/p27/audit/phase02_assert_p27_current_run_truth_before.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` before edit | pass | `target/p27/audit/phase02_assert_p27_agents_md_current_before.log` |
| `rg` active docs for P22-P27 run references before edit | inspected | `target/p27/audit/phase02_active_doc_run_refs_before.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` after edit | pass | `target/p27/audit/phase02_assert_p27_current_run_truth_after.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` after edit | pass | `target/p27/audit/phase02_assert_p27_agents_md_current_after.log` |
| `rg` active docs for P22-P27 run references after edit | inspected; active docs point to P27 and historical P24/P25/P26 references are labeled | `target/p27/audit/phase02_active_doc_run_refs_after.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` with pipefail | expected fail at out-of-scope ownership scanner guard after current-run checks pass | `target/p27/audit/phase02_verify_current_skip_cargo_after.log` |
| stale current-run phrase scan | pass; no `current run` P22-P26 phrase found | `target/p27/audit/phase02_stale_current_run_phrase_scan.log` |
| snapshots of changed docs | captured | `target/p27/audit/phase02_status_snapshot_after.md`, `target/p27/audit/phase02_readme_snapshot_after.md` |

## Evidence emitted

- `target/p27/audit/phase02_assert_p27_current_run_truth_before.log`
- `target/p27/audit/phase02_assert_p27_agents_md_current_before.log`
- `target/p27/audit/phase02_active_doc_run_refs_before.log`
- `target/p27/audit/phase02_assert_p27_current_run_truth_after.log`
- `target/p27/audit/phase02_assert_p27_agents_md_current_after.log`
- `target/p27/audit/phase02_active_doc_run_refs_after.log`
- `target/p27/audit/phase02_verify_current_skip_cargo_after.log`
- `target/p27/audit/phase02_stale_current_run_phrase_scan.log`
- `target/p27/audit/phase02_status_snapshot_after.md`
- `target/p27/audit/phase02_readme_snapshot_after.md`

## 11A semantic impact

- Exact/approx labels touched: none.
- Proof/check hooks added: none.
- Degradation/support labels changed: none.

The touched docs remain active local operator truth/evidence docs. No canonical sibling truth boundary changed.

## Support profile impact

- No support-tier claim changed.
- `SUPPORT_PROFILE.md` remains conservative: inherited supported-local surfaces are still `to-be-revalidated`.

## Issues closed

- `P27-003`: active `AGENTS.md`, `README.md`, `STATUS.md`, `SOURCE_BASIS.md`, and `SUPPORT_PROFILE.md` agree that P27 is the current run, and historical P24/P25/P26 docs are not active instructions.

## New issues / risks

- `P27-004` still blocks full `scripts/verify_current.sh`: `scripts/assert_p27_ownership_scanner_fail_closed.py` reports that `scripts/make_type_ownership_inventory.py` does not expose `canonical_inventory_unavailable`.
- `P27-002` remains open: package self-replay is not attempted in Phase 02 and no P27 package zip exists yet.
- Root Markdown drift remains for Phase 05; this phase only normalized the protected active docs.

## Decision

Rationale: Phase 02 active-run truth normalization is complete. Continue only to package replay / truth-surface phases; do not start capability work while replay and ownership scanner gates remain open.

Decision: continue
