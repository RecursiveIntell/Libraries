# proveKV speed fix v5 — SIMD Gram lookups + batch head scoring

## Architecture

The fully prepared scoring loop per token is:
```text
for block_idx in 0..block_count:
    total += gram[query_idx[block] * N + stored_idx[token, block]]
```

For head_dim=64, block_count=16, N=32. The Gram table is 32x32 = 1024 f32 = 4KB.

### Optimization 1: Pre-fetched Gram rows

Since query_indices is the same for all tokens, precompute the 16 relevant Gram rows
into a contiguous `block_count * N` f32 buffer. Then per-token scoring becomes:

```text
for block_idx in 0..block_count:
    total += gram_rows[block * N + stored_idx[token, block]]
```

This eliminates the `query_idx * N` multiply and makes the access pattern more
cache-friendly (16 rows of 128 bytes each = 2KB working set vs 4KB scattered).

### Optimization 2: SIMD f32 horizontal sum

The 16 Gram lookups per token can be done as:
1. Gather 16 f32 values from gram_rows using stored_idx offsets
2. Sum them horizontally

On AVX2: 8 f32 per register. 16 values = 2 registers. Sum = 1 horizontal add.
The gather instruction (`vgatherdps`) can do this, but it's often slower than
sequential loads on modern CPUs. The simpler approach: unrolled sequential loads
+ fma accumulation, which the compiler can auto-vectorize.

### Optimization 3: Batch head scoring

Currently: score one head at a time, prepare query per head.
Optimization: prepare all 12 heads' queries, score all heads in one pass.
This amortizes the token loop overhead across heads and improves cache utilization
for the key_indices (which are laid out as token*num_heads + head_idx).

## Implementation plan

### P0 — Pre-fetched Gram rows in fully prepared index

Add to `FullyPreparedCompressedIndex`:
- `gram_rows: Vec<f32>` — pre-fetched Gram rows for a specific query
  (computed per query, not per index build, but stored in the index for reuse)

Add `FullyPreparedCompressedIndex::prepare_query_gram_rows(&mut self, query: &[f32])`:
- Prepare query via scorer
- For each block, copy gram row `query_indices[block]` into `gram_rows[block * N..]`

Add `SharedKVPool::attention_topk_fully_prepared_simd(&self, index, query, top_k)`:
- Call `prepare_query_gram_rows`
- Score loop: `total += gram_rows[block * N + stored_idx]` — sequential, auto-vectorizable

### P0 — Batch head scoring

Add `SharedKVPool::attention_topk_batch_heads(&self, index, queries: &[&[f32]], top_k)`:
- For each query (one per head): prepare gram rows
- For each token: score all heads in one pass
- Return per-head top-k results

### P1 — Receipts/docs/tests

- Add test for SIMD path matching regular attention
- Run benchmark with SIMD path
- Store receipt
- Update docs

## Acceptance gates

- `cargo test --all-targets` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo package --allow-dirty --no-verify` packages
- `bash scripts/provekv_stack_smoke.sh` passes
- Benchmark shows whether pre-fetched Gram rows improve speed

## Claim boundary

Safe only if measured. Synthetic random vectors. Not production speedup.