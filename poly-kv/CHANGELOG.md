# Changelog

All notable changes to `poly-kv` are documented here.

## [Unreleased]

### Added

- `AgentShell::attention_topk_compressed(...)` scores Fib cold-pool keys and Turbo hot-shell keys in compressed form, performs global top-k selection, and decodes only selected values.
- `CompressedAttentionSelectionReceipt` now carries source candidate counts, selected pool/shell counts, optional agent/shell identity, exact-fallback requirement, and a claim boundary.
- `run_model_replay(...)` emits `poly_kv_model_replay_receipt_v1` receipts comparing compressed pool+shell selection against exact full-decode attention with synthetic projected-logit KL/top-1/PPL-proxy metrics and adaptive candidate-k selection.
- `poly_kv_model_replay_receipt` example stores the current deterministic model-shaped replay receipt under `docs/codex-runs/P3/`.
- `run_captured_model_replay(...)` emits `poly_kv_captured_model_replay_v1` receipts from captured Q/K/V/logit fixtures.
- `capture_tiny_transformer_replay.py` generates a deterministic NumPy tiny-transformer captured replay fixture when torch/transformers are unavailable.
- `capture_distilgpt2_replay.py` generates a pretrained DistilGPT2 safetensors/tokenizer captured replay fixture via manual NumPy forward pass; the stored receipt is a diagnostic negative showing the current single-head projection proxy is insufficient for KV-cache preservation claims.

### Claim boundary

- The compressed shell attention path and model-shaped/captured replay gates are candidate/replay plumbing. The stored tiny-transformer captured receipt uses deterministic NumPy tensors. The stored DistilGPT2 captured receipt uses pretrained model tensors but fails the strict logit/PPL-proxy gate. Neither proves pretrained LLM model-quality or KV-cache preservation without full-forward intervention/replay from a real small model.

## [0.1.0-alpha.1] — 2026-06-02

First crates.io release.

### Added

- `SharedKVPool::build(corpus, profile)` — build a
  deterministic, immutable shared KV-cache pool. Emits a
  `PoolBuildReceipt` with the profile digest, the block
  digest, and the per-block statistics.
- `AgentShell::materialize(pool, agent_docs, profile)` —
  per-agent overlay on top of the shared pool. 17ms for 12
  docs, 768-dim. Emits a `ShellMaterializeReceipt`.
- `PoolManifest` — typed manifest serializing the corpus,
  profile, codec, seed, block layout, receipt, and content
  digest.
- `FallbackReceiptV1` — typed receipt for exact-fallback
  events. BLAKE3-hashed and signed.
- `Policy` — typed policy object passed to `build` and
  `materialize`. Single decision point for admissibility.
- Exact fallback contract: any compressed representation
  can be re-derived back to its raw input.

### Benchmarks (June 2026)

- 10-agent contention: 10/10 agents find their target at
  rank 1. Zero cross-agent contamination.
- Single-route parity: Recall@1=1.000 across all routes
  (exact, fib-quant, turbo-quant, two-tier).
- "Do All" perf pass: 5.7-40× speedup on qwen3/nomic pool
  builds (AVX2+FMA SIMD + Rayon parallel).
- GPU path: 2.5-2.7% Hadamard-only win on larger corpora.

### Test coverage

- 4 integration test files (600 lines): `integration_tests`,
  `pool_tests`, `receipt_tests`, `shell_tests`.
- 4 examples: `poly_kv_dynamic_cache_roundtrip`,
  `poly_kv_fast_roundtrip`, `poly_kv_gpu_bench`,
  `test_compact_decode`.
- 1 bench: `synthetic_pool`.
- 25+ Python validation scripts for preflight, schemas,
  public claim checking, receipt integrity, package
  hygiene, final state.

[Unreleased]: https://github.com/RecursiveIntell/Libraries/tree/main/poly-kv/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/RecursiveIntell/Libraries/tree/main/poly-kv/releases/tag/v0.1.0-alpha.1
