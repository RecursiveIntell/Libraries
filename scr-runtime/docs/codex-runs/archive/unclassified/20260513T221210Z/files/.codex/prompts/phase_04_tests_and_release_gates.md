# Phase 04 — Tests and Release Gates

Goal: make the active repo pass all P0 completion checks.

Required commands:

```bash
python -m pytest -q
bash scripts/run_all_checks.sh
python scripts/validate_codex_pack.py
python scripts/assert_codex_active_pack.py
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
```

If any fail, fix the cause. Do not weaken tests to pass. Do not delete Codex pack tests.

Required test additions or updates:
- active `.codex/` exists;
- `prompt_manifest.json` references all phase and auto-injection files;
- `auto_phase_runner.py --dry-run` emits receipt;
- no active manual injection requirement remains;
- archive include assertion exists.
