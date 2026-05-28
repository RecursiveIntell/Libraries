# v25 apply sequence

1. Read the supersession note and the docs/v25 pack.
2. Land the root spec files and current taught-surface docs.
3. Confirm `profile-runtime` is in the workspace.
4. Confirm `stack-ids`, `contract-schema-gen`, and `knowledge-runtime` surfaces are present.
5. Confirm all v25 schemas, examples, and manifests exist.
6. Confirm the expanded fixture corpus exists and matches `contracts/fixtures/v25/manifest.json`.
7. Run `bash scripts/check_v25_repo_truth.sh`.
8. Run `bash scripts/run_v25_local_checks.sh` when the Rust toolchain is available.
9. Sync `libraries-source/` using `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh`.
10. Package or review the repo only after the above checks pass.
