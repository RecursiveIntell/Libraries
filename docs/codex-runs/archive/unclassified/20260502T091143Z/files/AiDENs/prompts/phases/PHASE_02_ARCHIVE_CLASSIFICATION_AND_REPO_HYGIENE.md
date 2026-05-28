# Phase 02 — Archive Classification and Repo Hygiene

## Goal
Replace narrow stale-run detection with explicit classification for all Codex-run and historical-run artifacts.

## Required actions

1. Create `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json` or equivalent.
2. Classify every Pxx/Pyy-marked artifact outside the archive as one of:
   - `current_instruction`
   - `active_regression_fixture`
   - `active_test_fixture`
   - `active_support_matrix`
   - `archived_execution_evidence`
   - `deprecated_template`
   - `quarantined_legacy`
3. Archive or quarantine unclassified stale artifacts.
4. Move old audit and template material out of active instruction scope unless explicitly classified as active regression fixture.
5. Preserve evidence through manifests and supersession records.
6. Update `z.py` to use the classification registry when deciding active-vs-stale.

## Required tests

- `scripts/assert_codex_artifact_classification.py .`
- `python3 z.py --verify-codex-archive-hygiene --codex-current-run P23 --strict --dry-run` or equivalent.
- Normal packages must have zero unclassified Pxx/Pyy active artifacts.

## Acceptance gate

No file with `P20`, `P21`, `P22`, `CODEX`, `codex`, `phase`, `handoff`, or old run markers may remain active without machine-readable classification.
