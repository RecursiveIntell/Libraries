# Phase 01 Receipt - Libraries Canonical Closure

Date: 2026-05-25

## Commands

- `python3 codex/validation/scan_path_deps.py --root /home/sikmindz/Coding/Libraries --json-out docs/post-salvage-validation/receipts/phase01_path_deps.json`
- `python3 codex/validation/scan_duplicate_packages.py --root /home/sikmindz/Coding/Libraries --json-out docs/post-salvage-validation/receipts/phase01_duplicate_packages.json`
- `cargo metadata --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --format-version 1 > docs/post-salvage-validation/receipts/phase01_cargo_metadata.json`
- `cargo check --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --workspace > docs/post-salvage-validation/receipts/phase01_cargo_check_workspace.log 2>&1`
- `python3 codex/validation/scan_duplicate_packages.py --root /home/sikmindz/Coding/Libraries --allow semantic-memory --json-out docs/post-salvage-validation/receipts/phase01_duplicate_packages_allow_contained.json`

## Results

- Active Cargo path dependencies: pass. The structural TOML scan reports 0 missing active path dependencies.
- Archived salvage Cargo path dependencies: 26 missing paths under `_salvage_from_libraries2/Libraries2`; classified as archived salvage evidence, not active implementation truth.
- `cargo metadata`: pass after repairing `turbo-quant/Cargo.toml`, which incorrectly declared a nested `[workspace]` while also being a parent workspace member.
- `cargo check --workspace`: pass. Log: `phase01_cargo_check_workspace.log`.
- Duplicate package scan: one active duplicate without allowlist: `semantic-memory` in `semantic-memory/Cargo.toml` and `turbo-semantic/Cargo.toml`.

## Semantic-Memory Closure Status

Canonical package identity is `Libraries/semantic-memory/Cargo.toml`.

Containment evidence:

- Cargo metadata includes one `semantic-memory` package: `/home/sikmindz/Coding/Libraries/semantic-memory/Cargo.toml`.
- `turbo-semantic` is not a `Libraries` workspace member and was not found as a path dependency in `Libraries`, `Recall`, `Recall-Coding`, or `Gloss` manifests.
- The duplicate scan passes only with explicit `--allow semantic-memory`; this is a containment waiver, not a claim that the collision is resolved.

Phase 05 must keep this as a boundary issue unless Josh approves a rename, merge, or physical quarantine.

## Changed Files In This Phase

- `turbo-quant/Cargo.toml`: removed nested `[workspace]` declaration so parent `Libraries` workspace metadata/check can resolve.
- `codex/validation/scan_path_deps.py`: switched path dependency detection from regex to TOML parsing so comments are not treated as dependencies.
- `codex/validation/scan_duplicate_packages.py`: reports active duplicates separately from archived salvage duplicates and supports explicit allowlisting.

## Gate

`Libraries` active Cargo closure passes. Semantic-memory duplicate is contained for workspace/build purposes but remains unresolved as a semantic collision for Phase 05.
