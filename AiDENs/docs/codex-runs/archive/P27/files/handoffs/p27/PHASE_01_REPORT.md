# P27 Phase Report

## Phase

- Phase ID: 01
- Phase title: Verifier and CI hard repair
- Date: 2026-05-04T22:50:18Z

## Scope

- Intended work: repair or confirm the P27 current verifier entrypoint, compatibility wrappers, CI verifier target, and script-reference assertion surface.
- Issue IDs in scope: `P27-001`.
- Explicit non-goals: no capability work, no package self-replay proof, no ownership scanner repair beyond preserving the failing guard, no support-claim widening, no canonical-owner boundary changes.

## Files inspected

- `prompts/phases/P27_PHASE_01_PROMPT.md`
- `phase_injections/P27_GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md`
- `phase_injections/P27_GATE_AFTER_PHASE_01_BEFORE_PHASE_02.md`
- `handoffs/p27/PHASE_00_REPORT.md`
- `scripts/verify_current.sh`
- `scripts/verify.sh`
- `scripts/p27_verify.sh`
- `scripts/assert_p27_verifier_surface.py`
- `scripts/assert_script_refs_strict.py`
- `.github/workflows/ci.yml`

## Files changed

- `scripts/p27_verify.sh`
- `scripts/assert_p27_verifier_surface.py`
- `handoffs/p27/PHASE_01_REPORT.md`

## Changes made

- Added `scripts/assert_script_refs_strict.py` to the P27 verifier so strict script-reference resolution is part of the current verifier entrypoint.
- Fixed the bare historical verifier-name regex in `scripts/assert_p27_verifier_surface.py` by using a real word-boundary escape. This preserves the P27 allowance for `p27_verify.sh` and keeps historical `pNN_verify.sh` references fail-closed.
- Confirmed `.github/workflows/ci.yml` already calls `P27_REQUIRE_CARGO=1 bash scripts/verify_current.sh`.
- Confirmed `scripts/verify_current.sh` delegates to `scripts/p27_verify.sh`, and `scripts/verify.sh` delegates to `scripts/verify_current.sh`.

## Commands run

| Command | Result | Log |
|---|---|---|
| `python3 scripts/assert_script_refs_strict.py .` before edit | pass | `target/p27/audit/phase01_assert_script_refs_strict_before.log` |
| `python3 scripts/assert_p27_verifier_surface.py .` before edit | pass | `target/p27/audit/phase01_assert_p27_verifier_surface_before.log` |
| `grep -RIn "p[0-9][0-9]_verify\|verify_current\|verify.sh" scripts .github/workflows` before edit | pass; active scripts/CI target P27/current wrappers | `target/p27/audit/phase01_verifier_refs_before.txt` |
| `find scripts .github/workflows -maxdepth 3 -type f -print \| sort` | pass | `target/p27/audit/phase01_script_files_before.txt` |
| `python3 scripts/assert_p27_verifier_surface.py .` after edit | pass | `target/p27/audit/phase01_assert_p27_verifier_surface_after.log` |
| `python3 scripts/assert_script_refs_strict.py .` after edit | pass | `target/p27/audit/phase01_assert_script_refs_strict_after.log` |
| `bash -n scripts/p27_verify.sh scripts/verify_current.sh scripts/verify.sh` | pass | `target/p27/audit/phase01_bash_syntax.log` |
| `python3 -m py_compile scripts/assert_p27_verifier_surface.py scripts/assert_script_refs_strict.py` | pass | `target/p27/audit/phase01_py_compile.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` with pipefail | expected fail at out-of-scope ownership scanner guard after Phase 01 checks pass | `target/p27/audit/phase01_verify_current_skip_cargo_after.log` |
| `grep -RIn "p[0-9][0-9]_verify\|verify_current\|verify.sh" scripts .github/workflows` after edit | pass; no missing historical verifier target in active scripts/CI | `target/p27/audit/phase01_verifier_refs_after.txt` |

## Evidence emitted

- `target/p27/audit/phase01_assert_script_refs_strict_before.log`
- `target/p27/audit/phase01_assert_p27_verifier_surface_before.log`
- `target/p27/audit/phase01_verifier_refs_before.txt`
- `target/p27/audit/phase01_script_files_before.txt`
- `target/p27/audit/phase01_assert_p27_verifier_surface_after.log`
- `target/p27/audit/phase01_assert_script_refs_strict_after.log`
- `target/p27/audit/phase01_bash_syntax.log`
- `target/p27/audit/phase01_py_compile.log`
- `target/p27/audit/phase01_verify_current_skip_cargo_after.log`
- `target/p27/audit/phase01_verifier_refs_after.txt`

## 11A semantic impact

- Exact/approx labels touched: none.
- Proof/check hooks added: strict script-reference resolution is now a verifier check.
- Degradation/support labels changed: none.

The touched artifacts remain local verifier/operator evidence. No advisory result was promoted to truth.

## Support profile impact

- No support-tier claim changed.
- P27 current verifier support remains gated by later truth repairs, because the full verifier still fails on the ownership scanner fail-closed marker.

## Issues closed

- `P27-001`: current verifier wrapper targets exist; compatibility wrapper delegates to current verifier; CI targets `scripts/verify_current.sh`; strict script-reference assertion is wired into the current verifier and passes.

## New issues / risks

- `P27-004` still blocks full `scripts/verify_current.sh`: `scripts/assert_p27_ownership_scanner_fail_closed.py` reports that `scripts/make_type_ownership_inventory.py` does not expose `canonical_inventory_unavailable`.
- `P27-002` remains open: package self-replay is not attempted in Phase 01 and no P27 package zip exists yet.

## Decision

Rationale: Phase 01 verifier/CI hard repair is complete. Continue only to the next truth-surface phase; do not start capability work while the ownership scanner and package replay gates remain open.

Decision: continue
