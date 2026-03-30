# 06_TEST_AND_CONFORMANCE_PLAN

## Non-negotiable gates

### Release truth

- `bash scripts/check_pack_truth.sh`
- `python3 scripts/check_root_archive_manifest.py`
- `bash scripts/check_doc_truth.sh`
- `python3 scripts/check_closeout_receipt.py`

### Repo truth

- `bash scripts/check_repo_surface.sh`
- `bash scripts/check_manifest_truth.sh`
- `bash scripts/check_hotspot_budgets.sh`
- `bash scripts/check_schema_registry_uniqueness.sh`
- `bash scripts/check_mirror_discipline.sh`
- `python3 scripts/check_public_type_drift.py`
- `python3 scripts/check_public_api_docs.py`

### Cargo lane (must match the declared support lane)

Define one release command and use it everywhere. Example shape:

```bash
cargo test -p contract-schema-gen
cargo test -p forge-memory-bridge
cargo test -p forge-pilot
cargo test -p kernel-conformance
cargo test -p kernel-execution
cargo test -p kernel-oracles
cargo test -p knowledge-runtime
cargo test --manifest-path living-memory/living-memory/Cargo.toml
cargo test -p llm-tool-runtime
cargo test -p recursive-kernel-core
cargo test -p semantic-memory
cargo test -p semantic-memory-forge
cargo test -p stack-ids
cargo test -p verification-adjudication
cargo test -p verification-calibration
cargo test -p verification-control
cargo test -p verification-policy
```

## New fixtures required by this finish pack

### For RUNTIME-001
- one kernel fixture where scheduler degradation reason is `budget_exhausted`
- one kernel fixture where it is `explicit_changed_nodes_required_for_delta`
- runtime snapshot proving the exact reason appears in advisory / explanation / risk-gate outputs

### For EXEC-001
- one cross-crate fixture spanning tool runtime -> pilot -> verification/control
- one serde round-trip for `ExecutionContextV1`
- one schema diff proving compatibility classification

### For TYPE-001
- grep proof that duplicate `SurfaceStatus` definitions are gone
- schema regen proof that downstream artifacts remain stable

### For SAFE-001
- supported-lane unwrap audit report
- either widened panic guard or renamed/narrowed status language

### For PACK-004
- release artifact listing with no `target-*` roots

## Reference-interpreter obligations to keep

At minimum, executable reference behavior must cover:
- valid-time / recorded-time query semantics
- view widening semantics
- bridge atomicity invariants
- repair-record invariants

If those stay prose-only, the repo will drift again.
