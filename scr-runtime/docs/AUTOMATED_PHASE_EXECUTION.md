# Automated Phase Execution

SCR P0 completion does not use manual phase injections.

The active flow is:

1. `MASTER_AUTOMATED_COMPLETION.md` defines the run.
2. `.codex/prompt_manifest.json` defines phase order.
3. `.codex/prompts/phase_*.md` define phase work.
4. `.codex/auto_gates/phase_*_gate.md` define automatic phase gates.
5. `.codex/tools/auto_phase_runner.py` assembles and optionally executes the phase prompts.

Dry-run proof:

```bash
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
```
