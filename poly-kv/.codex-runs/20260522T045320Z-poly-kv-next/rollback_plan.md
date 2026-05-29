# Rollback Plan

## Phase 01

Revert:

- `crates/quant-codec-core/src/shape.rs`
- `crates/quant-codec-core/src/lib.rs`
- `crates/quant-codec-core/tests/shape_validation.rs`

## Phase 02-03

Revert:

- `crates/poly-kv/src/codecs/q8_keys.rs`
- `crates/poly-kv/src/manifest.rs`
- `crates/poly-kv/src/memory.rs`
- `crates/poly-kv/src/pool.rs`
- `crates/poly-kv/src/reader.rs`
- `crates/poly-kv/src/receipts.rs`
- `crates/poly-kv/tests/memory_accounting.rs`
- `crates/poly-kv/tests/receipt_roundtrip.rs`
- `crates/poly-kv/tests/synthetic_roundtrip.rs`
- `scripts/assert_realized_accounting.py`

## Phase 04-05

Remove:

- `crates/poly-kv-python/`
- `python/`
- `pyproject.toml`

Then remove `crates/poly-kv-python` from workspace `Cargo.toml` and refresh `Cargo.lock`.

## Phase 06

Remove:

- `scripts/bench_rust_synthetic.py`
- `scripts/bench_boundary.py`
- `scripts/compare_receipts.py`
- generated Phase 06 JSON artifacts under `.codex-runs/20260522T045320Z-poly-kv-next/`

## Phase 07

Revert:

- `README.md`
- `crates/poly-kv/README.md`
- `docs/PY_SIDECAR_SPEC.md`
- `docs/BENCHMARK_AND_HARNESS_SPEC.md`

Remove:

- `docs/NEXT_RELEASE_PLAN.md`
- `docs/BENCHMARK_TIERS.md`
- `docs/CLAIM_BOUNDARY.md`

## Audit artifacts

Remove `.codex-runs/20260522T045320Z-poly-kv-next/` to roll back run records only.
