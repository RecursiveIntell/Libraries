# Libraries2 Final Salvage Receipt

Date: 2026-05-25
Branch: `salvage/libraries2-20260525`
Source root: `/home/sikmindz/Coding/Libraries2`
Destination root: `/home/sikmindz/Coding/Libraries`

## Copy And Hash Receipts

- Full source tree copied to `_salvage_from_libraries2/Libraries2/` preserving original paths.
- Non-target SHA-256 source/destination manifest: `docs/salvage/libraries2_sha256_manifest.csv`.
- Manifest rows: 1,733; mismatches: 0.
- Symlink manifest: `docs/salvage/libraries2_symlink_manifest.csv`.
- Promoted crate SHA-256 manifest: `docs/salvage/promoted_crate_sha256_manifest.csv`.

## Ledger Classification

Every row from `10_LEDGER/LIBRARIES2_SALVAGE_PLAN.csv` is classified in `docs/salvage/libraries2_ledger_classification.csv`.

Promoted Rust crates, with canonical dependency paths repaired where needed:

- `ai-batch-queue`
- `agent-graph`
- `comfyui-rs`
- `job-queue`
- `llm-output-parser`
- `llm-pipeline`
- `ollama-vision`
- `tauri-queue`

Promoted non-Rust workstream:

- `tauri-react-hooks`

Archived-only rows:

- `Libraries2/Cargo.toml` workspace root: not activated because the source workspace has stale/missing members and absolute symlinks.
- `demo-tauri-libraries`: archived as a demo app, not promoted as canonical library code.
- `repo_overlay/*` and `scaffolds/*`: archived as historical reference; no canonical same-name overwrite performed.

## Validation Summary

Final validation summary: `docs/salvage/final_validation_summary.json`.

Rust receipts:

- `llm-output-parser`: `cargo check` pass, `cargo test` pass.
- `job-queue`: `cargo check` pass, `cargo test` pass.
- `agent-graph`: `cargo check` pass, `cargo test` pass.
- `comfyui-rs`: `cargo check` pass, `cargo test` pass.
- `ollama-vision`: stale `.parser-lib` path repaired to `../llm-output-parser`; `cargo check` pass, `cargo test` pass.
- `ai-batch-queue`: `cargo check` pass, `cargo test` pass.
- `tauri-queue`: `cargo check` pass, `cargo test` pass.
- `llm-pipeline`: canonical `llm-tool-runtime` API drift repaired from `ExecutionPermit` to `ToolExecutionPermit`; stale internal test calls updated for explicit timeout; `cargo check` pass, `cargo test` pass.

TypeScript receipt:

- `tauri-react-hooks`: `npm ci` pass, `npm run typecheck` pass, `npm run build` pass.

Build logs are under `docs/salvage/build_logs/`.

## Same-Name Crates

Same-name primary crates were not overwritten. Path-level review found no source-only files in `Libraries2/<crate>` for these crates; all source paths overlapped canonical paths and canonical had additional files. Diffs are archived for review:

- `docs/salvage/diff_attestation-exchange.patch`
- `docs/salvage/diff_constraint-compiler.patch`
- `docs/salvage/diff_discovery-portfolio.patch`
- `docs/salvage/diff_federated-settlement.patch`
- `docs/salvage/diff_profile-runtime.patch`
- `docs/salvage/diff_remote-oracle-admission.patch`
- `docs/salvage/diff_spec-execution.patch`

## Unique Fixtures And Docs

No-overwrite fixture/doc copies are listed in `docs/salvage/unique_fixture_doc_copies.csv`.

Copied into canonical paths:

- Six unique `contracts/fixtures/v18/*.json` files.
- The missing `conformance/` fixture/doc tree.

Overlay schemas/examples with matching canonical filenames were not overwritten.

## Reference Repair

Live scripts/prompts that required or scanned `/home/sikmindz/Coding/Libraries2` were updated to use `Libraries` only. Active reference scan receipt: `docs/salvage/reference_scan_receipt.txt`.

Result: no active path/dependency references to `Libraries2` outside salvage/archive receipts.
