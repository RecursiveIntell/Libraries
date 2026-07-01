# scr-runtime-compression

Runtime compression adapter layer for `semantic-memory`, `turbo-quant`, `fib-quant`, and the shared `compressed-scorer` trait.

This crate does not own codec truth and does not own semantic-memory retrieval semantics. It owns integration seams:

- `CompressedSearchPath` — carries compression metadata with a caller's search path.
- `ExactFallbackAdapter` — decode/fallback wrapper for exact reconstruction paths.
- `CompressedScorerAdapter` — compressed-domain candidate scorer that ranks compressed payloads without f32 decompression.

## Current adapter split

`ExactFallbackAdapter` is for verification/fallback:

```text
compressed bytes -> codec decode -> exact/decoded bytes
```

`CompressedScorerAdapter` is for hot-path candidate generation:

```text
query f32 -> prepare once -> score compressed candidates -> ranked approximate candidates
```

Semantic-memory can now choose either:

1. compressed candidate generation followed by exact f32 rerank, or
2. compressed-domain-only candidate scoring when `turbo_quant_require_exact_rerank = false`.

The second path still resolves metadata rows for returned hits, but it does not load/decode authoritative embedding blobs for final rerank.

## Example: generic compressed scorer

```rust
use scr_runtime_compression::{CodecId, CompressedScorerAdapter};

let adapter = CompressedScorerAdapter::turbo_quant(768, 8, 64, 0)?;
let candidates = vec![("item:a", turbo_code_a), ("item:b", turbo_code_b)];
let ranked = adapter.score_candidates(&query, &candidates, f32::NEG_INFINITY, 10)?;
assert!(ranked.len() <= 10);
```

## Feature flags

| Feature | Default | What it enables |
|---|---:|---|
| `turbo` | yes | `turbo-quant` encode/decode and `CompressedScorerAdapter::turbo_quant`. |
| `fib` | yes | `fib-quant` encode/decode and `CompressedScorerAdapter::fib_quant`. |
| `polar` | yes | Polar-only asymmetric encode/pass-through decode. |
| `qjl` | yes | QJL sketch encode/pass-through decode. |

Default features are `turbo`, `fib`, `polar`, and `qjl`.

## Integration contract

- Approximate scores are candidate evidence, not raw-vector truth.
- Exact rerank remains the conservative default.
- Compressed-only mode must be explicit in the caller config.
- Corrupt artifacts fail closed to raw f32 fallback where the caller has raw authority.
- This crate never imports semantic-memory database types; semantic-memory resolves IDs, filters, receipts, and source authority.

## Verification receipts from this integration pass

- `cargo test -p scr-runtime-compression` -> 26 passed, 1 doc-test passed, 1 ignored.
- `cargo tree -p scr-runtime-compression -i compressed-scorer` -> `compressed-scorer -> scr-runtime-compression`, no reverse cycle.
- Downstream semantic-memory check/test with `brute-force turbo-quant-codec` passed.

## MSRV

Rust 1.75, edition 2021.
