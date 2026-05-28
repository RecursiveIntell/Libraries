# P31A Script Specification

This file defines the intended behavior of the scripts supplied in `scripts/`.

## `assert_release_ledger_schema.py`

Validates `docs/codex-runs/CURRENT_RUN.json`:

- required fields exist;
- booleans are booleans;
- run IDs match `P\d+[A-Z]?`;
- active run differs from last certified run unless explicitly certified;
- positive certification booleans require evidence refs;
- feature expansion is false;
- boundary compiler and runtime receipt changes are deferred.

## `assert_current_run_truth.py`

Replaces stale default behavior. It must read `CURRENT_RUN.json`, not default to P28/P29. It checks `CURRENT_RUN.md` and protected docs for agreement.

## `assert_release_truth_consistency.py`

Checks protected docs:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `docs/codex-runs/CURRENT_RUN.md`

They must cite `CURRENT_RUN.json` and not claim another active/current/certified run.

## `assert_support_claims_have_evidence.py`

Prevents support-label inflation. Any claim of build/package/replay certification must have an evidence ref in the ledger.

## `assert_root_markdown_archive_policy.py`

Must read the ledger active run and classify root Markdown using that run. Stale root P24–P30 docs should fail unless archived/classified. P31 boundary compiler docs must be deferred/inactive, not active current instructions.

## `assert_codex_artifact_classification.py`

Must detect Pxx artifacts across P20–P31A, not only older P20–P22 patterns. It must fail on active unclassified run/Codex/handoff/phase artifacts.

## `assert_package_validation.py`

Must read active run from ledger unless `AIDENS_CURRENT_RUN` is explicitly provided. It validates package sidecars and manifest semantics.

## `assert_package_self_replay.py`

Must extract the package and run the extracted package's `scripts/verify_current.sh`. It must emit a receipt. It must not count source-tree verification as package replay.

## `verify_current.sh`

Must be the single final gate. It must run release truth, classification, invariant, build, static guard, and optionally package gates. It must record blockers honestly.
