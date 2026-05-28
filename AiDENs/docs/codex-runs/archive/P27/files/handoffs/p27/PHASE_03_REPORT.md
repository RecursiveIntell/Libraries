# P27 Phase Report

## Phase

- Phase ID: 03
- Phase title: Package self-replay and sibling-layout classification
- Date: 2026-05-04T23:00:51Z

## Scope

- Intended work: attempt package self-replay and classify replay status honestly, including sibling-layout prerequisites if encountered.
- Issue IDs in scope: `P27-002`.
- Explicit non-goals: no capability work, no ownership scanner repair, no root Markdown cleanup, no verifier redesign, no vendoring of sibling crates, no support-claim widening.

## Files inspected

- `prompts/phases/P27_PHASE_03_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_03_BEFORE_PHASE_04.md`
- `handoffs/p27/PHASE_02_REPORT.md`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_package_validation.py`
- `z.py`
- `P27_COMMANDS.md`
- `STATUS.md`
- `Cargo.toml`
- `target/p27/package/AiDENs-p27-codex-context.report.md`
- `target/p27/audit/phase03_package_self_replay_receipt_final.json`

## Files changed

- `scripts/assert_package_self_replay.py`
- `scripts/assert_package_validation.py`
- `P27_COMMANDS.md`
- `STATUS.md`
- `handoffs/p27/PHASE_03_REPORT.md`

## Changes made

- Updated `scripts/assert_package_self_replay.py` to accept the P27 documented `--package` interface while preserving positional compatibility.
- Added a local operator replay receipt JSON with `artifact_kind`, `support_tier`, `semantic_status`, `replay_status`, `classification`, verifier output tails, and known limits.
- Added explicit replay classifications including `package_missing`, `verifier_missing`, `sibling_workspace_missing`, `verifier_failed`, and `verifier_failed_p27_004`.
- Updated `scripts/assert_package_validation.py` default current run from P26 to P27.
- Updated `P27_COMMANDS.md` so the package command writes the exact `target/p27/package/AiDENs-p27-codex-context.zip` path used by self-replay.
- Updated `STATUS.md` to record that package replay was attempted and classified as blocked by `P27-004`, not passed.

## Commands run

| Command | Result | Log |
|---|---|---|
| `python3 -m py_compile scripts/assert_package_self_replay.py scripts/assert_package_validation.py` | pass | `target/p27/audit/phase03_py_compile_after.log` |
| missing-package replay probe | classified `package_missing` | `target/p27/audit/phase03_missing_package_self_replay.log`, `target/p27/audit/phase03_missing_package_replay_receipt.json` |
| sibling layout probe | sampled required sibling crates present in local parent workspace | `target/p27/audit/phase03_sibling_layout_present.log` |
| `python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P27 --output target/p27/package/AiDENs-p27-codex-context.zip` | pass; package written | `target/p27/audit/phase03_zpy_package_after_patch.log` |
| `python3 scripts/assert_package_validation.py` | pass | `target/p27/audit/phase03_package_validation_final.log` |
| `python3 scripts/assert_package_self_replay.py --package target/p27/package/AiDENs-p27-codex-context.zip --verifier scripts/verify_current.sh --receipt-out target/p27/audit/phase03_package_self_replay_receipt_final.json` | expected fail, classified `verifier_failed_p27_004` | `target/p27/audit/phase03_package_self_replay_final.log`, `target/p27/audit/phase03_package_self_replay_receipt_final.json` |
| package report fact extraction | pass | `target/p27/audit/phase03_package_summary_facts.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase03_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_verifier_surface.py .` | pass | `target/p27/audit/phase03_assert_p27_verifier_surface.log` |
| `python3 scripts/assert_script_refs_strict.py .` | pass | `target/p27/audit/phase03_assert_script_refs_strict.log` |

## Evidence emitted

- `target/p27/package/AiDENs-p27-codex-context.zip`
- `target/p27/package/AiDENs-p27-codex-context.manifest.json`
- `target/p27/package/AiDENs-p27-codex-context.report.md`
- `target/p27/package/AiDENs-p27-codex-context.findings.json`
- `target/p27/package/AiDENs-p27-codex-context.excluded.json`
- `target/p27/package/AiDENs-p27-codex-context.codex-archive.json`
- `target/p27/audit/phase03_zpy_package_after_patch.log`
- `target/p27/audit/phase03_package_validation_final.log`
- `target/p27/audit/phase03_package_self_replay_final.log`
- `target/p27/audit/phase03_package_self_replay_receipt_final.json`
- `target/p27/audit/phase03_package_summary_facts.log`
- `target/p27/audit/phase03_sibling_layout_present.log`

## Replay classification

- Package strict validation: pass.
- Package findings: `0` errors, `0` warnings.
- Package archive SHA-256: `2199ee4db2c89abd440a1b9a61cb21e1d00aed79c0bf985b85cfaea7fa5927eb`.
- External Cargo path dependency roots included: `40`.
- Replay status: `failed`.
- Replay classification: `verifier_failed_p27_004`.
- Replay blocker: extracted package reaches `scripts/verify_current.sh`, passes verifier surface, strict script references, current-run truth, and AGENTS checks, then fails at the ownership scanner fail-closed guard.

## 11A semantic impact

- Exact/approx labels touched: added `semantic_status: exact_check` to package replay receipts.
- Proof/check hooks added: package replay receipts now record verifier command outcome and classification.
- Degradation/support labels changed: added `support_tier: verification` and `known_limits` to replay receipts.

The replay receipt is AiDENs-local operator evidence. It does not promote package replay failure or sibling layout facts into canonical truth.

## Support profile impact

- No support-tier claim changed.
- Package self-replay is not green; it is honestly classified as verifier failure caused by `P27-004`.

## Issues closed

- `P27-002` false-green risk is closed by explicit replay attempt and classification. Package replay remains failed until `P27-004` is fixed and replay is rerun.

## New issues / risks

- `P27-004` blocks both local verifier and package self-replay.
- Root Markdown archive hygiene remains out of scope for Phase 03 and is still visible in package report metadata.

## Decision

Rationale: Phase 03 produced a strict-clean P27 package and an explicit self-replay receipt. Replay is not green, but it is no longer ambiguous: it fails on the known ownership scanner guard that Phase 04 is scoped to repair.

Decision: continue
