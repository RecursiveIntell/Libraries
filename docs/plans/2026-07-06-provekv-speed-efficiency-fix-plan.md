# proveKV/poly-kv speed and efficiency fix plan

## Claim under audit

Current receipts show decode-work reduction, but the isolated NumPy CPU attention benchmark is slower than exact dense attention. The task is to remove every speed/efficiency bug visible in the current harness/runtime path, rerun receipts, and keep the claim boundary honest.

## Findings before implementation

1. Timing bug: `vectorized_compressed_attention_outputs(...)` recomputes quantized key codes and scales inside every timed call. That is setup/materialization cost, not per-query operator cost.

2. Timing bug: compressed timed functions compute quality diagnostics (`exact_top`, Jaccard overlap) inside the benchmarked operation. That injects an exact dense score path into the compressed timing and makes the speed result pessimistic/invalid for operator timing.

3. Timing bug: scalar and vectorized paths share one receipt field, making it hard to tell naive implementation cost from optimized prequantized operator cost.

4. Full-forward efficiency bug: `distilgpt2_full_forward_intervention.py` quantizes every prefix key set in a Python loop for every token position. The key codes/scales for a layer/head should be computed once and sliced by prefix.

5. Selection bug/inefficiency risk: speed receipts must keep quality metrics outside the timed hot path, otherwise future speedups can be fake or masked by diagnostics.

6. Runtime architecture issue: Rust `attention_topk_compressed` still decodes code payloads and builds scorers on every call. This is only partially fixed in this pass: the top-k stage now avoids full sort with `select_nth_unstable_by`, but a full prepared-index API is still the next P0.

7. Runtime hot-path bug: Rust cold-pool and shell compressed top-k selected candidates by sorting every scored candidate. For top-k selection this should be partition-then-sort-selected, not full sort.

## Implementation plan

### P0 — Correct isolated benchmark methodology

- Add untimed quality functions and timed operator-only functions.
- Add prequantized-key setup outside timed loops.
- Add a new `optimized_prequantized_compressed_timing` field.
- Add `speed_ratio_exact_over_optimized_prequantized` aggregate.
- Keep old scalar/vectorized timings as diagnostics, but do not use them for the best optimized path.

### P0 — Reuse prequantized keys in full-forward replay

- Add `prepare_quantized_keys` / `quantized_scores_prepared` helpers to `distilgpt2_full_forward_intervention.py`.
- Make `sparse_attention_output(...)` accept optional precomputed key codes/scales.
- In `run_forward`, precompute capture-head key codes/scales once per layer/head, then reuse slices for every causal position.

### P1 — Rust hot-path top-k selection

- Add `select_nth_unstable_by` top-k partitioning to Rust `SharedKVPool::attention_topk_compressed(...)` and `AgentShell::attention_topk_compressed(...)` so the hot path sorts only selected candidates instead of all scored candidates.

### P2 — Receipts/docs/tests

- Add RED test for new optimized receipt fields.
- Regenerate `POLY_KV_DISTILGPT2_ATTENTION_SPEED_BENCH_RECEIPT.json` and summary.
- Update README/CHANGELOG and provekv skill reference with the corrected result.

### P2 — Explicit next P0 not implemented here

- Add Rust prepared compressed-attention index API to avoid per-query payload/code decode and scorer construction.
- Add low-level SIMD/Rust scorer or GPU/page scorer only after the corrected Python receipt proves the hot-path shape.

## Acceptance gates

- RED test fails before receipt/schema update.
- `python3 -m py_compile` passes for all poly-kv tools.
- Focused speed receipt test passes.
- `cargo test --test model_replay_tests -- --nocapture` passes.
- `cargo test --all-targets` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- `cargo package --allow-dirty` verifies.
- `bash scripts/provekv_stack_smoke.sh` passes.

## Claim boundary after this pass

Safe only if measured: optimized prequantized NumPy operator timing over precomputed Q/K/V, quality metrics outside timed path, setup excluded. Still not production speedup, GPU evidence, all-framework KV-cache speedup, or replacement for KIVI/Quest/SnapKV.
