# proveKV/poly-kv speed fix v2 — research-informed implementation plan

## Research findings

### Competitive landscape (from arxiv + semantic memory)

1. **KIVI** (ICML 2024, arXiv:2402.02750): 2-bit asymmetric KV cache quant. Key insight: quantize keys per-channel, values per-token. The bottleneck is memory bandwidth — loading the KV cache is what makes attention slow. 2.5x batch size, 2.23x throughput.

2. **Quest** (ICML 2024, arXiv:2406.10774): Query-aware sparsity. Keeps page-level min/max key values, estimates page criticality from query, loads only top-K pages. 2.23x self-attention speedup, 7.03x latency reduction. This is the closest competitor to proveKV's approach.

3. **SANTA** (ICML 2026, arXiv:2605.01910): Stochastic sparse attention for memory-bound inference.

4. **Salca** (2026): Sparsity-aware hardware accelerator for long-context attention decoding.

### Key insight from research

The bottleneck is **memory bandwidth**, not compute. Loading the full KV cache dominates attention time. Both KIVI and Quest show that reducing bytes loaded (quantization) or reducing positions loaded (sparsity) gives real speedup. The speedup comes from **loading fewer bytes**, not from the scoring operation being faster.

### Current proveKV bottlenecks (from codebase audit)

**Python benchmark (receipt generator):**
1. Per-position Python loop: each position does quantize + small matmul + topk + exact rerank + softmax + weighted sum separately. Python loop overhead dominates.
2. No batch position processing: N positions = N separate small matmuls instead of 1 big matmul.
3. Query code recomputed per position.
4. No page-level pre-filtering (Quest-style min/max).

**Rust runtime (pool.rs/shell.rs):**
1. `attention_topk_compressed` rebuilds scorer/quantizer/codes on every call.
2. No prepared index API — codec adapter, quantizer, scorer all reconstructed per query.
3. Score loop iterates one-by-one with bounds checking per element.
4. No batch scoring across heads.

**compressed-scorer crate:**
1. PerDimScorer.score_prepared does a per-dimension loop with LUT lookup. This is O(dim) per candidate, not O(1).
2. score_batch_prepared defaults to sequential iteration.
3. AttentionCache.logits loops per key.
4. CandidateList uses heapless Vec with linear scan for find_worst.

## Implementation plan

### P0 — Python: batch all-positions attention operator

Highest ROI: transforms N small matmuls into 1 big matmul, eliminates Python loop overhead.

- Add `batch_optimized_compressed_attention_outputs(...)` to `distilgpt2_attention_speed_bench.py`
- Precompute all query codes in one batch
- Compute all approx scores as one `key_codes @ query_codes.T` matmul
- Apply causal mask via triangular masking
- Batch top-k selection using argpartition on the full score matrix
- Batch exact rerank for all selected positions at once
- Batch softmax + weighted sum
- Add `batch_optimized_compressed_timing` to receipt

### P0 — Python: Quest-style page min/max pre-filter

Second highest ROI: skip entire key blocks that can't be in top-k before per-vector scoring.

- Divide keys into pages of P tokens
- Precompute per-page min/max bounds
- For each query, estimate page score bounds = sum of max positive contributions + min negative
- Only score vectors from pages whose upper bound exceeds the current k-th score
- This is the Quest algorithm: page-level pre-filter before per-vector scoring
- Add `quest_page_filtered_timing` to receipt

### P1 — Rust: prepared compressed index

Third highest ROI: avoid rebuilding scorer/quantizer/codes per call.

- Add `PreparedCompressedIndex` struct to poly-kv that caches:
  - Decoded key codes (once per layer)
  - Built scorer/quantizer (once per pool)
  - Prepared query (once per query)
- Add `attention_topk_compressed_prepared(&self, index: &PreparedCompressedIndex, ...)` to `SharedKVPool` and `AgentShell`

### P1 — Rust: unsafe score loop optimization

- In `attention_topk_compressed`, replace safe iterator bounds checking with raw pointer iteration for the hot scoring loop
- This removes per-element bounds checks that the optimizer may not eliminate

### P2 — Receipts/docs/tests

- Add RED test for batch_optimized and quest_page_filtered receipt fields
- Regenerate speed receipt
- Update README/CHANGELOG/skill

## Acceptance gates

- RED test fails before receipt update
- `python3 -m py_compile` passes
- `cargo test --all-targets` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo package --allow-dirty` verifies
- `bash scripts/provekv_stack_smoke.sh` passes
- New receipt shows whether batch/quest paths improve speed ratio

## Claim boundary

Safe only if measured: batch-optimized and quest-filtered NumPy operator timing over precomputed DistilGPT2 Q/K/V. Still not production speedup, GPU evidence, or end-to-end generation latency.