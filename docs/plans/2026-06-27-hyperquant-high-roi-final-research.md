# HyperQuant High-ROI Additions — Final Research Pass (2026-06-27)

## Bottom line

The highest ROI already landed in the current dirty tree: `hyperquant` moved from seed crate to pluggable, receipt-bearing primitive layer, and `quant-eval` now has governed policy receipts plus a synthetic retrieval/latency benchmark. The next high-ROI work is no longer “add more primitives”; it is evidence hardening and calibration.

Do next:
1. Real-corpus retrieval benchmark (`BEIR scifact` first) using `quant-eval` receipts.
2. Cross-codec comparison against `turbo-quant` / `fib-quant` on the same fixtures.
3. Calibrated HyperQuant policy tables in `quant-governor` from measured receipts, replacing fixed default degradation guesses.
4. Integrate RHT into the actual HyperQuant quantization pipeline as an opt-in profile, not just a standalone helper.
5. Implement E8 only after items 1-4, because E8 without real workload receipts creates paper-parity overclaim risk.

## Verified current state

Local files inspected:
- `/home/sikmindz/Coding/Libraries/hyperquant/README.md`
- `/home/sikmindz/Coding/Libraries/hyperquant/Cargo.toml`
- `/home/sikmindz/Coding/Libraries/hyperquant/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/hyperquant/src/lattice.rs`
- `/home/sikmindz/Coding/Libraries/hyperquant/src/rice.rs`
- `/home/sikmindz/Coding/Libraries/hyperquant/src/rht.rs`
- `/home/sikmindz/Coding/Libraries/hyperquant/src/codec.rs`
- `/home/sikmindz/Coding/Libraries/quant-eval/src/hyperquant_eval.rs`
- `/home/sikmindz/Coding/Libraries/quant-eval/src/hyperquant_retrieval.rs`
- `/home/sikmindz/Coding/Libraries/quant-governor/src/policy.rs`
- `/home/sikmindz/Coding/Libraries/quant-governor/src/decision.rs`

Current implemented surface:
- Z1 scalar lattice quantization.
- A2 triangular lattice quantization.
- D4 checkerboard lattice quantization.
- E8 named but explicitly unsupported.
- Result-bound receipts and claim boundary.
- Rice/zigzag bitstream and best-k byte profiling.
- Seeded CPU-local RHT helper.
- `quant-codec-core` compatibility adapter behind `compat` feature.
- `quant-governor` knows `CodecProfile::Hyperquant` and routes embedding policy by preset/admissibility/budget.
- `quant-eval` has:
  - primitive HyperQuant profile eval with Rice byte accounting;
  - governed HyperQuant admission receipts;
  - synthetic clustered retrieval/latency benchmark.

## External evidence basis

### arXiv:2606.23406

Paper: `HyperQuant: A Rate-Distortion-Optimal Quantization Pipeline for Large Language and Diffusion Models`

arXiv metadata fetched from arXiv API:
- ID: `2606.23406v1`
- Published: 2026-06-22
- Authors: Yuval Domb, Hadar Sackstein, Tomer Solberg
- Categories: `cs.LG`, `cs.AI`

Paper abstract claims:
- HyperQuant combines:
  1. per-tile Randomized Hadamard Transform;
  2. low-dimensional optimal lattice VQ: E8, D4, A2, or Z;
  3. bit-stripping + variable-length Rice coding;
  4. KV-cache bias correction for unbiased inner products.
- Reported paper claims include 3-5 bps weight wins vs HIGGS, KV cache wins vs TurboQuant/OCTOPUS down to 1.7 bps, ~3.9x weight compression, ~3.79x KV compression at 4 bps on H100.

Important: these are external paper claims, not local `hyperquant` claims.

### Official implementation

GitHub API evidence:
- Repo: `moonmath-ai/HyperQuant`
- URL: `https://github.com/moonmath-ai/HyperQuant`
- Language: Python
- License: MIT
- Stars at query time: 9
- Forks: 0
- Created: 2026-06-23
- Last pushed: 2026-06-23

Rust ecosystem evidence:
- GitHub API search `hyperquant language:Rust`: total count 0.
- crates.io API `hyperquant`: current crate exists as RecursiveIntell crate.
- crates.io versions:
  - `0.1.0`, created 2026-06-26, 18 downloads at query time.
  - `0.1.1`, created 2026-06-27, newest/max stable at query time.

Interpretation: first-mover Rust ecosystem position is real, but external traction is still tiny and too early to use as validation.

## New benchmark receipt from this pass

Command run:

```bash
cargo run -p quant-eval --example hyperquant_retrieval_benchmark
```

Config:
- dim: 32
- docs: 128
- queries: 12
- clusters: 8
- top_k: 5
- seed: 11
- scale: 16.0
- lattice: D4

Measured result:
- build latency: 18,502,287 ns
- raw p50: 291,805 ns
- raw p95: 409,265 ns
- HyperQuant p50: 311,072 ns
- HyperQuant p95: 411,960 ns
- recall@K: 0.6000
- top-K overlap: 0.4646
- NDCG@K: 0.6111
- exact top-1 recovery in HyperQuant top-K: 0.7500
- score error mean: 0.009753
- score error p95: 0.018735
- rank drift mean: 2.5833
- rank drift p95: 8
- raw bytes: 16,384
- HyperQuant estimated bytes: 1,977
- compression ratio: 8.2873x
- passed: false
- blocker: `exact_rerank_recovery_at_1 0.75 < 0.8`

Interpretation:
- Storage ROI is strong on the synthetic fixture.
- Retrieval quality is not admissible under current threshold.
- Therefore the next ROI is benchmark/calibration, not widening runtime use.

## Ranked high-ROI backlog

### P0 — Real-corpus retrieval benchmark receipts

Why high ROI:
- Synthetic retrieval is now built and already shows a useful blocker.
- A real corpus/qrels benchmark is the evidence gap that determines whether HyperQuant is worth using for semantic-memory/vector retrieval.
- Without this, any public retrieval claim is weak.

Implementation target:
- Add BEIR `scifact` builder under `quant-eval/tools/hyperquant_corpus/`.
- Use local embeddings, preferably Ollama `all-minilm` or an existing Candle embedding path if already stable.
- Emit a binary/JSON fixture and a receipt under `quant-eval/docs/codex-runs/<date>/`.
- Compare raw exact search vs HyperQuant candidate set + exact rerank.

Acceptance gates:
- Receipt includes corpus name/version, model, doc/query counts, qrels count.
- Metrics: recall@1/5/10, top-K overlap, exact-rerank recovery@1, rank drift, score error, raw/HQ latency percentiles, byte ratio.
- Starting pass threshold: `top_k_overlap >= 0.30` and `exact_rerank_recovery_at_1 >= 0.80`.
- If it fails, document the kill honestly.

Effort: 0.5-1 day if reusing the real-corpus vector-codec recipe.

### P1 — Cross-codec comparison harness: HyperQuant vs turbo-quant vs fib-quant

Why high ROI:
- HyperQuant’s local value is not “it exists”; it is whether it beats or complements the existing RecursiveIntell quantizers under identical fixtures.
- This closes the internal architecture question: new codec, sidecar backend, or dead end.

Implementation target:
- Add comparison API in `quant-eval`, not in the runtime crates.
- Use the same fixture for:
  - raw baseline;
  - HyperQuant D4/A2/Z1;
  - `turbo-quant` sidecar/packed profiles;
  - `fib-quant` if shape and dependency cost are acceptable.
- Emit a single table receipt.

Acceptance gates:
- Same vectors, same queries, same qrels/exact baseline.
- Per-codec byte accounting and latency separated.
- Claim boundary says benchmark fixture only.

Effort: 1-2 days depending on `fib-quant` adapter friction.

### P2 — Calibrated `quant-governor` policy tables

Why high ROI:
- `quant-governor` currently routes HyperQuant using conservative fixed degradation and rough ratio assumptions.
- Policy should be receipt-calibrated: content type + lattice + scale + corpus class -> admissibility.

Implementation target:
- Add a calibration receipt type to `quant-eval` or `quant-governor`.
- Store profile summaries such as:
  - `lattice=D4`, `scale=16`, `dim=384`, `corpus=scifact`, `exact_rerank_recovery=...`, `compression_ratio=...`.
- `quant-governor` consumes calibration summaries for routing instead of hardcoded `Hyperquant => 0.07` degradation / `2.75x` ratio.

Acceptance gates:
- If no calibration receipt exists for a route, HyperQuant remains experimental or blocked except best-effort/storage-only paths.
- Strict/critical paths still raw/fallback.
- Tests assert no silent widening.

Effort: 0.5-1 day after P0/P1 receipts exist.

### P3 — Integrate RHT into HyperQuant quantization profiles

Why high ROI:
- Current `rht_tile` is a useful primitive but not part of `HyperQuantConfig::quantize`.
- The paper’s quality story depends on RHT making distributions more lattice-friendly.
- The failed retrieval threshold may improve with RHT preconditioning.

Implementation target:
- Add `HyperQuantPipelineConfig` or `Preconditioner::None | Rht { tile_dim, seed }` without breaking existing API.
- Apply RHT before lattice quantization and inverse RHT after reconstruction for symmetric encode/decode.
- Receipt records preconditioner, seed digest, and tile dim.

Acceptance gates:
- Roundtrip shape preserved.
- RHT pipeline deterministic.
- Norm roughly preserved before quantization.
- Retrieval benchmark has an A/B receipt: D4 vs RHT+D4.

Effort: 1 day.

### P4 — Scale/rate sweep and auto-select profiles

Why high ROI:
- Current scale is caller-chosen. That makes benchmark numbers arbitrary.
- Rate-distortion selection is core to the paper and useful even without E8/CUDA.

Implementation target:
- Add `sweep_hyperquant_scale` in `quant-eval` over lattice, scale range, and optionally RHT tile size.
- Produce Pareto frontier: bytes vs MSE / retrieval quality / latency.
- Export a recommended profile for a target quality floor.

Acceptance gates:
- Deterministic sweep receipt.
- Reports dominated profiles and selected profile rationale.
- Governor can consume the selected profile only through receipt-backed admission.

Effort: 0.5-1 day.

### P5 — E8 nearest-lattice quantization

Why not first:
- E8 is algorithmically attractive and paper-aligned, but it is not the current bottleneck.
- The current D4 synthetic retrieval receipt fails exact top-1 recovery threshold. More lattice prestige will not fix the evidence gap by itself.

When to do it:
- After P0/P3/P4 show D4/RHT+D4 is close enough that E8 could plausibly cross quality thresholds.

Implementation target:
- Add exact or bounded nearest E8 implementation with tests on known E8 lattice points.
- Keep E8 explicit unsupported until tests pass.
- Add E8 to quant-eval profile matrix only after local receipt.

Acceptance gates:
- Known E8 points reconstruct exactly / near-zero MSE.
- Handles tails deterministically.
- Does not regress D4/A2/Z1 tests.
- E8 benchmark receipt beats or explains D4.

Effort: 1-3 days depending on implementation route.

### P6 — Bit-stripping beyond generic Rice

Why medium ROI:
- Rice exists, but paper-specific bit stripping is not implemented.
- It can improve bytes, but quality/admissibility is the bigger blocker right now.

Implementation target:
- Analyze lattice code streams for pinned/highly predictable bits after RHT.
- Add reversible structural stripping metadata before Rice coding.

Acceptance gates:
- Exact code roundtrip.
- Byte reduction vs Rice-only on fixture.
- Receipt records stripped fields and reversibility proof.

Effort: 1-2 days.

### P7 — Runtime integration into semantic-memory as experimental compressed candidate backend

Why defer:
- `semantic-memory-mcp` already has TurboQuant config fields.
- HyperQuant should not enter semantic memory until P0/P1/P2 produce admission receipts.

Implementation target:
- Add config fields only after policy calibration:
  - `hyperquant_enabled`
  - `hyperquant_lattice`
  - `hyperquant_scale/profile_receipt`
- Raw vectors remain authority.
- Exact rerank remains required.

Acceptance gates:
- Default off.
- Exact fallback on missing receipt.
- No truth-bearing route depends only on HyperQuant approximate candidates.

Effort: 1-2 days after evidence gates.

### P8 — CUDA / model-layer / KV bias-correction parity

Why low immediate ROI:
- This is where paper parity lives, but it is the biggest overclaim trap.
- Requires model harnesses, GPU kernels, and PPL/KV evals.

Do only when:
- The Rust primitive + evidence path already justifies it.
- You have a concrete local model eval target.

Acceptance gates:
- Model-level baseline and HyperQuant run under the same prompt/corpus.
- PPL or task metric receipt.
- No paper superiority claims without reproduction.

Effort: multi-day to multi-week.

## Keep / kill decisions

Keep now:
- HyperQuant primitive crate.
- D4/Rice/RHT/compat foundation.
- Governed receipt path.
- Synthetic benchmark as a regression and smoke tool.

Do not widen runtime use yet:
- The synthetic retrieval benchmark fails exact top-1 recovery threshold.
- Use the failure as a useful guardrail, not a setback.

Do not spend next effort on:
- README marketing.
- E8 before real corpus receipts.
- CUDA/HF/model claims.
- Direct semantic-memory runtime integration.

## Final recommendation

The next implementation should be P0: real-corpus retrieval benchmark receipts. It is the narrowest work that can change the decision from “interesting primitive” to “admissible backend candidate” or kill the retrieval route cleanly. Everything else should queue behind that evidence.
