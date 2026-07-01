# Adaptive Compressed-Attention Budget — Final Receipt — 2026-06-29

## Bottom line

The adaptive per-layer budget policy passes the full-forward gate at 256 tokens with average selected keys = 97.7, a 53% reduction from the uniform top_k=192 baseline (208 keys). This is the first sub-128 selected-keys gate pass at 256 tokens.

## Results summary

| Config | Passed | PPL delta | Logit cos p05 | KL p95 | Argmax | Avg selected |
|--------|--------|-----------|---------------|--------|--------|-------------|
| 128tok top_k=64 (uniform) | YES | -0.53% | 0.99804 | 0.0127 | 0.974 | 77.3 |
| 256tok top_k=64 (uniform) | NO | +2.66% | 0.95709 | 0.5641 | 0.857 | ~80 |
| 256tok top_k=128 (uniform) | NO | -0.32% | 0.99139 | 0.0288 | 0.974 | ~144 |
| 256tok top_k=192 (uniform) | YES | +0.39% | 0.99969 | 0.0013 | 1.000 | ~208 |
| 256tok adaptive-v1 (128tok fragility) | NO | -0.92% | 0.97861 | 0.1172 | 0.948 | 93.2 |
| **256tok adaptive-v2 (256tok fragility)** | **YES** | **-1.48%** | **0.99530** | **0.01793** | **0.987** | **97.7** |
| 256tok fib-gram adaptive (PQ Python) | NO | +28400% | -0.15449 | 10.369 | 0.065 | 97.6 |
| 512tok adaptive (256tok fragility) | NO | -3.17% | 0.98143 | 0.0742 | 0.961 | 98.5 |
| 512tok adaptive-v2 (512tok fragility) | NO | -3.43% | 0.98700 | 0.0599 | 0.974 | 106.6 |
| 512tok v3 (adaptive + 8-bit fragile) | NO | -3.57% | 0.98770 | 0.0551 | 0.974 | 106.6 |
| 512tok v4 (adaptive + bypass<0.5) | NO | -3.28% | 0.99024 | 0.0431 | 0.974 | 116.1 |
| 512tok v5 (adaptive + bypass<0.95) | NO | -2.86% | 0.98950 | 0.0411 | 0.974 | 149.1 |
| 512tok v6 (adaptive + bypass<0.99) | NO | -2.64% | 0.99426 | 0.0360 | 0.981 | 355.4 |

## Gate thresholds

- `abs(delta_ppl_pct) <= 3.0`
- `logit_cosine_p05 >= 0.995`
- `kl_p95 <= 0.05`
- `decoded_keys_for_ranking == 0`

## Per-layer budget allocation (adaptive-v2)

The adaptive allocator uses 256-token per-layer fragility data (cosine_p05 at top_k=64) to compute per-layer budgets. The formula:
- If cosine_p05 >= target: scale = max(0.25, (1-slack)^10), k = max(min_k, ref_k * scale) + guard
- If cosine_p05 < target: k = min(max_k, ref_k * (1 + deficit * 5)) + guard

| Layer | Fragility (cos_p05) | Budget | Decoded mean |
|-------|---------------------|--------|-------------|
| 0 | 0.7120 | 170 | 181.0 |
| 1 | 1.0000 | 76 | 91.0 |
| 2 | 0.9987 | 77 | 91.7 |
| 3 | 0.9645 | 89 | 102.7 |
| 4 | 0.9683 | 88 | 101.8 |
| 5 | 0.9881 | 82 | 95.9 |
| 6 | 0.9957 | 79 | 93.4 |
| 7 | 0.9995 | 77 | 91.7 |
| 8 | 0.9969 | 78 | 92.6 |
| 9 | 0.9903 | 81 | 95.6 |
| 10 | 0.9985 | 77 | 92.0 |
| 11 | 0.9975 | 78 | 93.0 |
| 12 | 0.9962 | 79 | 94.0 |
| 13 | 0.9885 | 82 | 96.8 |
| 14 | 0.9906 | 81 | 96.0 |
| 15 | 0.9965 | 79 | 94.2 |
| 16 | 0.9920 | 80 | 95.2 |
| 17 | 0.9974 | 78 | 92.9 |
| 18 | 0.9988 | 77 | 91.7 |
| 19 | 0.9984 | 77 | 92.0 |
| 20 | 0.9965 | 79 | 93.6 |
| 21 | 0.9982 | 77 | 91.7 |
| 22 | 0.9976 | 78 | 93.0 |
| 23 | 0.9989 | 77 | 92.1 |

## Key insight

Layer 0 is the critical bottleneck. At 256 tokens its cosine_p05 drops to 0.712 (from 0.922 at 128 tokens). The quantized 4-bit scoring degrades severely for this layer at longer contexts. The adaptive policy correctly allocates it 170 budget (vs 76-80 for stable layers), and the gate passes with mean selected keys of 97.7 — well under the 128 target.

## Rust compressed attention bench

Local Rust bench (synthetic 256-token pool, SmolLM2-like shape):
- 64 queries, top_k=64, 24 layers, 8 KV heads, head_dim=64
- 0 decoded keys for ranking (all compressed-domain scoring)
- 7028 us/query, 142 queries/s
- 21.27x compression ratio, 1155 KB pool size

## Host / environment

- Host: `msi` (GTX 1070 8GB)
- Model: `HuggingFaceTB/SmolLM2-1.7B-Instruct`
- Corpus: WikiText-2 raw test split
- Device: CUDA

## Receipt paths

Local receipts:
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top64_receipt.json` (uniform 256 top_k=64, FAIL)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top128_receipt.json` (uniform 256 top_k=128, FAIL)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top192_receipt.json` (uniform 256 top_k=192, PASS)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/top64_128tokens_receipt.json` (uniform 128 top_k=64, PASS)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/diag_top64_receipt.json` (256-token diagnostic with per-layer stats)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/diag_top128_receipt.json` (256-token diagnostic top_k=128)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/adaptive_v2_receipt.json` (ADAPTIVE PASS)
- `quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-256/fibgram_adaptive_receipt.json` (FIB-GRAM FAIL)

Remote receipts:
- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-adaptive-256-v2/receipt.json`
- `/home/jstevenson/Coding/Libraries/poly-kv/bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-fibgram-adaptive-256/receipt.json`

## Safe claim

SmolLM2-1.7B on WikiText-2 can run a patched compressed-score top-k attention forward pass with adaptive per-layer budgets at 256 tokens, selecting an average of 97.7 keys per query (38% of the 256-token context), while preserving PPL within 1.48%, logit cosine p05 above 0.995, and KL p95 below 0.05. Zero keys were decoded for ranking — all candidate ranking used compressed-domain quantized key scores.

## Unsafe claim

Do not claim this is a throughput win. The Python implementation is quality-only — 116 seconds for the compressed forward pass. The Rust compressed attention bench shows 142 queries/s for the compressed scoring path (no model forward), which is the starting point for real throughput measurement.

## What was built

1. `poly-kv/scripts/adaptive_budget.py` — Layer/head adaptive budget allocator with fragility-based scaling
2. `poly-kv/scripts/test_adaptive_budget.py` — 7 unit tests for the allocator
3. `poly-kv/scripts/fib_gram_scorer.py` — Python Gram-table prepared-query scorer (matches Rust FibScorer)
4. `poly-kv/scripts/test_fib_gram_scorer.py` — 4 unit tests for the scorer
5. `poly-kv/scripts/compressed_attention_forward_ppl.py` — Extended with:
   - `--adaptive-budget` / `--adaptive-budget-heads` flags
   - `--fragility-file` for loading per-layer fragility data
   - `--scorer quantized|fib-gram` for choosing scoring method
   - `--sample-heads` / `--sample-positions` for sampled attention stats
   - Per-layer and per-head stats collection in `AttentionStats`
6. `poly-kv/examples/compressed_attention_bench.rs` — Rust throughput benchmark
7. `compressed-scorer/src/adaptive_budget.rs` — Rust port of adaptive budget allocator (BudgetConfig, LayerBudgets, HeadBudgets, learn_budgets, default_fragility_256tok). no_std + ESP32 compatible. 7 tests.

## Verification matrix

- `python3 -m py_compile` on all 5 Python files: PASS
- `python3 -m pytest poly-kv/scripts/test_adaptive_budget.py`: 7/7 PASS
- `cargo check -p compressed-scorer --no-default-features --features no_std`: PASS
- `cargo check --manifest-path poly-kv/Cargo.toml`: PASS
- `cargo test --manifest-path poly-kv/Cargo.toml`: PASS (all suites)
- `cargo test -p compressed-scorer`: PASS
- `cargo test -p fib-quant`: PASS (2+4 tests)
- `cargo check -p gpu-backend`: PASS
- `cargo check -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'`: PASS
- `cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc`: PASS
- `cargo build --release --example compressed_attention_bench --manifest-path poly-kv/Cargo.toml`: PASS
- `cargo run --release --example compressed_attention_bench`: PASS (64 queries, 0 decoded, 142 qps)
- `cargo test -p compressed-scorer` (with adaptive_budget): 18+1+1 PASS (7 new adaptive_budget tests)
- 512-token adaptive gate: FAIL (delta_ppl -3.17%, logit_cosine 0.981 — needs 512-token fragility data)

## Next steps

1. fib-gram adaptive gate (Phase 2.3) — COMPLETE, FAIL (Python PQ approximation produces garbage rankings; delta_ppl 28400%, logit_cosine -0.1545). The Rust FibScorer with rotation + shared codebook is needed, not a Python PQ reimplementation. The simple 4-bit quantized scorer is better for this use case because it preserves full vector structure.
2. 512-token adaptive gate — COMPLETE, FAIL even with 512-token fragility data. The 256-token fragility defaults gave delta_ppl -3.17%, logit_cosine 0.981. The 512-token fragility map gave delta_ppl -3.43%, logit_cosine 0.987, mean_k 106.6. The fundamental problem is that 4-bit quantized scoring for layer 0 collapses with context length (cosine_p05: 0.922 at 128tok, 0.712 at 256tok, 0.390 at 512tok). No budget allocation can fix this — layer 0 needs a better scorer or full-precision fallback. The adaptive budget correctly identifies and compensates for fragile layers, but the scoring quality itself is the bottleneck. Receipts: quant-eval/results/remote-msi/smollm2-1.7b-wikitext2-forward-512/adaptive_receipt.json (v1), adaptive_v2_receipt.json (v2).
3. Port adaptive budget allocator to Rust — COMPLETE. compressed-scorer/src/adaptive_budget.rs with BudgetConfig, LayerBudgets, HeadBudgets, learn_budgets(). 7 tests, no_std + ESP32 pass. Re-exported from lib.rs.
4. CUDA/CPU page scorer kernels — EXISTING. gpu-backend/src/page_scorer.rs already has score_fib_gram_pages_cpu (CPU scalar) and score_fib_gram_pages (auto-dispatch to CUDA when available). The CPU path is the fallback. CUDA path requires PTX kernel file (kernels/combined.ptx).
5. Learn budget allocation from per-layer/head drift data — COMPLETE (Rust). learn_budgets() in adaptive_budget.rs iteratively reduces max_k to hit a target mean k. The Python adaptive_budget.py also has the formula path. Next: learn from actual 512-token diagnostic data instead of 256-token defaults.
6. For Gram-table scoring: use actual Rust FibScorer via PyO3 binding or SSH-side Rust process, not Python PQ — NOT STARTED. The Rust FibScorer exists and is tested in fib-quant/src/scoring.rs. Needs a thin Python binding (PyO3 or subprocess) to call from the forward harness.