# HyperQuant stack integration matrix — 2026-07-05

## Bottom line

HyperQuant is useful in your stack, but not as the first production embedding codec today.

The live Scifact comparison says simple int8 beats current HyperQuant Z1/A2 for embedding retrieval quality and compression ratio. HyperQuant should be pursued as an evidence-bearing lattice quantization research path and as a future governed codec backend, not as a replacement for semantic-memory's authoritative f32 vectors or as a hosted-model context-window extender.

Highest ROI split:

1. Product path now: add/use simple int8 or existing turbo/poly-kv candidate sidecars with exact f32 rerank for semantic-memory/RAG.
2. HyperQuant path now: make HyperQuant pluggable and paper-closer: quant-codec-core adapter, real Rice/zigzag byte accounting, D4, RHT, then re-run BEIR/Scifact comparisons.
3. Research path next: integrate HyperQuant into compressed-scorer and poly-kv only after it can score compressed-domain candidates or produce a KV-cache receipt against exact attention/reference.

## Evidence checked

Live local checks:

- `/home/sikmindz/Coding/Libraries` branch: `feat/full-integration`.
- Current HyperQuant comparison commit: `88f51f5 bench: compare hyperquant scifact baselines`.
- `cargo metadata --no-deps --format-version 1` confirms current local dependency graph:
  - `quant-eval -> hyperquant`
  - `semantic-memory -> poly-kv, quant-governor, scr-runtime-compression, turbo-quant`
  - `compressed-scorer -> fib-quant, turbo-quant`
  - `scr-runtime-compression -> fib-quant, quant-governor, turbo-quant`
  - no current `semantic-memory -> hyperquant`
  - no current `compressed-scorer -> hyperquant`
  - no current `scr-runtime-compression -> hyperquant`
- `hyperquant/src/lib.rs` exports only primitive lattice quantization modules: `error`, `lattice`, `receipt`, `scalar`.
- `hyperquant/README.md` explicitly forbids claims of paper parity, model-quality preservation, production readiness, CUDA, HuggingFace integration, superiority over GPTQ/AWQ/TurboQuant/FibQuant, and production semantic-memory/KV use.
- `quant-eval` now has stored BEIR/Scifact all-minilm receipts and a hostile codec comparison receipt.
- `semantic-memory` already has receipt fields for approximate derived candidates and exact rerank.
- `context-governor` already owns receipt-backed prompt/context compaction with exact fallback references.
- Hermes has a pluggable `ContextEngine` surface; hosted-provider KV-cache injection is not available.

Prior semantic-memory recall checked with `sm_search` only:

- Prior finding: HyperQuant is best treated as a receipt-bearing global rate-distortion allocator/compression governor, not as direct semantic-memory search replacement.
- Prior finding: Hermes context compaction path should preserve exact fallback and disclose lossy compression via receipts.
- Prior finding: KV-cache compression belongs to local inference crates, not hosted Hermes compaction.
- Prior finding: compressed-cache next frontier is exact-reference attention harness first, then progressive coarse-to-fine scoring, query-aware sparse attention, RoPE-aware bit allocation, head/layer policy, and hardware kernels.

External paper context checked via arXiv API:

- KIVI: `2402.02750` — asymmetric 2-bit KV-cache quantization.
- KVQuant: `2401.18079` — KV cache quantization for very long context inference.
- QServe: `2405.04532` — W4A8KV4 quantization/system co-design.
- PyramidKV: `2406.02069` — dynamic KV-cache compression via information funneling.
- Recent 2026 KV eviction/compression papers: `2605.08840`, `2605.09649`.
- PolyKV paper surfaced as `2604.24971`; local repo also has `poly-kv` implementation/docs. Treat as internal/local evidence unless independently verified externally.

## Current measured comparison

Stored receipt:

- `/home/sikmindz/Coding/Libraries/quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_RECEIPT.json`

Dataset:

- BEIR Scifact test split
- 5,183 docs
- 300 test queries with positive qrels
- local Ollama `all-minilm:latest`
- top_k=10, candidate_k=40, scale=8.0

| Profile | Compression | R@10 | Top-K overlap | Exact-rerank recovery@1 | Meaning |
|---|---:|---:|---:|---:|---|
| scalar_i8_per_vector_scale | 3.9588x | 0.7800 | 0.9933 | 0.8767 | best current embedding baseline |
| scalar_i8_global_symmetric | 4.0000x | 0.7733 | 0.9595 | 0.8767 | best simple baseline |
| hyperquant_A2_scale_8 | 2.0000x | 0.7582 | 0.5910 | 0.8733 | passes gate, but loses to int8 |
| hyperquant_Z1_scale_8 | 2.0000x | 0.7549 | 0.5514 | 0.8667 | passes gate, weaker than A2 |
| sign_binary_1bit | 32.0000x | 0.7238 | 0.5652 | 0.8567 | negative/high-compression control |

Interpretation:

- Current HyperQuant A2/Z1 are viable candidate generators under this receipt.
- Current HyperQuant A2/Z1 are not the best immediate embedding-compression path.
- int8 baselines should be the production reference until HyperQuant has Rice/bitstream/D4/RHT and proves a new advantage.

## Integration/use matrix

### 1. quant-eval benchmark substrate

Status: implemented and high value.

How HyperQuant combines:

- `quant-eval` imports `hyperquant` and evaluates Z1/A2 against exact f32 retrieval.
- This is the correct first integration layer because it turns experiments into receipts before runtime adoption.

What it enables:

- BEIR/Scifact candidate gates.
- Baseline comparison against int8/sign controls.
- Regression detection for future D4/RHT/Rice changes.

ROI:

- Keep and expand.
- Add FIQA/NFCorpus/TREC-COVID after D4/Rice if results improve.

Do not overclaim:

- quant-eval evidence is retrieval candidate evidence, not model-quality/KV-cache evidence.

### 2. semantic-memory derived candidate backend

Status: architecture-ready, not wired to HyperQuant.

How HyperQuant could combine:

- Build HyperQuant-derived vector artifacts from semantic-memory's authoritative f32 embedding snapshot.
- Use HyperQuant only to generate candidate IDs.
- Load authoritative f32 embeddings for exact rerank before returning final results.
- Emit `VectorSearchReceiptV1` fields: `candidate_backend`, `codec_family`, `codec_profile_digest`, `artifact_generation_id`, `approximate_candidate_count`, `exact_rerank_count`, `fallback`.

Good use:

- optional experimental backend: `hyperquant_candidate_then_exact_f32`.
- disabled by default unless a stored benchmark receipt admits the profile.

Bad use:

- replacing f32 source embeddings;
- returning HyperQuant-ranked results without exact rerank;
- compressing facts/claims/evidence text directly.

ROI:

- Medium after adapter/Rice/D4.
- Low right now because int8 is better for immediate embedding compression.

Acceptance gate:

- Same Scifact gate plus at least one semantic-memory local corpus replay.
- fallback must be receipted if artifacts are stale/missing/incomplete.

### 3. context-governor / Hermes context compaction allocator

Status: conceptually strong; HyperQuant math only, not current crate API.

How HyperQuant could combine:

- Treat context compaction as a rate-distortion allocation problem:
  - rate = prompt-token cost of keeping/summarizing/archive-only;
  - distortion = expected harm of omission/compression;
  - authority = must-preserve exact weight;
  - fallback = whether exact content is recoverable from receipt store/semantic-memory;
  - output = keep verbatim, summarize, archive, omit, quarantine.

Current stack fit:

- `context-governor` already classifies items and emits exact fallback receipts.
- Hermes already has `ContextEngine` plugin interface.
- Agent-memory-kits already package context-governor rules/MCP/receipts.

Where HyperQuant helps:

- as a policy/allocator inspiration or future small optimization helper;
- not as a text compressor;
- not as a hosted-model KV-cache extender.

ROI:

- High if framed as `context-governor` allocator/eval work.
- Low if trying to literally use `hyperquant::quantize_a2()` on raw chat text.

Acceptance gate:

- replay long sessions against built-in Hermes compressor;
- measure answerability, stale-task avoidance, exact fallback recovery, token savings, and unsafe omission rate.

### 4. compressed-scorer adapter

Status: best technical bridge after HyperQuant grows beyond reconstruction-only ranking.

Current stack:

- `compressed-scorer` defines `CompressedScorer` and `ProgressiveCompressedScorer`.
- It already supports/fits `fib-quant` and `turbo-quant` style compressed-domain scoring.
- It has `AttentionCache` that scores compressed keys and decodes only top-k values.

How HyperQuant could combine:

- Add `HyperQuantScorerAdapter` implementing `CompressedScorer`.
- Prepared query: quantized/rotated query profile.
- Compressed docs: HyperQuant codes plus scale/profile digest.
- Score path: approximate dot/cosine from codes without reconstructing full vector.
- Decode path: reconstruct only top-k.

Key issue:

- current HyperQuant path reconstructs vectors, so it does not yet give the `compressed-scorer` hot-path advantage.
- Need code-domain scoring or lookup tables, plus measured error bounds.

ROI:

- High after `quant-codec-core` adapter + Rice/D4/RHT.
- Medium now as a thin adapter only, because it would mostly decode/reconstruct and not beat int8.

Acceptance gate:

- compressed-domain score error p95 vs exact;
- Scifact candidate gate;
- attention-cache top-k output cosine/MSE if used for KV.

### 5. poly-kv / local inference KV-cache

Status: strategically relevant, but not a current HyperQuant integration.

How HyperQuant could combine:

- Cold shared tier: HyperQuant D4/E8/Rice as alternative to fib-quant for stable shared context blocks.
- Hot agent tier: HyperQuant A2/Z1/RHT as alternative to turbo-quant only if it can preserve near-lossless hot-shell quality.
- Query-aware sparse attention: use HyperQuant/compressed-scorer to select top-k cached keys, decode only selected values.
- Receipts: each compressed KV path must emit profile digest, layer/head role, decoded-count, output cosine/MSE, fallback behavior.

External research context:

- KIVI/KVQuant/QServe/PyramidKV show KV cache compression is a real field.
- Those papers validate the problem, not your implementation.

Current risk:

- HyperQuant has no model-forward PPL/logit evidence.
- Current Scifact embedding evidence does not prove KV-cache quality.

ROI:

- Medium/long-term high.
- Not the next immediate move unless the goal is inference-engine research.

Acceptance gate:

- compressed-cache attention benchmark first;
- exact full-attention reference;
- output cosine/MSE, top-k overlap, decoded value count, bytes loaded/query, latency;
- then real LLM PPL/logit drift.

### 6. quant-governor policy layer

Status: correct routing home.

How HyperQuant could combine:

- Add `CodecProfile::HyperQuantA2`, `HyperQuantD4`, etc. only after adapter exists.
- Treat HyperQuant as experimental/default-deny unless a receipt admits a specific corpus/model/use.
- Route by content type:
  - raw for evidence-critical;
  - int8/turbo for embedding candidate search;
  - fib/poly-kv for cold shared context;
  - HyperQuant for admitted research profiles only.

ROI:

- High as governance glue after the adapter lands.
- Low before a real backend exists.

Acceptance gate:

- policy decisions must include receipt references and exact fallback requirements.

### 7. scr-runtime-compression dispatch

Status: direct extension point, but currently only names Turbo/Fib/Polar/QJL/Uncompressed.

How HyperQuant could combine:

- Add `CodecId::HyperQuant`.
- Dispatch encode/decode through hyperquant adapter.
- Make `requires_exact_fallback()` true.
- Preserve lossy disclosure in every runtime receipt.

ROI:

- Medium.
- Needs `quant-codec-core` adapter first, or this becomes ad hoc glue.

Acceptance gate:

- encode/decode roundtrip tests;
- fallback receipt test;
- semantic-memory derived candidate path test if wired.

### 8. agent-memory-kits / Pro release-gate / claim-ledger

Status: good for receipts and product proof; not a runtime compression target.

How HyperQuant could combine:

- Package benchmark receipts as proof packets.
- Add release-gate checks: no HyperQuant claim promotion unless Scifact/baseline receipt passes and claim boundary is present.
- Add tool receipt fields when a HyperQuant benchmark/build runs.

ROI:

- High for credibility.
- Does not improve model performance directly.

Acceptance gate:

- release-gate rejects unsafe claims like “HyperQuant beats int8” given current receipt.
- release-gate promotes narrow claim: “passes Scifact candidate gate but loses to int8 baseline.”

### 9. semantic-memory / context-governor hybrid memory compaction

Status: most product-aligned non-KV use.

How HyperQuant could combine:

- Not compressing text.
- Not replacing summaries.
- Use vector compression/importance scoring to decide what archived context should stay hot vs cold.
- Keep exact fallback in context-governor receipt store and semantic-memory document/chunk store.

Possible pipeline:

1. Context-governor classifies transcript items.
2. semantic-memory archives durable facts/docs/chunks.
3. HyperQuant/int8/turbo sidecar stores cheap candidate vectors for archived items.
4. Query retrieves candidates.
5. exact f32 rerank + receipt expansion returns source text.
6. final prompt includes only exact/source-bounded context.

ROI:

- High if implemented with int8 first and HyperQuant as experimental alternative.

### 10. model weights / diffusion / paper-parity path

Status: not stack-ready yet.

How HyperQuant could combine:

- Implement the actual paper pipeline: per-tile RHT, lattice VQ, structural bit stripping, Rice coding, KV bias correction.
- Apply to weight or activation tensors in a controlled local inference harness.

ROI:

- High prestige, low immediate product ROI.
- Large implementation/repro burden.

Acceptance gate:

- reproduce model-level quality metrics on a real model;
- compare to GPTQ/AWQ/int8/fp16 where appropriate;
- do not claim paper parity until this exists.

### 11. ESP32 / edge path

Status: possible later, not current best use.

How HyperQuant could combine:

- only after no_std/alloc-friendly codec path and packed byte format exist;
- useful as tiny code-domain selector or embedding/context sidecar, not large model compression on ESP32 today.

ROI:

- Low now.
- ESP32 matmul/fixed-point/SIMD remains higher ROI than HyperQuant integration.

### 12. Knowledge/runtime/agent graph provenance consumers

Status: receipt propagation only.

How HyperQuant could combine:

- carry `codec_family`, profile digest, generation ID, fallback status, and exact-rerank flag through `knowledge-runtime`, `llm-tool-runtime`, `agent-graph`, and claim systems.
- do not make those crates depend directly on HyperQuant.

ROI:

- Medium after runtime backend exists.
- Important for architecture cleanliness.

## Highest ROI implementation order

### Phase 0 — do not skip

Keep current claim boundary in public docs:

- HyperQuant passes a Scifact candidate gate.
- HyperQuant currently loses to simple int8 baselines for embedding retrieval.
- HyperQuant is research/evidence-bearing, not the first production embedding codec.

### Phase 1 — make HyperQuant pluggable and honestly measured

1. Add `quant-codec-core` adapter behind a feature.
2. Add Rice/zigzag bitstream and real byte accounting.
3. Add D4 nearest-lattice implementation.
4. Add CPU RHT helper.
5. Re-run Scifact comparison against int8/sign baselines.

Why first:

- These close the current paper/stack gap without unsafe product claims.
- They make HyperQuant eligible for governance/runtime routing.

### Phase 2 — add stack integration behind gates

1. Add `HyperQuantScorerAdapter` to `compressed-scorer`.
2. Add `CodecId::HyperQuant` to `scr-runtime-compression`.
3. Add experimental `quant-governor` policy profile.
4. Add semantic-memory derived candidate backend only with exact rerank.

Gate:

- HyperQuant must beat or meaningfully differ from int8 on at least one real metric before it becomes product-default anywhere.

### Phase 3 — context compaction allocator

1. Add context-governor eval comparing deterministic allocator modes.
2. Treat HyperQuant/rate-distortion as allocation math, not text compression.
3. Preserve exact fallback and boundary-audit summaries.

Gate:

- better answerability/context survival than Hermes built-in compressor on replay tasks.

### Phase 4 — local inference/KV research

1. Build compressed-cache attention harness using `compressed-scorer::AttentionCache` shape.
2. Add HyperQuant only if it supports compressed-domain scoring.
3. Compare against full attention, int8, turbo/fib, and top-k sparse baselines.
4. Only then attempt model PPL/logit drift.

Gate:

- full-attention reference receipt, not just vector reconstruction MSE.

## Kill / keep decisions

Keep:

- HyperQuant as a research primitive.
- quant-eval Scifact gate and hostile baseline comparison.
- exact-rerank-only integration rule.
- receipts/proof packets around every compression claim.

Use now over HyperQuant:

- int8 per-vector scaling for immediate embedding/RAG compression.
- existing semantic-memory usearch/f32 for authoritative retrieval.
- turbo/poly-kv paths where they already have integration receipts.

Do not do now:

- direct semantic-memory production backend from current HyperQuant Z1/A2.
- raw text compression with HyperQuant.
- hosted Hermes KV-cache/context-window extension claims.
- E8 before D4/Rice/RHT unless deliberately chasing prestige over ROI.
- model-weight/KV claims without PPL/logit/full-attention receipts.

## Safe public wording

Safe:

> HyperQuant is an experimental Rust lattice-quantization primitive with receipt-backed BEIR/Scifact candidate-gate evidence. Current Z1/A2 pass the candidate gate, but simple int8 baselines outperform them for embedding retrieval, so HyperQuant is best positioned as a research path toward governed lattice compression rather than today's production embedding codec.

Unsafe:

- HyperQuant beats int8.
- HyperQuant is production-ready.
- HyperQuant extends hosted model context windows.
- HyperQuant preserves LLM quality/KV-cache behavior.
- HyperQuant is paper-parity.
- HyperQuant should replace semantic-memory f32 vectors.

## Files inspected

- `/home/sikmindz/Coding/Libraries/hyperquant/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/hyperquant/README.md`
- `/home/sikmindz/Coding/Libraries/quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_RECEIPT.json`
- `/home/sikmindz/Coding/Libraries/quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_SUMMARY.md`
- `/home/sikmindz/Coding/Libraries/semantic-memory/Cargo.toml`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/types.rs`
- `/home/sikmindz/Coding/Libraries/compressed-scorer/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/compressed-scorer/src/trait_def.rs`
- `/home/sikmindz/Coding/Libraries/compressed-scorer/src/attention_cache.rs`
- `/home/sikmindz/Coding/Libraries/compressed-scorer/src/adaptive_budget.rs`
- `/home/sikmindz/Coding/Libraries/poly-kv/README.md`
- `/home/sikmindz/Coding/Libraries/docs/provekv-derived-candidate-architecture.md`
- `/home/sikmindz/Coding/Libraries/scr-runtime-compression/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/quant-governor/src/lib.rs`
- `/home/sikmindz/Coding/agent-memory-kits/README.md`
- `/home/sikmindz/.hermes/hermes-agent/agent/context_engine.py`
- `/home/sikmindz/.hermes/hermes-agent/agent/context_compressor.py`
- `/home/sikmindz/.hermes/hermes-agent/agent/conversation_compression.py`

## Verification commands run

```bash
date -Iseconds
cargo metadata --no-deps --format-version 1
```

Also queried arXiv API for targeted KV-cache/context/vector-compression paper context; used only as directional research context, not as proof about local implementation.
