# AGENTS Completion Appendix — Automated Phase Governance

This appendix overrides any older manual-injection workflow for the SCR P0 completion pass.

## Automated phase law

Manual phase injection prompts are no longer active workflow artifacts.

Active phase control must use:

- `.codex/prompt_manifest.json`
- `.codex/prompts/MASTER_AUTOMATED_COMPLETION.md`
- `.codex/prompts/phase_*.md`
- `.codex/auto_gates/phase_*_gate.md`
- `.codex/tools/auto_phase_runner.py`

The runner or Codex itself must load the phase gate automatically after each phase. The operator must not be required to paste phase injections.

## Completion law

The run is incomplete unless a fresh checkout/unzip can pass:

```bash
python -m pytest -q
bash scripts/run_all_checks.sh
python scripts/validate_codex_pack.py
python scripts/assert_codex_active_pack.py
bash scripts/run_completion_checks.sh
```

If an archive is produced, it must preserve active `.codex/` control files.

## Scope

This pass repairs and completes P0 only. Do not implement P1 features.
