# Changelog

All notable changes to `fib-quant` are documented here.

## [Unreleased]

## [0.1.0-beta.4] — 2026-07-12

### Added

- C kernels for codec encode/decode (`c-kernels/codec.c`) and
  compressed attention logits + softmax (`c-kernels/attention.c`),
  compiled at build time via `cc` crate with `-O3 -mavx2 -mfma`.
- `src/ffi.rs` — FFI boundary module with `extern "C"` declarations.
- `src/archive/codec_rust.rs`, `src/archive/attention_rust.rs` —
  original Rust implementations preserved with headers.
- `build.rs` and `cc` build dependency.
- Fully prepared compressed index with pre-unpacked indices and norms.

### Changed

- `src/codec.rs`, `src/scoring.rs`, `src/kv/compressed_attention.rs`
  now call through the FFI layer to C kernels.
- `unsafe_code` lint allowed at crate level for the FFI module.

### Verified

- 122 tests pass with `cargo test --all-features`.
- `cargo clippy --all-features --all-targets -- -D warnings` clean.

## [0.1.0-alpha.1] — 2026-06-02

First crates.io release.

### Added

- `FibQuantizer` with `encode`, `decode`, `encode_batch`,
  `decode_batch`. Single-thread by default; `parallel`
  feature flag enables Rayon.
- `FibProfile` — typed profile with `paper_default` (k=4,
  N=32), `compact` (k=4, N=32, binary-packed), and `kv`
  (KV-cache-tuned).
- `LloydRefinement` of Fibonacci-sampled seed codebooks
  (parity-verified).
- Fast Walsh-Hadamard rotation with CPU fallback and optional
  CUDA dispatch via `gpu-backend`.
- `KvCacheCodec` impl operating on `KvTensorShape`
  (`src/kv/`, ~2,500 lines).
- `FibEncodeReceipt` and `KvEncodeReceipt` typed receipts.

### Benchmarks

- Compression ratio: 3.6× JSON, ~48× binary.
- Recall@1: 1.000 (P26 measurement, 8 queries / 200 docs /
  768-dim).
- Cosine fidelity: 0.863 single-vector, 0.9996 with
  turbo-quant rerank.
- Encode batch speedup: 5.7-40× on qwen3/nomic after the
  June 1 perf pass (SIMD + Rayon).
- GPU path: 2-7% win on the encode pipeline (Hadamard-only).

### Test coverage

- 23 integration test files covering encode/decode roundtrip,
  corruption rejection, profile digests, Lloyd refinement,
  rotation identity, spherical-beta sampling, KV-cache shape
  contracts, KV-cache attention quality, and property-based
  bitpack/codec tests.

[Unreleased]: https://github.com/RecursiveIntell/Libraries/tree/main/fib-quant/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/RecursiveIntell/Libraries/tree/main/fib-quant/releases/tag/v0.1.0-alpha.1
