# Changelog

All notable changes to `scr-runtime-compression` are documented here.

## [Unreleased]

## [0.1.0] — 2026-06-02

First crates.io release.

### Added

- `CompressedSearchPath` — a search path that uses a
  compressed candidate index followed by exact rerank against
  the raw vectors. Production-mode path for memory recall with
  a compressible corpus.
- `ExactFallbackAdapter` — typed wrapper that takes any
  compressed representation and a raw-fallback, and always
  returns the raw result. Emits a `FallbackReceiptV1` on
  every call.
- Feature flags: `turbo`, `fib`, `polar`, `qjl` (all on by
  default), `gpu` (off by default).
- 4 codec adapters: `turbo-quant`, `fib-quant`, polar, QJL.

### Test coverage

- Exact-fallback contract tests.
- Oversample sweep tests (k=10, oversample 1, 4, 16, 64).
- Admissibility routing tests.

[Unreleased]: https://github.com/nousresearch/Libraries/compare/scr-runtime-compression-v0.1.0...HEAD
[0.1.0]: https://github.com/nousresearch/Libraries/releases/tag/scr-runtime-compression-v0.1.0
