# Changelog

All notable changes to `quant-eval` are documented here.

## [Unreleased]

### Added

- HyperQuant real-corpus/qrels retrieval gate: `run_hyperquant_real_corpus_eval` compares exact f32 retrieval against HyperQuant Z1/A2 reconstructed-vector retrieval and emits `hyperquant-real-corpus-eval-v1` receipts.
- BEIR/Scifact all-minilm receipt: 5,183 documents, 300 test queries, Z1 exact-rerank recovery@1 0.8667 / top-K overlap 0.5514, A2 exact-rerank recovery@1 0.8733 / top-K overlap 0.5910, both passing declared candidate-gate thresholds.
- Scifact codec-comparison receipt: simple int8 baselines outperform current HyperQuant Z1/A2 for embedding retrieval quality and compression ratio; HyperQuant still passes the candidate gate and beats the 1-bit sign control.
- Receipt metrics now include recall@1/5/10/K, NDCG@K, top-K overlap, exact-rerank recovery@1, rank-drift mean/p95/max, score-error mean/p95/max, search timing, byte accounting, compression ratio, and explicit blockers.
- Runnable examples `hyperquant_real_corpus_receipt`, `hyperquant_scifact_eval`, and `hyperquant_scifact_compare`, plus Scifact Ollama builder `tools/hyperquant_scifact/build_scifact_ollama.py` and stored receipts/summaries under `docs/codex-runs/`.

### Claim boundary

- The in-tree tiny fixture proves the reusable gate/API/receipt path. It is not BEIR/Scifact quality evidence or production admissibility.
- The P2 Scifact receipt is BEIR/Scifact candidate-gate evidence for all-minilm embeddings only; it is not model-quality preservation evidence or production admissibility.
- The comparison receipt says current HyperQuant is worth pursuing as a research/evidence-bearing lattice primitive, not as the first production embedding codec over int8.

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
