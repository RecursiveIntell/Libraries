
# 02_SOURCE_BASIS

## Snapshot basis

- archive basis: `libraries-source-clean-20260323.zip`
- unpacked for audit into an isolated working directory
- audit date: 2026-03-24
- audit mode: static source inspection + shell/python repo checks
- no local `cargo` / `rustc` were available in this environment

## What I directly inspected

### Workspace topology

- workspace members: **30**
- default members: **29**
- excluded crates/paths: **19**

### Rust surface

- workspace member Rust LOC (including tests): **103,987**
- workspace member non-test Rust LOC: **67,083**
- workspace member tests found by static scan: **1,049**
- full repo Rust LOC (workspace + excluded crates, including tests): **142,804**
- full repo non-test Rust LOC: **96,641**
- full repo tests found by static scan: **1,644**

### Docs/config surface

- markdown files: **481**
- markdown lines: **53,485**
- json files: **1,753**
- json lines: **54,111**
- top-level root files: **54**

## Core crates by size

| crate | prod_loc | tests | pub_fn | doc_comments |
|---|---|---|---|---|
| semantic-memory | 17505 | 303 | 208 | 847 |
| forge-pilot | 9772 | 54 | 92 | 59 |
| living-memory/living-memory | 9126 | 181 | 138 | 645 |
| knowledge-runtime | 5665 | 145 | 75 | 692 |
| semantic-memory-forge | 3930 | 46 | 34 | 302 |
| forge-memory-bridge | 1723 | 44 | 10 | 271 |
| kernel-conformance | 1376 | 47 | 20 | 20 |
| kernel-execution | 1217 | 10 | 7 | 7 |
| kernel-oracles | 1007 | 12 | 8 | 8 |
| llm-tool-runtime | 2454 | 11 | 32 | 31 |
| stack-ids | 2309 | 56 | 37 | 416 |

## Thin governance/runtime shells

| crate | prod_loc | pub_fn | impls | doc_comments | tests |
|---|---|---|---|---|---|
| assurance-runtime | 263 | 0 | 0 | 0 | 12 |
| attestation-exchange | 121 | 0 | 0 | 0 | 5 |
| authority-delegation | 215 | 0 | 0 | 0 | 7 |
| constitutional-memory | 246 | 2 | 0 | 0 | 4 |
| continuity-runtime | 216 | 0 | 0 | 0 | 7 |
| discovery-portfolio | 267 | 1 | 0 | 0 | 4 |
| effect-runtime | 234 | 1 | 1 | 1 | 3 |
| federated-settlement | 422 | 3 | 1 | 0 | 7 |
| mechanism-runtime | 206 | 1 | 0 | 0 | 4 |
| remote-oracle-admission | 141 | 0 | 1 | 0 | 1 |
| spec-execution | 378 | 3 | 0 | 0 | 4 |
| verification-calibration | 121 | 2 | 2 | 0 | 2 |

## Largest production files

| file | loc |
|---|---|
| profile-runtime/src/adapters.rs | 1776 |
| living-memory/living-memory/src/lab/evidence.rs | 1617 |
| semantic-memory/src/db.rs | 1609 |
| semantic-memory/src/lib.rs | 1600 |
| forge-pilot/src/main_support/mod.rs | 1592 |
| semantic-memory/src/search.rs | 1583 |
| semantic-memory/src/projection_lane.rs | 1471 |
| living-memory/living-memory/src/store/db.rs | 1400 |
| semantic-memory-forge/src/envelope.rs | 1394 |
| stack-ids/src/ids.rs | 1222 |
| kernel-execution/src/lib.rs | 1217 |
| verification-control/src/lib.rs | 1209 |

## Repo checks run during this audit

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

## Audit limitations

### What I could certify
- filesystem truth
- script truth for shell/python-only checks
- dependency/layout truth
- static source metrics
- presence/absence of files, modules, duplicated types, and specific feature paths

### What I could not certify here
- `cargo test`
- `cargo fmt`
- `cargo clippy`
- schema generation/build checks requiring Cargo
- runtime behavior requiring successful compilation

Where the existing repo receipt claims cargo-backed green, I treated that as **historical repo evidence**, not as something re-certified in this environment.
