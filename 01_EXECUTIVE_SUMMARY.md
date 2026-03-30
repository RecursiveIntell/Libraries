
# 01_EXECUTIVE_SUMMARY

## Verdict

**Serious core, stale release storytelling, and an unfinished production-closure lane.**

The repo is not vapor. The core stack is real: `semantic-memory`, `forge-pilot`, `living-memory/living-memory`, `knowledge-runtime`, `semantic-memory-forge`, `forge-memory-bridge`, and the kernel crates all contain substantial logic and real tests.

What is still broken is the **front door and truth surface** around that core.

## What changed since the older hostile audit

Several older criticisms are now stale:

- `forge-pilot` is no longer “7 tests and path normalization only.”
- tracked public-function rustdoc coverage is no longer near-zero.
- some earlier zero-test / zero-doc claims now materially understate the current snapshot.

But several structural criticisms still hold:

- the front door still fails immediately,
- status surfaces overclaim green,
- the thin governance/runtime shells still create credibility drag,
- oversized modules still dominate review difficulty,
- the `llm-refinement` feature is still effectively narrative,
- and the line-based Rust symbol extractor is still fragile.

## What held up well

- The active workspace has **30 members** (29 default members) and **67,083 non-test Rust LOC** across those members (**103,987 including tests**) with **1,049 test functions** found by static scan.
- The larger repo surface (workspace members + excluded satellites) has **96,641 non-test Rust LOC** (**142,804 including tests**) and **1,644 tests**.
- The older hostile claim that `forge-pilot` had only path-normalization tests is stale. The current snapshot has 54 forge-pilot tests covering observation, scoring, loop halts, verification control, execution-evidence lineage, repo chat, bootstrap, and roundtrip paths.
- `forge-pilot/src/main_support/mod.rs` (1592 LOC) and `forge-pilot/src/loop_runner.rs` (1034 LOC) are still oversized and remain the sharpest review/maintenance seam in the agent lane.

## What failed immediately

- `bash scripts/check_pack_truth.sh` fails.
- `python3 scripts/check_root_archive_manifest.py` fails.
- `bash scripts/check_no_prod_panics.sh` fails because the script scans inline test modules under `src/` and the receipt still claims this gate is green.
- `bash scripts/check_v25_repo_truth.sh`, `bash scripts/run_v25_local_checks.sh`, and `bash scripts/run_v25_production_pack_checks.sh` fail because shipped v25 surfaces are missing files.
- `python3 scripts/check_v25_production_closure.py` fails because the effect/policy/control v25 closure is not complete.
- `.github/workflows/ci.yml` is missing.

## Current audit snapshot

| command | status | detail |
|---|---|---|
| bash scripts/check_pack_truth.sh | fail |  |
| python3 scripts/check_root_archive_manifest.py | fail | group legacy_root_residue archived_dir file count 29 != 30 / root archive manifest check failed |
| bash scripts/check_doc_truth.sh | pass | doc truth check passed |
| python3 scripts/check_current_closeout_lane.py | pass | current closeout lane ok: 2026-03-22-hardening-closeout supported crates= 17 |
| bash scripts/check_repo_surface.sh | pass | repo surface check passed |
| python3 scripts/check_public_api_docs.py | pass | public api doc coverage report / forge-pilot: documented 59/59 public functions / kernel-conformance: documented 20/20 public functions / llm-tool-runtime: documented 31/31 public  |
| python3 scripts/check_public_type_drift.py | pass | public type drift report / V25ConstitutionCitation: federated-settlement, verification-control / public type drift check passed with 1 allowlisted duplicate name(s) |
| bash scripts/check_no_prod_panics.sh | fail | supported-lane panic audit failed: forge-memory-bridge/src/transform_tests.rs:22 / supported-lane panic audit failed: forge-memory-bridge/src/transform_tests.rs:53 / supported-lane |
| bash scripts/check_hotspot_budgets.sh | pass | hotspot budget checks passed |
| python3 scripts/check_v25_production_closure.py | fail | v25 production closure check failed: / - effect-runtime/src/effect.rs missing marker: ApplicabilityContextId / - effect-runtime/src/effect.rs missing marker: ProfileSetId / - effec |
| python3 scripts/check_v25_json_surface.py | pass | v25 JSON surface checks passed |
| bash scripts/check_schema_registry_uniqueness.sh | pass | schema registry uniqueness checks passed |
| bash scripts/check_schema_compat.sh | fail | schema compatibility check failed: cargo not available |
| bash scripts/check_v25_repo_truth.sh | fail | missing required file: 24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md |
| bash scripts/run_v25_local_checks.sh | fail | missing required file: 24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md |
| bash scripts/run_v25_production_pack_checks.sh | fail | missing production pack file: docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.csv |

## My opinion

This repo is worth finishing.

But it needs one non-negotiable posture correction:

> **Stop letting the dashboard, receipt, and shipped scripts sound more finished than the repo can currently prove.**

Close the release-truth seam first. Then close the v25/CI surface. Then do the credibility cleanup.
