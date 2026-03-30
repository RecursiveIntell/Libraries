# CONFORMANCE_GATES

A finish pass is conforming only if:

1. `bash scripts/check_pack_truth.sh` passes for the numbered hostile-finish pack.
2. `python3 scripts/check_root_archive_manifest.py` passes for the active root archive manifest.
3. `STATUS_DASHBOARD.md`, `STATUS_EVIDENCE_MANIFEST.json`, and `release/closeout_receipt_v1.json` describe the same reproducible HEAD state.
4. `SUPPORT_PROFILE.md`, `Makefile`, and the receipt encode one explicit release lane.
5. `cargo run -p contract-schema-gen -- schemas.generated` remains the schema regeneration command taught by the front door.
6. `cargo clippy --workspace --all-targets --all-features -- -D warnings` remains the warnings-fatal policy for full-workspace hygiene, while the release lane cargo proof stays scoped to `SUPPORT_PROFILE.md`.
7. degraded reasons survive the kernel -> runtime artifact path.
8. duplicate primitive types are gone.
9. anything shipped as `source-clean` contains no `target-*` tree.
10. one live proof/demo lane exists and is labeled honestly.
