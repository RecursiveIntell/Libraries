# MASTER PROMPT — ClaimLedger P0 Completion with Automated Phase Injection

You are completing the ClaimLedger P0 run. This is a repair-and-completion pass, not P1.

## Required source files to read first

Read, in this order:

1. `AGENTS.md`
2. `AGENTS_COMPLETION_APPEND.md` if present
3. `.codex/prompt_manifest.json`
4. this master prompt

## Current known failure

The previous package is partially successful: the Python ClaimLedger MVP exists, but the active `.codex/` control pack was pruned/archived while tests still require it. Therefore fresh-unzip test execution fails on missing `.codex/hooks.json`, `.codex/config.toml`, `.codex/prompt_manifest.json`, prompts, skills, and phase runner.

## Non-negotiable correction

Manual phase injections are abolished. Do not ask the operator to paste injections between phases.

- phase prompts live under `.codex/prompts/`;
- automated gate/injection prompts live under `.codex/auto_injections/`;
- `.codex/prompt_manifest.json` declares phase order and the auto-injection for each phase;
- `.codex/tools/auto_phase_runner.py` assembles master + phase + auto-injection prompts and runs/prints the sequence;
- CI/release checks must verify the automatic phase system exists and is runnable in dry-run/print mode.

## Mission

Bring ClaimLedger P0 to a clean completion state.

Completion means:

1. Core Python tests pass.
2. Codex control-pack tests pass.
3. `.codex/` active control pack exists in the repo.
4. `.agents/skills/` exists and has valid repo skills.
5. `.codex/skills/` mirror exists if validators expect it.
6. Manual injection files are replaced by automated injection files or retained only as archived historical artifacts outside the active control path.
7. `auto_phase_runner.py` can print/execute the declared phase sequence.
8. Packaging/certifier policy includes active `.codex/` files in next-codex-context/release archives.
9. Generated build artifacts such as `claim_ledger.egg-info/` are removed from source package unless explicitly generated during build.
10. Root `z.py` is moved/renamed to a clear script path, e.g. `scripts/zip_source_certifier.py`, or quarantined from source package if not part of ClaimLedger.
11. A fresh unzip can run all release checks and pass.

## Required execution mode

Use the automated phase runner.

First verify/print the sequence:

```bash
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
```

Then execute phases internally by reading each phase and its auto-injection from `.codex/prompt_manifest.json`.

