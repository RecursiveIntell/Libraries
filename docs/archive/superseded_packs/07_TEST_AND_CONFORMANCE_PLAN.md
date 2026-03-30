
# 07_TEST_AND_CONFORMANCE_PLAN

## Objective

Turn the current finish pass into something a hostile reviewer can replay.

## Layer 1 — root truth

These checks must all be green together:

- `bash scripts/check_pack_truth.sh`
- `python3 scripts/check_root_archive_manifest.py`
- `bash scripts/check_doc_truth.sh`
- `bash scripts/check_repo_surface.sh`

## Layer 2 — gate/receipt convergence

Required property:

> the command set recorded in the proof ledger and receipt must be the same command set the front door actually runs

This is not true today.

Conformance proof:
- one canonical gate list file or generator
- Makefile uses it
- evidence manifest uses it
- receipt generation uses it
- release docs cite it

## Layer 3 — safety/script hygiene

- `bash scripts/check_no_prod_panics.sh`
- `bash scripts/check_hotspot_budgets.sh`
- `python3 scripts/check_public_type_drift.py`
- `python3 scripts/check_public_api_docs.py`

Special note:
The panic audit must stop counting inline test modules under `src/` as production evidence before it can be used honestly.

## Layer 4 — v25 shipped-script closure

- `bash scripts/check_v25_repo_truth.sh`
- `bash scripts/run_v25_local_checks.sh`
- `bash scripts/run_v25_production_pack_checks.sh`
- `python3 scripts/check_v25_production_closure.py`

These must either pass or be retired from the shipped surface.

## Layer 5 — supported cargo lane

Not re-certified here because Cargo was unavailable in this environment, but still required for final closure:

- `cargo fmt $(python3 scripts/print_supported_lane.py --cargo-package-flags) -- --check`
- `cargo clippy $(python3 scripts/print_supported_lane.py --cargo-package-flags) --all-targets --all-features -- -D warnings`
- `cargo test $(python3 scripts/print_supported_lane.py --cargo-package-flags)`
- `bash scripts/check_schema_compat.sh`

## Layer 6 — CI proof

The GitHub workflow must run:
- root truth scripts
- safety/doc/drift scripts
- the supported cargo lane
- the v25 repo-truth/closure scripts that are still part of the shipped story

## Layer 7 — documentation truth

The dashboard, issue matrix, evidence manifest, and receipt must agree on:
- which lane is supported
- which gates are green
- which gates are not currently reproducible
- what remains open

## Minimum hostile re-review bundle

A final closure pass is not done until a hostile reviewer can:
1. clone the repo,
2. run the front door,
3. inspect one CI run,
4. open the receipt,
5. and find the same story in all five places.
