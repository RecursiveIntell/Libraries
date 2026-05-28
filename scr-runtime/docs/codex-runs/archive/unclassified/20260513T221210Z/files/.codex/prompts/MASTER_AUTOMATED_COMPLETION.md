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

Instead:

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
5. `.codex/skills/` mirror exists if tests/docs expect it.
6. Manual injection files are replaced by automated injection files or retained only as archived historical artifacts outside the active control path.
7. `auto_phase_runner.py` can print/execute the declared phase sequence.
8. Packaging/certifier policy includes active `.codex/` files in next-codex-context/release archives.
9. Generated build metadata such as `claim_ledger.egg-info/` is removed from source control/package manifests unless explicitly generated during build.
10. Root `z.py` is moved/renamed to a clear script path, e.g. `scripts/zip_source_certifier.py`, or quarantined from source package if not part of ClaimLedger.
11. A fresh unzip can run all release checks and pass.

## Absolute non-goals

Do not implement P1.
Do not add SQLite.
Do not add embeddings.
Do not add live LLM extraction.
Do not integrate AiDENs or semantic-memory.
Do not claim v11 compliance.
Do not claim completion without receipts and command evidence.

## Required commands before edits

Run and record:

```bash
git status --short || true
find . -maxdepth 3 -type f | sort | sed -n '1,220p'
python -m pytest -q || true
bash scripts/run_all_checks.sh || true
python scripts/validate_codex_pack.py || true
```

## Required execution mode

Use the automated phase runner.

First verify/print the sequence:

```bash
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
```

Then execute phases internally by reading each phase and its auto-injection from `.codex/prompt_manifest.json`. If the Codex CLI is available and the operator explicitly runs this outside the current session, `auto_phase_runner.py` may use `codex exec`. Inside an interactive Codex session, you must still follow the same manifest order automatically without asking for manual injection pastes.

## Phase order

Use `.codex/prompt_manifest.json` as source of truth. Expected phases:

0. current-state failure proof
1. restore active Codex pack
2. implement automated phase runner
3. fix packaging policy
4. tests and release gates
5. fresh-unzip certification
6. hostile auditor handoff

Each phase must consume its matching `.codex/auto_injections/phase_XX_gate.md` automatically.

## Required final gates

All must pass:

```bash
python -m pytest -q
bash scripts/run_all_checks.sh
python scripts/validate_codex_pack.py
python scripts/assert_codex_active_pack.py
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
bash scripts/run_completion_checks.sh
```

If an archive is produced, also run:

```bash
python scripts/assert_archive_includes_codex.py <path-to-archive.zip>
```

## Required final report

Write `docs/PASS_CL_P0_COMPLETION_AUTOMATED_PHASES.md` with:

- status: PASS / PARTIAL / FAIL;
- exact changed files;
- commands run;
- command results;
- fresh-unzip result;
- archive-includes-codex result if archive produced;
- unresolved risks;
- explicit statement that phase injections are automated;
- exact next recommended pass.

Do not end with a narrative-only success claim. Receipts or it did not happen.
