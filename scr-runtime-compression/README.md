# scr-runtime-compression

Runtime integration adapter for `semantic-memory`'s compression
layer.

`scr-runtime-compression` is the **runtime adapter** that lets
`semantic-memory` use `turbo-quant` and `fib-quant` without
taking a hard dependency on either. The two key types are:

- **`CompressedSearchPath`** — a search path that uses a
  compressed candidate index (from `turbo-quant` or
  `fib-quant`) followed by exact rerank against the raw
  vectors. This is the production-mode path for memory recall
  with a compressible corpus.
- **`ExactFallbackAdapter`** — a typed wrapper that takes
  any compressed representation and a raw-fallback, and
  always returns the raw result. The contract: the adapter
  emits a `FallbackReceiptV1` on every call, so the audit
  trail captures which path served the request.

The crate is **alpha**. The runtime adapter works but the
GPU path is gated off-by-default because the per-call H2D/D2H
overhead negates the kernel speedup at the current call
granularity.

## What's in the box

### `CompressedSearchPath`

```rust
pub struct CompressedSearchPath {
    compressed_index: Box<dyn CompressedIndex>,
    raw_corpus: Vec<Vec<f32>>,
    profile_digest: CodecProfileDigest,
}

impl CompressedSearchPath {
    pub fn search(&self, query: &[f32], k: usize) -> Result<SearchResult, CompressionError> {
        // 1. Get top-k * oversample candidates from compressed index
        // 2. Exact rerank on raw_corpus
        // 3. Return reranked top-k with a Receipt
    }
}
```

The `oversample` factor is the key control: higher oversample
gives better recall at the cost of more rerank work. The
default is 4 (matches the turbo-quant smoke benchmark setup).

### `ExactFallbackAdapter`

```rust
pub struct ExactFallbackAdapter<C: CompressedIndex, R: RawStore> {
    compressed: C,
    raw: R,
    // Emits FallbackReceiptV1 on every call
}
```

The adapter's contract:

- **If the compressed index is admissible for the query**
  (size, accuracy, latency), it serves the result from the
  compressed index and emits a `FallbackReceiptV1 { path: "compressed" }`.
- **If the compressed index is not admissible** (e.g. caller
  asked for `Admissibility::Exact`), it serves from the raw
  store and emits a `FallbackReceiptV1 { path: "raw" }`.
- **Every call emits exactly one receipt.** The audit trail
  records the path taken.

### Feature flags

| Feature | Default | What it enables |
|---|---|---|
| `turbo` | yes | `turbo-quant` codec adapter |
| `fib` | yes | `fib-quant` codec adapter |
| `polar` | yes | Polar-only compression (asymmetric) |
| `qjl` | yes | QJL sketches for residual recovery |
| `gpu` | no | GPU dispatch via `gpu-backend` |

The default is `["turbo", "fib", "polar", "qjl"]` — all four
codecs available, no GPU (because the GPU path is slower in
integration at this time).

## Quick Start

```rust
use scr_runtime_compression::{CompressedSearchPath, ExactFallbackAdapter};
use turbo_quant::{TurboSidecarCode, TurboSidecarIndex};
use quant_codec_core::{KvTensorShape, DType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a compressed index.
    let code = TurboSidecarCode::encode(&profile, &corpus)?;
    let index = TurboSidecarIndex::build(&profile, code)?;

    // Wrap it in the search path.
    let path = CompressedSearchPath::new(index, corpus.clone(), profile_digest);

    // Search.
    let result = path.search(&query, 10)?;
    assert_eq!(result.results.len(), 10);
    println!("Receipt: {:?}", result.receipt);
    Ok(())
}
```

Run it: `cargo run --example basic_search` (see `examples/`).

## Test coverage

- **Exact-fallback contract tests** — every adapter call
  emits exactly one receipt, and the path recorded matches
  the actual path taken.
- **Oversample sweep tests** — k=10 with oversample = 1, 4,
  16, 64; assert recall@10 and rerank-cost.
- **Admissibility routing** — caller says
  `Admissibility::Exact`, the adapter routes to raw; caller
  says `Admissibility::Approximate`, the adapter routes to
  compressed.
- `cargo test --all-features` clean.
- `cargo clippy --all-targets -- -D warnings` clean.

## MSRV

Rust 1.75 (2021 edition). Stable features only.

## Dependencies

- `bytemuck` (with `derive`) — for safe zero-copy codec
  output.
- `serde` (with `derive`).
- `serde_json`.
- `thiserror`.
- `chrono` (for receipt timestamps).
- `quant-governor` — for the policy routing layer.
- `turbo-quant` (optional) — for the `turbo` feature.
- `fib-quant` (optional) — for the `fib` feature.

## License

MIT. See `LICENSE-MIT` for the full text.

## Changelog

See `CHANGELOG.md` for the release history.

## Where it's used

`scr-runtime-compression` is the integration layer for:

- `semantic-memory` — every recall over a corpus with
  `Admissibility::Standard` or below routes through
  `CompressedSearchPath`.
- The `quant-governor` policy engine — when the policy
  routes to a compressed codec, the `ExactFallbackAdapter` is
  the one that actually executes the call.

Any system that wants to **add governed compression** to an
existing search path can adopt `scr-runtime-compression`
directly.
