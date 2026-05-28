# Phase 08 - Regression And Receipt Validation

Date: 2026-05-25

## Scope

Phase 08 reran the post-repair gates against the canonical `Libraries` workspace and the downstream apps modified during this validation tranche.

## Commands Run

```bash
python3 codex/validation/scan_path_deps.py --root /home/sikmindz/Coding/Libraries --json-out docs/post-salvage-validation/receipts/phase08_path_deps.json
python3 codex/validation/scan_duplicate_packages.py --root /home/sikmindz/Coding/Libraries --allow semantic-memory --json-out docs/post-salvage-validation/receipts/phase08_duplicate_packages.json
cargo metadata --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --format-version 1
npm run build
python3 -m compileall claim_ledger tests scripts
```

Earlier repair-phase checks retained as Phase 08 evidence:

```bash
cargo check --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --workspace
cargo check --manifest-path /home/sikmindz/Coding/Recall/Cargo.toml
cargo check --manifest-path /home/sikmindz/Coding/Recall-Coding/Cargo.toml
cargo check --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --features semantic-memory-backend
```

## Results

- `Libraries` active path dependencies: pass. `phase08_path_deps.log` reports `active_missing=0`; the remaining 26 missing paths are archived salvage evidence under `_salvage_from_libraries2`.
- Duplicate package gate: pass with explicit containment allowance for `semantic-memory`. `phase08_duplicate_packages.log` reports no active duplicate package failure.
- `Libraries` cargo metadata: pass. JSON receipt: `phase08_libraries_metadata.json`; stderr receipt is empty.
- `Gloss` frontend build: pass. Receipt: `phase08_gloss_npm_build.log`.
- `ClaimLedger` Python compile check: pass. Receipt: `phase08_claimledger_compileall.log`.
- `Libraries`, `Recall`, `Recall-Coding`, and `Gloss/src-tauri` cargo checks: pass in Phase 01/04 receipts and retained as regression evidence.

## Skipped

- No full broad cleanup was run. Phase 07 found thousands of generated artifact candidates, and the validation pack forbids mass cleanup.
- No duplicate `semantic-memory` rename, merge, or deletion was attempted. Phase 05 records that as an unresolved boundary requiring owner approval.

## Conclusion

Phase 08 gates support the conclusion that canonical `Libraries` is cargo-metadata closed, active repaired downstream manifests resolve, and the remaining salvage warnings are archived evidence rather than active dependency truth.
