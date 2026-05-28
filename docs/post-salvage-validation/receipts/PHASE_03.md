# Phase 03 Receipt - Residual Libraries2 References

Date: 2026-05-25

## Active Manifest Repairs

Rewrote active downstream manifests from stale `_vendor/Libraries2` crates to canonical `Libraries` crates:

- `Recall/Cargo.toml`
  - `agent-graph`: `_vendor/Libraries2/agent-graph` -> `../Libraries/agent-graph`
  - `job-queue`: `_vendor/Libraries2/job-queue` -> `../Libraries/job-queue`
- `Recall-Coding/Cargo.toml`
  - `agent-graph`: `_vendor/Libraries2/agent-graph` -> `../Libraries/agent-graph`
  - `job-queue`: `_vendor/Libraries2/job-queue` -> `../Libraries/job-queue`

Canonical targets exist and were validated in Phase 01.

## Active Script Repairs

- `Recall/gui.sh`: removed `../Libraries2` sibling requirements.
- `Recall/scripts/build-release.sh`: vendors `agent-graph` and `job-queue` from `../Libraries` into `_vendor/Libraries`; removed `Libraries2` vendoring and rewrite logic.
- `Recall/scripts/verify_super_workspace.sh`: checks `../Libraries/agent-graph` and `../Libraries/job-queue`.
- `Recall/scripts/codex_preflight_ultimate_fixpack.sh`: full workspace mode depends on `../Libraries` only.
- `Recall/scripts/bootstrap.sh`: removed stale `Libraries2` fallback wording.
- `Recall/recall-session/tests/release_truth_tests.rs`: release truth test now checks `../Libraries` only.
- `Recall-Coding/scripts/codex_preflight_ultimate_fixpack.sh`: full workspace mode depends on `../Libraries` only.
- `Recall-Coding/scripts/bootstrap.sh`: removed stale `Libraries2` fallback wording.
- `Recall-Coding/scripts/verify_runtime_workspace.sh`: checks canonical sibling `../Libraries/agent-graph` and `../Libraries/job-queue`.
- `Recall-Coding/scripts/verify_source_only_workspace.sh`: checks canonical sibling `../Libraries/agent-graph` and `../Libraries/job-queue`.
- `Gloss/scripts/assert_existing_crate_boundaries.py`: scans canonical `Libraries` only.
- `Gloss/src-tauri/vendor/turbo-quant/scripts/inspect_fib_quant_sibling.py`: removed `Libraries2/fib-quant` candidates.

## Quarantine Movement

Moved stale vendor copies out of active `_vendor/Libraries2` paths:

- `Recall/_vendor/Libraries2` -> `Recall/docs/archive/post-salvage-validation/Libraries2-vendor-20260525`
- `Recall-Coding/_vendor/Libraries2` -> `Recall-Coding/docs/archive/post-salvage-validation/Libraries2-vendor-20260525`

Manifest:

- `docs/post-salvage-validation/receipts/phase03_vendor_libraries2_pre_quarantine_manifest.json`

Counts:

- Recall stale vendor files: 73
- Recall-Coding stale vendor files: 73

## Scan Results

- Active targeted `Cargo.toml` refs to `Libraries2`: 0.
- Active `_vendor/Libraries2` directories under Recall/Recall-Coding: 0.
- Remaining `Libraries2` strings are in docs/history, `docs-archive`, `.c.bak`, validation guards, changelog provenance, or backup/generated files such as `Cargo.toml.orig`; Phase 07 handles generated/backup artifact hygiene.

## Gate

Phase 03 passes for active manifests and active vendor paths. Remaining references are historical/archival/guard references and are not implementation dependency truth.
