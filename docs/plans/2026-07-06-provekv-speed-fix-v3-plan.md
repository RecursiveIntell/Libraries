# proveKV speed fix v3 — Rust prepared index + Python batch optimization

## Research conclusions

From KIVI (arXiv:2402.02750), Quest (arXiv:2406.10774), SANTA (ICML 2026), and codebase audit:

1. The bottleneck is memory bandwidth, not compute. Loading fewer bytes (quantization) or fewer positions (sparsity) is what gives real speedup.
2. Quest achieves 2.23x self-attention speedup by page-level pre-filtering before per-vector scoring.
3. The current Python batch-optimized path (0.5822x) is approaching break-even but is still slower than exact dense on aggregate because Python loop overhead and per-position exact rerank/softmax remain.
4. The Rust runtime path rebuilds codec adapters, scorers, and decodes code payloads on every call. This is the single biggest runtime overhead.

## Implementation plan

### P0 — Rust PreparedCompressedIndex

Highest ROI: eliminate per-call reconstruction of codec/scorer/codes.

Add to `poly-kv/src/pool.rs`:
- `PreparedCompressedIndex` struct that caches:
  - Decoded key codes (once per layer)
  - Built FibScorer/quantizer (once per pool)
  - Layer/head metadata
- `SharedKVPool::prepare_compressed_index(layer_idx, head_idx) -> PreparedCompressedIndex`
- `SharedKVPool::attention_topk_compressed_prepared(&self, index: &PreparedCompressedIndex, query: &[f32], top_k: usize) -> Result<CompressedAttentionSelection>`
  - Same logic as `attention_topk_compressed` but uses pre-built scorer/codes

Add to `poly-kv/src/shell.rs`:
- `PreparedShellIndex` struct that caches shell-specific decoded codes + turbo quantizer
- `AgentShell::prepare_compressed_index(&self, pool: &SharedKVPool, layer_idx, head_idx) -> PreparedShellIndex`
- `AgentShell::attention_topk_compressed_prepared(&self, pool: &SharedKVPool, index: &PreparedShellIndex, query: &[f32], top_k: usize) -> Result<CompressedShellAttentionSelection>`

### P0 — Rust isolated attention benchmark

Add `poly-kv/examples/poly_kv_isolated_attention_bench.rs`:
- Build a synthetic pool with known tokens
- Time:
  - exact dense attention (decompress + score all)
  - compressed attention (current API, rebuilds per call)
  - prepared compressed attention (new API, pre-built index)
- Emit JSON receipt with timing, speed ratios, quality metrics

### P1 — Python batch path: vectorized exact rerank

Optimize `batch_optimized_compressed_attention_outputs`:
- Batch exact rerank: gather all selected keys into one batch and score against all queries in one matmul
- Batch softmax: compute softmax for all positions at once
- Batch weighted sum: gather all selected values and compute weighted sum in batch

### P2 — Receipts/docs/tests

- Add RED test for Rust prepared index receipt
- Run Rust isolated attention benchmark
- Regenerate Python speed receipt with vectorized batch
- Update README/CHANGELOG/skill

## Acceptance gates

- RED test fails before implementation
- `cargo test --all-targets` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo package --allow-dirty` verifies
- `bash scripts/provekv_stack_smoke.sh` passes
- `python3 -m py_compile` passes
- New receipts show whether prepared index improves speed

## Claim boundary

Safe only if measured. Rust prepared index timing is synthetic pool evidence, not production speedup.