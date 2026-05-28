# Post-Salvage Validation Final Report

Date: 2026-05-25

## Outcome

The post-`Libraries2` salvage validation pass completed Phases 00 through 09. Fresh package sidecars were generated before implementation edits. `Libraries2` is absent from `/home/sikmindz/Coding`, prior deletion readiness evidence exists, and active downstream dependency repairs were made only where canonical `Libraries` crates were confirmed.

## Changed Files

Primary changes from this run:

- `Libraries/turbo-quant/Cargo.toml`
- `Libraries/codex/validation/*.py`
- `Libraries/docs/post-salvage-validation/**`
- `Recall/Cargo.toml`
- `Recall/deps/llm-pipeline/Cargo.toml`
- `Recall/recall-session/Cargo.toml`
- `Recall/recall-contracts/Cargo.toml`
- `Recall/gui.sh`
- `Recall/scripts/build-release.sh`
- `Recall/scripts/verify_super_workspace.sh`
- `Recall/scripts/codex_preflight_ultimate_fixpack.sh`
- `Recall/scripts/bootstrap.sh`
- `Recall/recall-session/tests/release_truth_tests.rs`
- `Recall/docs/archive/post-salvage-validation/Libraries2-vendor-20260525/**`
- `Recall-Coding/Cargo.toml`
- `Recall-Coding/deps/llm-pipeline/Cargo.toml`
- `Recall-Coding/recall-session/Cargo.toml`
- `Recall-Coding/recall-contracts/Cargo.toml`
- `Recall-Coding/scripts/codex_preflight_ultimate_fixpack.sh`
- `Recall-Coding/scripts/bootstrap.sh`
- `Recall-Coding/scripts/verify_runtime_workspace.sh`
- `Recall-Coding/scripts/verify_source_only_workspace.sh`
- `Recall-Coding/docs/archive/post-salvage-validation/Libraries2-vendor-20260525/**`
- `Gloss/src-tauri/Cargo.toml`
- `Gloss/scripts/assert_existing_crate_boundaries.py`
- `Gloss/src-tauri/vendor/turbo-quant/scripts/inspect_fib_quant_sibling.py`

## Commands Run

Key commands and receipts:

- Fresh sidecars:
  - `python3 package_codex.py --dry-run --no-zip --root /home/sikmindz/Coding/Libraries ...`
  - `python3 package_codex.py --dry-run --no-zip --root /home/sikmindz/Coding --no-check-secrets --max-file-size-mb 2 ...`
- `cargo metadata --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --format-version 1`
- `cargo check --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --workspace`
- `cargo metadata --manifest-path /home/sikmindz/Coding/Recall/Cargo.toml --format-version 1`
- `cargo metadata --manifest-path /home/sikmindz/Coding/Recall-Coding/Cargo.toml --format-version 1`
- `cargo metadata --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --format-version 1`
- `cargo check --manifest-path /home/sikmindz/Coding/Recall/Cargo.toml`
- `cargo check --manifest-path /home/sikmindz/Coding/Recall-Coding/Cargo.toml`
- `cargo check --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --features semantic-memory-backend`
- `npm run build` in `/home/sikmindz/Coding/Gloss`
- `python3 -m compileall claim_ledger tests scripts` in `/home/sikmindz/Coding/ClaimLedger`
- Validation scripts under `Libraries/codex/validation/`

## Tests

Passed:

- `Libraries` workspace cargo metadata and cargo check.
- `Recall` metadata and cargo check after canonical dependency repair.
- `Recall-Coding` metadata and cargo check after canonical dependency repair.
- `Gloss/src-tauri` metadata and cargo check with `semantic-memory-backend`.
- `Gloss` `npm run build`.
- `ClaimLedger` Python compile check.
- Shell syntax checks for modified shell scripts.
- Python compile checks for modified Python scripts.
- Active path dep scan for `Libraries`.
- Duplicate package scan with explicit `semantic-memory` containment allowance.

## Skipped

- Full generated-artifact cleanup was skipped because Phase 07 found 3659 candidates and the pack forbids mass cleanup.
- Full recursive dependency repair across every repo under `/home/sikmindz/Coding` was not attempted; this tranche targeted active downstream apps with confirmed stale `Libraries2` or `_vendor/Libraries` dependency paths.
- `semantic-memory` duplicate rename, merge, or quarantine was skipped pending owner approval.

## Libraries2

`/home/sikmindz/Coding/Libraries2` does not exist. No active manifest dependency remains on `_vendor/Libraries2` or `../Libraries2` in the repaired downstream manifests. Stale vendor trees in `Recall` and `Recall-Coding` were moved into dated archives with a pre-quarantine manifest receipt.

## Duplicate

The duplicate `semantic-memory` package is contained, not resolved. Cargo metadata for canonical `Libraries` includes `Libraries/semantic-memory/Cargo.toml`; `Libraries/turbo-semantic/Cargo.toml` remains outside the active workspace and has no active path dep consumers identified in this pass.

## Path Dep

Active `Libraries` path dependencies are closed. The remaining missing path dependencies reported by the scanner are archived salvage evidence under `_salvage_from_libraries2`, not active workspace dependencies.

## Unresolved

- Decide the final owner action for duplicate `semantic-memory`: rename, merge, or quarantine.
- Decide whether old backup/generated artifacts such as `.orig` files and broad sidecars should be archived in a separate cleanup pass.
- Historical `Libraries2` strings remain in receipts, archived evidence, guard scripts, and generated/backup files; they were intentionally not erased.

## Rollback

Rollback is file-scoped:

- Revert manifest path edits in the changed downstream `Cargo.toml` files if canonical `Libraries` dependency wiring needs to be backed out.
- Move the archived `Libraries2-vendor-20260525` directories back to `_vendor/Libraries2` only if a downstream rollback explicitly requires the stale vendor source.
- Revert `Libraries/turbo-quant/Cargo.toml` nested workspace removal if that crate must again be built as an independent workspace root.
- Remove `Libraries/docs/post-salvage-validation/**` and `Libraries/codex/validation/**` additions if only implementation changes are desired, preserving externally copied receipts first if needed.

## Auditor Handoff

The validation evidence supports a safe post-salvage state for active dependency truth: canonical `Libraries` is cargo-metadata closed, repaired downstream apps build against canonical `Libraries` crates, and unresolved semantic overlap is documented rather than silently collapsed.
