# proveKV speed fix v4 — fully prepared index + scale sweep + real data

## Research conclusions

The fair benchmark showed prepared compressed is 2.9-5.2x slower than pre-decoded exact dense. The speed ratio improves with scale (0.19x at 64, 0.35x at 512), suggesting break-even at ~2K-4K tokens.

The bottleneck in `score_prepared` is:
1. `unpack_indices()` called per code per call — O(block_count) bit unpacking
2. `decode_stored_norm()` called per code per call — norm decoding
3. The Gram table lookup loop itself is cheap — O(block_count) lookups

If we pre-unpack all indices and norms in the prepared index, the scoring loop becomes just Gram table lookups with no per-call unpacking.

## Implementation plan

### P0 — Fully prepared index: pre-unpack indices and norms

Add to `PreparedCompressedIndex`:
- `pre_unpacked_indices: Vec<Vec<u32>>` — one Vec per token, pre-unpacked
- `pre_stored_norms: Vec<f64>` — one norm per token, pre-decoded

Add `SharedKVPool::attention_topk_fully_prepared(&self, index, query, top_k)`:
- Prepare query (O(dim))
- For each token: just loop over pre-unpacked indices and do Gram lookups
- No unpack_indices, no decode_stored_norm per call
- This is the tightest possible hot path

### P0 — Larger scale sweep

Run the benchmark at 64, 128, 256, 512, 1024, 2048 tokens in release mode.
- Use fewer repeats for large scales (repeat=20 for 1024+, repeat=50 for smaller)
- Find the break-even point

### P0 — Real DistilGPT2 Q/K/V in Rust benchmark

Add a mode that loads the captured DistilGPT2 fixture JSON and builds a pool from those tensors:
- Load `POLY_KV_CAPTURED_DISTILGPT2_FIXTURE.json`
- Extract layer-0 head-0 Q/K/V
- Build pool from K vectors (head_dim=64)
- Run exact dense vs prepared vs fully-prepared
- Measure top-k overlap on real attention data

### P1 — head_dim=64 support

DistilGPT2 uses head_dim=64, not 8. Run the benchmark at both dims to see how the ratio changes.

### P2 — Receipts/docs/tests

- Add RED test for fully prepared receipt
- Store all receipts
- Update README/CHANGELOG

## Acceptance gates

- RED test fails before implementation
- `cargo test --all-targets` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo package --allow-dirty` verifies
- `bash scripts/provekv_stack_smoke.sh` passes
- Receipt shows whether fully-prepared improves speed ratio and where break-even occurs

## Claim boundary

Safe only if measured. Synthetic random vectors and captured DistilGPT2 Q/K/V are not production speedup evidence.