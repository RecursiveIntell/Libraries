# ClaimLedger P0 Completion — Automated Phases

Status: PASS

Date: 2026-05-13

Scope:
- Restore active `.codex` control pack and remove manual phase injection dependency.
- Repair automated phase runner/gates.
- Preserve `.codex/` in next-codex-context archive path and validate with fresh-unzip checks.
- Keep all P0 completion and hardening checks executable.

Changed files:
- `.codex/tools/auto_phase_runner.py`
- `.codex/prompt_manifest.json`
- `scripts/run_all_checks.sh`
- `scripts/assert_codex_active_pack.py`
- `scripts/validate_codex_pack.py` (added)
- `scripts/run_completion_checks.sh` (added automated per-phase receipt checks and fresh-unzip invocation)
- `scripts/run_fresh_unzip_checks.sh` (added)
- `tests/test_auto_phase_pack.py`
- `z.py` (updated `.codex` inclusion policy for relevant roles)
- `docs/P0_COMPLETION_CURRENT_STATE.md`
- `docs/P0_COMPLETION_FRESH_UNZIP_CERTIFICATION.md`
- `docs/PASS_CL_P0_COMPLETION_AUTOMATED_PHASES.md`

Command results:

- `python -m pytest -q`
- `bash scripts/run_all_checks.sh`
- `python scripts/validate_codex_pack.py`
- `python scripts/assert_codex_active_pack.py`
- `python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json`
- `bash scripts/run_fresh_unzip_checks.sh`

Phase system proof:
- `.codex/prompt_manifest.json` declares `manual_injections_required: false` and `auto_injections_required: true`.
- `.codex/tools/auto_phase_runner.py` assembles prompt + gate and writes receipts.
- Per-phase dry-run receipts are emitted under `.codex/runs/P0-completion/`.
- `tests/test_auto_phase_pack.py` validates manifest shape, per-phase receipts, and archive inclusion.

Unresolved risks:
- Archive and fresh-unzip commands may be resource-heavy; rerun required checks in cold CI environments if timeouts appear.
- Any future edits to archive policy should re-run the fresh-unzip certification script.

Next recommended pass:
- Continue with the P1 plan only after this PASS is reviewed by hostile audit tooling.
