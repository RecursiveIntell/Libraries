# P27 Phase Report

## Phase

- Phase ID: 04
- Phase title: Ownership scanner fail-closed behavior
- Date: 2026-05-04T23:06:05Z

## Scope

- Intended work: make the ownership scanner fail closed when canonical sibling baseline is absent and prove that behavior with receipts.
- Issue IDs in scope: `P27-004`.
- Explicit non-goals: no capability work, no canonical-owner boundary changes, no support-claim widening, no root Markdown cleanup, no vendoring sibling crates.

## Files inspected

- `prompts/phases/P27_PHASE_04_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_04_BEFORE_PHASE_05.md`
- `scripts/make_type_ownership_inventory.py`
- `scripts/assert_p27_ownership_scanner_fail_closed.py`
- `scripts/assert_package_self_replay.py`
- `STATUS.md`
- `docs/contract-ownership/*`

## Files changed

- `scripts/make_type_ownership_inventory.py`
- `scripts/assert_p27_ownership_scanner_fail_closed.py`
- `scripts/assert_package_self_replay.py`
- `STATUS.md`
- `docs/contract-ownership/OWNERSHIP_SCAN_STATUS.json`
- `handoffs/p27/PHASE_04_REPORT.md`

## Changes made

- Added `docs/contract-ownership/OWNERSHIP_SCAN_STATUS.json` generation to the ownership scanner.
- Added explicit `canonical_inventory_unavailable=true|false` output.
- Kept absent canonical inventory as exit code `2` unless `--aidens-overlay-only` is explicitly used.
- Replaced the P27 ownership guard with a behavioral absent-baseline fixture check.
- Updated package replay receipts to record `P27_SKIP_CARGO`/`P27_REQUIRE_CARGO`; replay under `P27_SKIP_CARGO=1` is now labeled `semantic_status: degraded_exact_check`.
- Updated `STATUS.md` for `P27-001`, `P27-002`, and `P27-004` after the Phase 04 evidence.

## Commands run

| Command | Result | Log |
|---|---|---|
| `python3 scripts/make_type_ownership_inventory.py --root .` before edit | pass locally, but no fail-closed marker | `target/p27/audit/phase04_inventory_present_before.log` |
| `python3 scripts/assert_p27_ownership_scanner_fail_closed.py .` before edit | fail | `target/p27/audit/phase04_assert_ownership_before.log` |
| `python3 -m py_compile ...` | pass | `target/p27/audit/phase04_py_compile_final.log` |
| `python3 scripts/assert_p27_ownership_scanner_fail_closed.py .` | pass | `target/p27/audit/phase04_assert_ownership_final.log` |
| absent-baseline scanner fixture | fail-closed with exit code `2` and `canonical_inventory_unavailable=true` | `target/p27/audit/phase04_inventory_absent_fixture.log`, `target/p27/audit/phase04_inventory_absent_fixture_status.json`, `target/p27/audit/phase04_inventory_absent_fixture_exit_code.log` |
| `python3 scripts/make_type_ownership_inventory.py --root .` | pass; `canonical_inventory_unavailable=false` | `target/p27/audit/phase04_inventory_present_after.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase04_verify_current_skip_cargo_final.log` |
| `python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P27 --output target/p27/package/AiDENs-p27-codex-context.zip` | pass | `target/p27/audit/phase04_zpy_package_final.log` |
| `python3 scripts/assert_package_validation.py` | pass | `target/p27/audit/phase04_package_validation_final.log` |
| `P27_SKIP_CARGO=1 python3 scripts/assert_package_self_replay.py --package target/p27/package/AiDENs-p27-codex-context.zip --verifier scripts/verify_current.sh --receipt-out target/p27/audit/phase04_package_self_replay_receipt_final.json` | pass, degraded by explicit cargo skip | `target/p27/audit/phase04_package_self_replay_final.log`, `target/p27/audit/phase04_package_self_replay_receipt_final.json` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase04_assert_current_run_truth_final.log` |

## Evidence emitted

- `docs/contract-ownership/OWNERSHIP_SCAN_STATUS.json`
- `target/p27/audit/phase04_inventory_absent_fixture.log`
- `target/p27/audit/phase04_inventory_absent_fixture_status.json`
- `target/p27/audit/phase04_inventory_absent_fixture_exit_code.log`
- `target/p27/audit/phase04_inventory_present_after.log`
- `target/p27/audit/phase04_assert_ownership_final.log`
- `target/p27/audit/phase04_verify_current_skip_cargo_final.log`
- `target/p27/audit/phase04_zpy_package_final.log`
- `target/p27/audit/phase04_package_validation_final.log`
- `target/p27/audit/phase04_package_self_replay_final.log`
- `target/p27/audit/phase04_package_self_replay_receipt_final.json`
- `target/p27/audit/phase04_package_summary_facts_final.log`
- `target/p27/package/AiDENs-p27-codex-context.zip`
- `target/p27/package/AiDENs-p27-codex-context.report.md`
- `target/p27/package/AiDENs-p27-codex-context.findings.json`

## Ownership scan result

- Present local sibling baseline: `canonical_inventory_unavailable=false`.
- Canonical local definitions scanned: `633`.
- AiDENs contract local definitions scanned: `206`.
- Duplicate findings: `0`.
- Absent-baseline fixture: exit code `2`, `canonical_inventory_unavailable=true`, and known limit recorded.

## Package replay result

- Package strict validation: pass.
- Package findings: `0` errors, `0` warnings.
- Package archive SHA-256: `c1e8475050ac4bab66227be8961012b533a8bb9127044a80346c370494dd8550`.
- External Cargo path dependency roots included: `40`.
- Replay status: `passed`.
- Replay semantic status: `degraded_exact_check`.
- Replay degradation: cargo checks were skipped with `P27_SKIP_CARGO=1`.

## 11A semantic impact

- Exact/approx labels touched: `OWNERSHIP_SCAN_STATUS.json` uses `semantic_status: exact_check`; package replay receipt uses `semantic_status: degraded_exact_check` when cargo is skipped.
- Proof/check hooks added: absent-baseline fixture, ownership scan status receipt, package replay execution context.
- Degradation/support labels changed: replay receipts now record cargo-skip degradation and `support_tier: verification`.

The ownership status and replay receipt are AiDENs-local operator evidence. Canonical type ownership remains delegated to sibling crates.

## Support profile impact

- No support-tier claim changed.
- Package replay is green only for the explicitly degraded `P27_SKIP_CARGO=1` verifier mode; full cargo-backed replay remains deferred to later validation/final gate.

## Issues closed

- `P27-004`: ownership scanner now fails closed when canonical baseline is absent and emits `canonical_inventory_unavailable=true`.

## New issues / risks

- `P27-005` remains open: sibling workspace prerequisites still need a dedicated checker and source-basis hardening.
- Root Markdown archive hygiene remains out of scope for Phase 04.
- Full cargo-backed package replay has not been run in Phase 04.

## Decision

Rationale: The ownership scanner no longer produces a false-clean result when canonical baseline is absent, the current verifier passes with cargo explicitly skipped, and package self-replay now passes with the cargo-skip degradation recorded.

Decision: continue
