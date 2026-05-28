# Phase 01 — z.py Total Closure

## Goal
Finish `z.py` as a generic, future-proof source certifier. This phase must prevent P22-specific behavior from surviving into P23.

## Required fixes

1. Replace hard-coded P22 current-run allowlists with generic current-run derivation.
   - `--codex-current-run P23` must permit `docs/p23/**`, `prompts/p23/**`, `tasks/p23/**`, `handoffs/p23/**`, `phase_injections/**` only when classified as current-run control.
   - P24/P25 must work without editing code.
2. Add package modes or equivalent policy split:
   - `release-context`: clean source/release package, no Codex control docs, no archived history.
   - `next-codex-context`: source + minimal current handoff needed for next run, no archived history.
   - `codex-run-full`: current run control/evidence allowed, no archived history.
   - `audit-full`: deliberate full history.
   If renaming modes is too disruptive, implement exact equivalents and document them in `z.py --help` and tests.
3. Make strict script-reference checking capable of catching missing/excluded verifier dependencies.
   - If a script is included and references another local script, that target must exist and be included, unless an explicit source-repo-only marker is present.
4. Rename or allowlist intentional redaction/fixture files without weakening content secret scanning.
5. Ensure no normal package excludes a dependency of an included verifier.
6. Remove legacy `zip.py` or convert it into a hard-failing wrapper that points to `z.py`.
7. Emit package role metadata in manifest/report.

## Required tests

- `scripts/assert_zpy_total_contract.py`
- `scripts/assert_script_refs_strict.py`
- `scripts/assert_no_legacy_zip.py`
- `python3 z.py --help` contains the new package-role semantics.
- `python3 z.py --mode release-context --strict --dry-run` or equivalent.
- `python3 z.py --mode next-codex-context --strict --dry-run` or equivalent.
- `python3 z.py --mode audit-full --include-codex-archive --strict --dry-run`.

## Acceptance gate

P23 cannot proceed until `z.py` can package with `--codex-current-run P23` and does not contain P22-specific logic except in archived historical text.
