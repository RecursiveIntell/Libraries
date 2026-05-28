# Phase 00 — Current-State Failure Proof

Goal: prove the current state before repairing it. Do not edit until evidence is collected.

Run:

```bash
git status --short || true
find .codex -maxdepth 3 -type f | sort | sed -n '1,240p'
python -m pytest -q || true
bash scripts/run_all_checks.sh || true
python scripts/validate_codex_pack.py || true
```

Record:
- whether `.codex/` exists;
- whether `.codex/prompts/MASTER_AUTOMATED_COMPLETION.md` exists;
- whether `.codex/prompt_manifest.json` exists;
- whether `.codex/hooks.json` and `.codex/config.toml` exist;
- whether `.codex/tools/auto_phase_runner.py` exists;
- whether `claim_ledger.egg-info/` is present;
- whether root `z.py` is present;
- exact failing tests.

Write `docs/P0_COMPLETION_CURRENT_STATE.md`.
