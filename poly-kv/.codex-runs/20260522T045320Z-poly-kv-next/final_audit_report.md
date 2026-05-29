# Final Hostile-Auditor Handoff

Run id: `20260522T045320Z-poly-kv-next`

## Summary

This pass advanced the alpha workspace with Shape V2 contracts, persisted eval receipts, realized byte accounting, active reader scratch accounting, stronger decode receipts, an optional PyO3 Python sidecar skeleton, Python tests with explicit native skips, Tier 0/Tier 1 JSON harnesses, and updated claim-bounded docs.

## Source-of-truth decisions

- `quant-codec-core`: owns `KvCacheShapeV2` and `KvAttentionKind`.
- `poly-kv`: owns pool/fallback/manifest/receipt/accounting semantics.
- `poly-kv-python`: owns optional binding layer only.
- TurboQuant/FibQuant adapters remain unsupported stubs; external APIs were not inspected.

## Receipt/accounting changes

- `PoolBuildReceiptV1` now persists `compression_evals`.
- `CompressionEvalReceiptV1` and `BlockManifestEntryV1` expose ideal bits/scalar, realized encoded bytes, and metadata bytes.
- `DecodeReceiptV1` exposes full-block decode, decoded full values, returned values, and copy behavior.
- Manifest bytes use canonical serialized length.
- Active reader scratch bytes are tracked across attach/drop.

## Python sidecar status

- Layout, PyO3 crate, typed stubs, custom exceptions, wrappers, and tests exist.
- Rust core crates do not depend on PyO3/maturin.
- `maturin` is unavailable in this environment, so native wheel/build validation and native operation tests are skipped with explicit reasons.
- No daemon mode was added.

## Benchmark receipts

- `.codex-runs/20260522T045320Z-poly-kv-next/rust_synthetic_bench.json`: pass.
- `.codex-runs/20260522T045320Z-poly-kv-next/python_boundary_bench.json`: skip, native extension unavailable.
- `.codex-runs/20260522T045320Z-poly-kv-next/receipt_parity_report.json`: pass with recorded Python boundary skip.

## Validation

See `validation_results.md`. Full Rust gates pass. Public claim and boundary scripts pass. Python package import passes; native sidecar execution is skipped because `_native` is not installed.

## Unresolved blockers

- `maturin` unavailable.
- `cargo-semver-checks` unavailable.
- Native Python tests skipped until the extension is built.

## Rollback

See `rollback_plan.md`.
