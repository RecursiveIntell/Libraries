# Changelog

All notable changes to `quant-eval` are documented here.

## [Unreleased]

## [0.1.0] — 2026-06-02

First crates.io release.

### Added

- `admissibility` benchmark — verifies codec classifications
  against their actual behavior. Lossy codecs cannot
  mis-classify as Exact; the harness rejects them.
- `compression` benchmark — measures raw / encoded bytes,
  ratio, per-block statistics, and the theoretical ratio.
- `semantic` benchmark — measures NDCG@k, mean rank drift,
  cosine similarity, and exact-rerank recovery rate.
- `EvalReport` matching `quant_codec_core::EvalReport` shape.
- 1 integration test exercising the full pipeline on a
  synthetic corpus.

### Test coverage

- 3 internal benchmarks (admissibility.rs, compression.rs,
  semantic.rs) — 1,204 lines of test code.
- 1 integration test (tests/integration.rs) — 98 lines.

[Unreleased]: https://github.com/RecursiveIntell/Libraries/tree/main/quant-eval/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/RecursiveIntell/Libraries/tree/main/quant-eval/releases/tag/v0.1.0
