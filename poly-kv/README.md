# poly-kv

Shared KV-cache pool primitives for receipt-bearing compression experiments.

This nested Rust workspace implements pool construction, typed tensor shapes, per-reader access, exact fallback, memory accounting, and persistence. Its `poly-kv` crate uses q8 key blocks and a pluggable value-codec boundary. FibQuant is an optional value adapter; the TurboQuant adapter is currently an explicit unsupported stub.

## Workspace components

| Component | Purpose |
|---|---|
| [quant-codec-core](crates/quant-codec-core/) | Shape/layout contracts, codec profiles, typed identifiers, and evaluation traits |
| [poly-kv](crates/poly-kv/README.md) | Shared pool, readers, manifests, fallback, receipts, and pool storage |
| [poly-kv-python](crates/poly-kv-python/) | Native Python boundary, with wrapper code in [python/poly_kv](python/poly_kv/) |
| [provekv-pool](crates/provekv-pool/) | Separate pool-builder/exporter/importer command-line package |

## Build and test from Libraries

```bash
cd poly-kv
cargo test -p quant-codec-core
cargo test -p poly-kv
cargo test -p poly-kv --features fibquant-adapter
cargo clippy -p poly-kv --all-targets --all-features -- -D warnings
```

The optional FibQuant adapter uses the sibling `Libraries/fib-quant` source through a path dependency. Keep the repository layout intact. Python packaging is configured in [pyproject.toml](pyproject.toml) and has its own native-extension requirements.

For the current Rust API, start with the [crate README](crates/poly-kv/README.md) and [synthetic roundtrip tests](crates/poly-kv/tests/synthetic_roundtrip.rs).

## Current behavior

- `SharedKvPool::builder()` constructs a pool from typed key/value blocks
- `PoolReader` exposes scoped slice and layer decoding, with a separate explicit exact-fallback path
- `Q8KeyCodec` provides symmetric per-block key quantization; `RawExactValueCodec` provides the reference value path
- `FibQuantValueCodec`, behind `fibquant-adapter`, supplies an experimental compressed-value path with quality checks
- `KvPoolStore` and pool-bundle encode/decode APIs support persistence and reload, with digests and admission checks
- Build, decode, fallback, and evaluation receipts expose what happened; memory accounting includes retained fallback and reader costs

See [pool persistence tests](crates/poly-kv/tests/pool_persistence.rs), [shape rejection tests](crates/poly-kv/tests/shape_rejection.rs), and [memory accounting tests](crates/poly-kv/tests/memory_accounting.rs) for executable contracts.

## Evidence and limits

Compression ratios, retrieval quality, construction time, and memory use depend on the tensor shape, selected codec, retained exact fallback, serialization, and workload. Benchmark the complete path you intend to use. The earlier two-tier `SharedKVPool`/`AgentShell` examples do not describe the current `SharedKvPool` API.

The current `turbo-quant-adapter` feature exposes a constructor that returns `UnsupportedAdapter`; it does not install a working hot-tier codec. Transformer-shaped import/export types and experimental CUDA/llama integration sources in this tree are not evidence of a validated drop-in serving integration.

The [claim boundary](docs/PUBLIC_CLAIM_BOUNDARY.md), [benchmark tiers](docs/BENCHMARK_TIERS.md), and [source map](docs/SOURCE_OF_TRUTH_MAP.md) describe additional design and evidence boundaries. Read dated plans alongside current source rather than treating their proposed work as implemented.

## Licensing

The `poly-kv` package declares `MIT OR Apache-2.0` in its [manifest](crates/poly-kv/Cargo.toml). Check each workspace member's manifest and accompanying notices for its own licensing information.
