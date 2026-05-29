# Current State Audit — poly-kv 20260520 package

## Observed current state

The uploaded package is a real Rust workspace, not a blank scaffold. It contains:

- root `Cargo.toml` workspace with `crates/quant-codec-core` and `crates/poly-kv`;
- `quant-codec-core` with codec IDs, digests, dtype, eval, shape, and codec traits;
- `poly-kv` with `SharedKvPool`, builder, reader, exact fallback, q8 key path, raw exact value path, manifests, receipts, metrics, tests, and benchmark harness;
- docs for implementation spec, claim boundary, schema proposal, benchmark plan, source-of-truth map, and risk register;
- scripts for preflight, Rust gates, schema validation, public-claim checking, final-state validation;
- prior `.codex-runs/20260520T174516Z-alpha1` evidence.

## Static findings from direct zip inspection

### S1 — Shape semantics are too weak for real HF / GQA / MQA correctness

Current `KvTensorShape` uses `key_heads` and `value_heads`, but real cache interoperability needs at least:

- `batch`
- `num_layers`
- `num_q_heads`
- `num_kv_heads`
- `seq_len`
- `head_dim`
- layout
- dtype
- attention kind: MHA / MQA / GQA / unsupported

Current synthetic fixtures name `shape_mqa` and `shape_gqa` while setting different key/value head counts. In most transformer KV caches, key and value head counts should normally match `num_kv_heads`; query heads are a separate model property. This can cause false confidence before HF adapter work.

Fix direction: replace or supersede `KvTensorShape` with `KvCacheShapeV2` that models query heads and KV heads explicitly. Preserve V1 serde only if needed behind `compat-v1` and mark it deprecated.

Acceptance gate: tests for MHA, MQA, GQA shapes must assert `num_q_heads % num_kv_heads == 0`, `num_kv_heads > 0`, `batch > 0`, and reject unsupported MLA/hybrid layouts.

### S1 — Compression eval receipts are created but discarded

`PoolBuilder::build_from_blocks` builds `CompressionEvalReceiptV1` values, then discards them. That violates the design goal that material lossy operations emit receipt-bearing artifacts.

Fix direction: store `compression_eval_receipts: Vec<CompressionEvalReceiptV1>` in `PoolBuildReceiptV1` or make them retrievable from `SharedKvPool::compression_eval_receipts()`.

Acceptance gate: a test must fail if a lossy key codec builds a pool without an eval receipt per compressed key block.

### S1 — Manifest byte accounting is estimated, not serialized

Current manifest accounting uses an estimate. For the next pass, realized byte accounting must derive from actual serialized/canonical manifest bytes, and codec fields must separate:

- `ideal_codec_bits_per_scalar`
- `realized_encoded_bytes`
- `metadata_bytes`
- `fallback_bytes`
- `manifest_bytes`
- `decoded_working_bytes`
- `per_reader_scratch_bytes`

Acceptance gate: a test serializes the manifest and asserts `manifest_bytes == serialized_manifest.len()` or records the exact canonicalization method used.

### S1 — Reader scratch memory accounting uses default config, not actual attached readers

`SharedKvPool::memory_accounting()` recomputes per-reader scratch as `reader_count * ReaderConfig::default().scratch_bytes()`. If callers attach readers with custom budgets, accounting lies.

Fix direction: store actual active scratch bytes in an atomic aggregate and decrement it on `Drop`, or maintain a receipt-bearing reader registry.

Acceptance gate: attach readers with two different scratch budgets; total memory must equal the sum of actual budgets, then return to zero after drop.

### S2 — Decode path decodes full blocks for slice requests

`decode_slice` decodes the full block and extracts the slice. That is acceptable for alpha but must be explicit in receipts.

Fix direction: include `full_block_decoded: bool`, `decoded_full_values`, and `returned_values` in `DecodeReceiptV1`; later implement slice-aware decode.

Acceptance gate: slice decode receipt records whether the full block was decoded.

### S2 — Adapter stubs are honest but the next pass must not call them "support"

`TurboQuantValueCodec` and `FibQuantValueCodec` are unsupported stubs. That is correct until external APIs are inspected.

Fix direction: either inspect local `~/Coding/Libraries/turbo-quant` / `fib-quant` and wire real optional adapters, or preserve stubs and keep README status as unsupported.

Acceptance gate: no README/PyPI docs claim real adapter support unless compile and tests prove it.

### S2 — Schema proposal is not generated from Rust types

Current schema proposal is a hand-maintained artifact. That is okay for an alpha proposal, but it is not sufficient as schema source of truth.

Fix direction: add a `schema-export` feature using `schemars` or a controlled manual generator; output generated schemas under `target/` or `docs/generated/` with receipts.

Acceptance gate: schema validation verifies generated schema includes all public receipt/manifest structs.

### S2 — No Python sidecar surface exists yet

The Rust core now exists. The next needed adoption path is a PyO3/maturin sidecar, but it must be a separate bindings layer and not a daemon-first architecture.

Fix direction: add `crates/poly-kv-python` or `bindings/python`, `pyproject.toml`, `python/poly_kv`, private `_native` module, handwritten `.pyi`, `py.typed`, custom exceptions, smoke tests, and JSON receipt parity tests.

Acceptance gate: `maturin develop` or `maturin build` succeeds locally; `python -c "import poly_kv"` passes; Python smoke tests verify manifest/receipt parity with Rust.

### S3 — Documentation needs "alpha2 target" alignment

Docs still describe the alpha state. Next pass should introduce an explicit `docs/NEXT_RELEASE_PLAN.md` and make README status table reflect alpha2 targets without overclaiming.

Acceptance gate: public claim checker blocks production/compatibility/performance claims.
