# Compressed-Domain Scoring — Use Case Catalog

**Date:** 2026-06-30  
**Author:** Research pass  
**Scope:** Every potential application, integration point, and use case for `PerDimScorer`, `AttentionCache`, `CompressedScorer` trait, and related compressed-domain scoring infrastructure across the RecursiveIntell codebase and externally.

---

## Architecture Summary

### Core Crates

| Crate | Role | Key Types |
|-------|------|-----------|
| `compressed-scorer` | Codec-agnostic compressed-domain scoring | `CompressedScorer` trait, `PerDimScorer`, `AttentionCache<S>`, `CompressedWorkingSet<S>`, `CandidateList`, `AdaptiveBudget` |
| `scr-runtime-compression` | Runtime integration adapter | `CompressedScorerAdapter<S>`, `CodecId` (TurboQuant, FibQuant, Polar, Qjl, PerDim, Uncompressed), `CompressedSearchPath` |
| `turbo-quant` | Polar/QJL vector compression | `TurboQuantizer`, `TurboCode`, `PolarCode`, `QjlSketch` |
| `fib-quant` | Radial-angular codebook quantization | `FibQuantizer`, `FibScorer`, `FibCodeV1`, Gram-table scoring |
| `quant-governor` | Policy-driven codec selection | `GovernancePolicy`, `CodecProfile`, `CodecDecision` |
| `hyperquant` | Lattice quantization (RHT + Rice) | `HyperQuantConfig`, `LatticeKind`, RHT transforms |
| `semantic-memory` | Hybrid semantic search (SQLite + FTS5 + usearch) | `MemoryStore`, search pipeline, `turbo-quant-codec` feature |
| `poly-kv` | Shared compressed KV-cache pool | Two-tier codec pool (fib-quant cold + turbo-quant hot) |

### Performance Characteristics (from roadmap)

| Metric | Value | Context |
|--------|-------|---------|
| Memory compression | 12–57x vs fp16 | PerDim 4-bit to 8-bit |
| Cosine quality (8-bit) | >0.99999 | 256-token gate |
| Cosine quality (4-bit) | >0.99 | 512-token gate |
| PPL delta | +0.10% (256tok), -0.50% (512tok) | Quality gates pass |
| GPU speed vs cuBLAS | 4–6x slower | GTX 1070, no Tensor Cores |
| ESP32-S3 compilation | ✅ Passes | `cargo +esp check` |
| no_std | ✅ 17 tests pass | `--no-default-features --features no_std` |

### Key Insight

The per-dim scorer **loses on GPU** (cuBLAS is unbeatable for small matmuls) but **wins on memory-bound embedded** (ESP32-S3: no cuBLAS, PSRAM reads dominate, 1 byte/dim vs 4 bytes/dim = 4x less memory traffic).

---

## Use Case Catalog

### UC-01: semantic-memory — PerDim Candidate Pre-Filter

**Target:** `semantic-memory/src/search.rs` — `turbo_quant_vector_outcome()`  
**Description:** Add a `PerDim` codec path parallel to the existing `turbo_quant_vector_outcome()`. The search pipeline already supports `DerivedVectorBackendPolicy::TurboQuantCandidateOnly` which uses `CompressedScorerAdapter::turbo_quant()` for compressed-domain candidate scoring, then exact f32 rerank. A `PerDimCandidateOnly` policy would use `CompressedScorerAdapter::per_dim()` for even cheaper candidate filtering (1 byte/dim vs turbo-quant's polar code + residual sketch).

**Feasibility:** HIGH — the integration template already exists in `search.rs:1101-1389`. The `CompressedScorerAdapter::per_dim()` constructor is already implemented in `scr-runtime-compression`. Requires: (1) adding `PerDimCandidateOnly` to `DerivedVectorBackendPolicy`, (2) implementing `per_dim_vector_outcome()` mirroring `turbo_quant_vector_outcome()`, (3) adding `per_dim_bits` to `SearchConfig`.

**Effort:** 2–3 days (copy-adapt from TurboQuant path, add config, tests, receipts).

**ROI:** MEDIUM — PerDim is cheaper to encode/decode than TurboQuant but less accurate. Best for large corpora where the compressed pre-filter saves memory bandwidth. Not a GPU win, but helps on CPU-only desktop RAG (Gloss).

**Competitive positioning vs TurboQuant:** PerDim is simpler (uniform min/max, no rotation, no codebook), cheaper to fit (one pass for min/max vs seeded rotation + projection), but TurboQuant's polar coordinate approach captures more structure. For retrieval candidate filtering where exact rerank follows, PerDim's lower quality is acceptable.

---

### UC-02: ESP32-S3 — AttentionCache Replacing Int4KvCache

**Target:** `projects/esp32-reusable/crates/ri-esp-llm/src/kv_cache.rs`  
**Description:** The current `Int4KvCache<D, P, N>` stores int4-quantized key/value pairs and computes attention by **dequantizing every key** to f32 before computing the dot product (`attention_scores()` at line 69–83). This is O(N × D) dequantization + dot product per query. `AttentionCache<S: CompressedScorer>` from `compressed-scorer` scores compressed keys **without decompression** and only decodes the top-K values.

**Current hot path (Int4KvCache):**
```rust
// For each of N cached keys:
dequantize_i4(key, scale, &mut key_f32);  // O(D) memory write
dot = sum(query[i] * key_f32[i]);         // O(D) multiply-add
```

**Proposed hot path (AttentionCache<PerDimScorer>):**
```rust
// Prepare query once: O(D) quantize
// For each of N cached keys:
score = scorer.score_prepared(&prepared, &compressed_key);  // O(D) u8×u8 lookup — no dequantize
// Only decode top-K values: O(K × D) instead of O(N × D)
```

**Feasibility:** HIGH — `compressed-scorer` already compiles for `xtensa-esp32s3-none-elf` with `no_std` feature. `AttentionCache` uses `Vec` (alloc), which works on ESP32-S3 with PSRAM. The `PerDimScorer` needs a `fit()` call on a calibration batch (could be done at model init).

**Effort:** 3–5 days. Need: (1) replace `Int4KvCache` with `AttentionCache<PerDimScorer>` in `ri-esp-llm`, (2) wire `PerDimScorer::fit()` at init, (3) benchmark attention latency + memory on ESP32-S3 hardware, (4) handle const generics (current Int4KvCache uses `const D, P, N` — AttentionCache uses dynamic Vec).

**ROI:** HIGH — This is the **highest-ROI use case** per the roadmap. ESP32-S3 is memory-bound, no cuBLAS, PSRAM reads at 100+ ns. Compressed keys: 1 byte/dim vs 4 bytes/dim = 4x less memory traffic. Only top-K values decoded. Direct speed win on the exact hardware where the algorithm wins.

**Challenges:** `AttentionCache` uses `Vec` (heap alloc). ESP32-S3 with PSRAM supports this, but the current `Int4KvCache` uses stack-allocated arrays (`[KvEntry<D, P, N>]`). May need a `heapless::Vec` variant of `AttentionCache` or accept PSRAM allocation. The `CompressedScorer` trait uses `Vec<f32>` in `decode()` — could use a callback pattern or pre-allocated buffer.

---

### UC-03: ESP32-S3 — RNN Hidden State Compression

**Target:** `projects/esp32-reusable/crates/ri-esp-llm/src/rnn.rs`  
**Description:** The RNN step function (`rnn_step`) computes `hidden × weight_hh` as a full f32 matrix-vector product every step. The hidden state `h: [f32; HIDDEN]` could be compressed using PerDimScorer, and the recurrent matrix product estimated in compressed domain. However, RNN hidden states are updated every step and need full precision for stability — compressed-domain scoring would introduce error that compounds over timesteps.

**Feasibility:** LOW — RNN hidden states require full precision for stable recurrence. Compression error would accumulate catastrophically. The `rnn_step` function is already int8-quantized for weights (not hidden state). The hidden state must remain f32.

**Effort:** N/A — not recommended for the recurrent path.

**ROI:** LOW — the hidden state is small (HIDDEN × 4 bytes), and the bottleneck is the weight matrix, not the state vector.

---

### UC-04: quant-governor — Add PerDim CodecProfile

**Target:** `quant-governor/src/decision.rs` — `CodecProfile` enum  
**Description:** The `CodecProfile` enum currently has: Raw, Q8, Q4, Turbo, Fib, Polar, Qjl, Hyperquant. Adding `PerDim` would let the governor route to per-dim quantization for workloads where it's the right choice (memory-constrained embedded, large-corpus CPU-only retrieval).

**Current routing in `policy.rs`:**
- `ContentType::Embedding` + HighPriority → Q4 or Q8
- `ContentType::Embedding` + low_latency → Turbo
- `ContentType::Embedding` + storage_efficient + BestEffort → Hyperquant

**Proposed addition:**
```rust
CodecProfile::PerDim => {
    // Best for: memory-constrained, CPU-only, large corpus
    // Asymmetric: score only, no reconstruction
    default_degradation_threshold: 0.05,
    estimated_compression_ratio: 4.0, // 8-bit per dim vs 32-bit float
}
```

Add routing logic in `select_for_embedding()`:
- If `latency_tolerance_ms > 500` and `size_bytes > 500_000` and not HighPriority → `PerDim` (cheap candidate filter for large CPU-only corpora)
- If embedded target (detect via custom `AdmissibilityClass::Embedded` or a new field) → `PerDim`

**Feasibility:** HIGH — additive change to existing enum, no breaking API changes.

**Effort:** 1 day (add enum variant, update Display, update routing logic, tests).

**ROI:** MEDIUM — enables governance-aware routing to PerDim for the right workloads. Without it, PerDim exists but isn't reachable through the policy engine.

---

### UC-05: context-governor — Compressed Context Selection

**Target:** `context-governor/src/lib.rs`  
**Description:** `context-governor` compacts conversation context for LLM agents. It summarizes old messages, keeping recent ones active. `CompressedWorkingSet<S: CompressedScorer>` from `compressed-scorer` implements query-aware working-set selection: it scores compressed "pages" (KV cache segments) against a query, applies guard policies (always include recent/sink tokens), and selects the top-K pages with progressive refinement.

**Integration concept:** When compacting context for a multi-turn agent, instead of just summarizing old messages, compress each message's embedding into PerDim format, then use `CompressedWorkingSet::select()` to pick which historical messages are most relevant to the current query. This gives **query-aware context selection** rather than fixed-window summarization.

**Feasibility:** MEDIUM — `context-governor` is host-agnostic and doesn't currently depend on `compressed-scorer`. Would need a feature-gated dependency. The `CompressedWorkingSet` requires pre-compressed pages and a scorer instance. The integration would be as an optional selection strategy.

**Effort:** 3–4 days (add feature-gated dep, implement `CompressedContextSelection` strategy, wire into the compaction pipeline, tests).

**ROI:** MEDIUM — query-aware context selection is better than fixed-window summarization for long conversations. But the main bottleneck for context-governor is token budget, not compute — the compressed scoring adds complexity for a modest quality gain.

---

### UC-06: Gloss — RAG Search with PerDim Pre-Filter

**Target:** `Coding/Gloss/src-tauri/Cargo.toml` + `src-tauri/src/state.rs`  
**Description:** Gloss already uses `semantic-memory` with the `turbo-quant-codec` feature. It could benefit from a PerDim pre-filter path for large document collections. When the corpus exceeds a threshold (e.g., 10k documents), use PerDim compressed scoring as a coarse first pass, then rerank with exact f32 dot product (the existing `TurboQuantCandidateOnly` pattern).

**Current Gloss config** (`state.rs:378-411`):
- `semantic_memory_turbo_quant_require_fresh_artifacts`
- `semantic_memory_embedding_model` = "bge-m3" (768-dim)
- `semantic_memory_embedding_provider` = "ollama" or "fastembed"

**Integration:** Add `semantic_memory_per_dim_bits` setting (default 0 = disabled, 4 or 8 = enabled). When enabled, semantic-memory's search path uses `PerDimCandidateOnly` policy (UC-01) for the vector search stage.

**Feasibility:** HIGH — depends on UC-01 being implemented. Gloss already has the turbo-quant-codec feature wired; PerDim would be an additional feature flag.

**Effort:** 1 day after UC-01 (add UI setting, wire feature flag).

**ROI:** MEDIUM — Gloss is a desktop app, so GPU acceleration isn't the bottleneck. PerDim's memory savings help with large document collections (reduces memory pressure on the vector index). The exact rerank step already exists.

---

### UC-07: Recall-Coding — Embedder + Search Acceleration

**Target:** `Coding/Recall-Coding/recall-embedder/` + `recall-daemon/`  
**Description:** Recall-Coding is a coding assistant with semantic search. It uses `semantic-memory` for storage and search. The `recall-embedder` crate implements a three-tier embedding chain (FastEmbed → Ollama → Deterministic). PerDim compressed scoring could accelerate the search step for large codebases.

**Feasibility:** MEDIUM — Recall-Coding already depends on `semantic-memory`. If UC-01 is implemented, Recall-Coding gets the benefit automatically through the semantic-memory search path. No direct code changes needed in Recall-Coding itself.

**Effort:** 0 days after UC-01 (automatic via semantic-memory).

**ROI:** LOW (for direct work) / MEDIUM (for end-user experience via UC-01).

---

### UC-08: poly-kv — Shared Compressed KV Pool with PerDim

**Target:** `poly-kv/` — shared compressed KV-cache pool  
**Description:** poly-kv already implements a two-tier codec pool (fib-quant cold + turbo-quant hot). Adding PerDim as a third tier (or replacing one) could reduce memory for the hot tier. The pool stores compressed KV pairs that multiple agents share.

**Current two-tier:**
- Hot tier: turbo-quant (polar + residual sketch)
- Cold tier: fib-quant (codebook + Gram table)

**Proposed:** Add PerDim as a "warm" tier between hot and cold:
- Hot: turbo-quant (highest quality, most memory)
- Warm: PerDim (8-bit, 4x compression, very cheap scoring)
- Cold: fib-quant (codebook, best long-term storage)

Or use PerDim for the hot tier on CPU-only/embedded targets where turbo-quant's rotation overhead is unnecessary.

**Feasibility:** MEDIUM — poly-kv's `KvPoolCodec` trait is pluggable. PerDim needs a `KvCacheCodec` implementation (like fib-quant's `compat` feature provides). The `CompressedScorerAdapter::per_dim()` already exists in `scr-runtime-compression`.

**Effort:** 2–3 days (implement `KvCacheCodec` for PerDim, add to pool dispatch, tests).

**ROI:** MEDIUM — PerDim is simpler than turbo-quant (no rotation, no projection) and cheaper to encode. For multi-agent pools where many agents share the same KV cache, the 4x memory reduction (8-bit vs 32-bit) is significant.

---

### UC-09: hnsw-bench — Compressed Pre-Filter Benchmark

**Target:** `hnsw-bench/src/main.rs`  
**Description:** The current benchmark compares `hnsw_rs` vs `usearch` for vector search. Add a third path: compressed-domain pre-filter using PerDim. Score all N vectors in compressed domain, take top-K × oversample candidates, then exact rerank with f32 dot product. Compare recall, latency, and memory against both ANN backends.

**Benchmark matrix:**

| Method | Insert | Search | Recall | Memory |
|--------|--------|--------|--------|--------|
| hnsw_rs | O(N log N) | O(log N) | high | index + raw |
| usearch | O(N log N) | O(log N) | high | index + raw |
| PerDim pre-filter | O(N) | O(N) compressed + O(K) exact | depends on oversample | compressed codes only |

**Feasibility:** HIGH — `compressed-scorer` is a workspace member. The benchmark already generates synthetic corpora. Adding a PerDim path is straightforward.

**Effort:** 1–2 days (add PerDim benchmark path, generate receipt, compare).

**ROI:** HIGH for positioning — this benchmark would definitively show where PerDim wins (large N, CPU-only, memory-constrained) vs where ANN wins (moderate N, sub-linear search). It's the evidence needed to support or refute the "PerDim for retrieval" claim.

---

### UC-10: quant-eval — Add PerDim to Compression Benchmark Suite

**Target:** `quant-eval/src/benchmarks/compression.rs` + `compressed_attention.rs`  
**Description:** `quant-eval` already has `CompressedAttentionBenchConfig` and `CompressedAttentionBenchReceipt` for benchmarking compressed attention. Add PerDim as a codec option alongside turbo-quant and fib-quant.

**Existing benchmark receipt fields:** `compression_ratio`, `topk_overlap`, `attention_output_cosine`, `attention_output_mse`, `logit_mae`, `latency_p50_us`, `latency_p95_us`, `passed`.

**Feasibility:** HIGH — the benchmark infrastructure is generic. Add `per_dim` as a codec string, wire `PerDimScorer` into the benchmark harness.

**Effort:** 1 day (add codec variant, wire scorer, run benchmarks).

**ROI:** HIGH — produces receipt-bearing evidence for PerDim's quality/latency tradeoffs. Needed for any external claim about PerDim performance.

---

### UC-11: ri_bench — Retrieval Benchmark with PerDim

**Target:** `Coding/benchmark/ri_bench/Cargo.toml`  
**Description:** `ri_bench` is a retrieval benchmark harness that already has optional deps on `turbo-quant` and `fib-quant`. Add `compressed-scorer` as an optional dependency and a `per-dim` feature flag to benchmark PerDim retrieval quality.

**Current features:** `turbo`, `fib`, `poly`. Add `per-dim = ["compressed-scorer"]`.

**Feasibility:** HIGH — straightforward dependency addition.

**Effort:** 1 day (add dep, add PerDim retrieval path, benchmark fixtures).

**ROI:** MEDIUM — extends the existing benchmark harness. Needed for cross-codec comparison.

---

### UC-12: semantic-memory — Multiscale Pipeline Compressed Stage

**Target:** `semantic-memory/src/pipeline.rs`  
**Description:** The multiscale retrieval pipeline (`#[cfg(feature = "multiscale")]`) runs search in stages with budgets and confidence thresholds. A compressed-domain scoring stage could serve as the **coarse first stage**: score all embeddings in PerDim compressed domain (very cheap), take top-K × oversample, then pass to the next stage (BM25, exact vector search, etc.).

**Current stages:** configurable closures with `StageBudget`. A compressed stage would be:
```
Stage 1 (coarse): PerDim compressed scoring — budget: max_items=N, max_time=10ms
Stage 2 (refine): exact f32 vector search on top-K — budget: max_items=K×oversample
Stage 3 (fusion): RRF with BM25 — budget: max_items=K
```

**Feasibility:** MEDIUM — the pipeline is pluggable via closures. The compressed stage needs pre-built compressed artifacts (like the turbo-quant-codec path). Requires the `compression-governor` feature to determine which embeddings get compressed.

**Effort:** 2–3 days (implement compressed stage closure, wire into pipeline config, tests).

**ROI:** MEDIUM — the multiscale pipeline is opt-in and not yet widely used. But if adopted, a compressed first stage could dramatically reduce latency for large corpora.

---

### UC-13: semantic-memory — Compression Governor Per-Vector PerDim

**Target:** `semantic-memory/src/compression_governor.rs`  
**Description:** The compression governor (`#[cfg(feature = "compression-governor")]`) scores each embedding's importance and assigns a `QuantizationLevel` (F32, SQ8, SQ4, SQ4Marked). Currently it uses access frequency, entropy, and structuring score. The governor could output `PerDim` as a quantization level, and the embeddings would be compressed using `PerDimScorer::compress()`.

**Current levels:** F32, SQ8, SQ4, SQ4Marked.  
**Proposed:** Add `PerDim8` and `PerDim4` levels.

**Feasibility:** MEDIUM — the compression governor doesn't currently produce compressed artifacts, just recommendations. Would need a separate pass that compresses embeddings according to their assigned level.

**Effort:** 2 days (add level variants, implement compression pass, tests).

**ROI:** LOW — the compression governor is opt-in and not yet production-deployed. Adding PerDim levels is easy but the whole pipeline needs to be activated first.

---

### UC-14: CompressedWorkingSet — Multi-Agent KV Page Selection

**Target:** `compressed-scorer/src/working_set.rs` — `CompressedWorkingSet<S>`  
**Description:** `CompressedWorkingSet` selects which compressed KV pages to load for a given query. It's designed for multi-agent shared KV cache scenarios where multiple agents share a pool of compressed KV pages. The selection uses `PageRole` (SharedCold, AgentHot, RecentGuard, SinkGuard, etc.) with guard policies and progressive refinement.

**Potential consumers:**
- `poly-kv` shared KV pool (UC-08)
- `context-governor` context compaction (UC-05)
- Any future multi-agent system that shares compressed KV state

**Feasibility:** HIGH — the module is already implemented and tested. It just needs consumers.

**Effort:** 0 (already built) — effort is in the consumers.

**ROI:** HIGH (as infrastructure) — this is the reusable selection layer that ties compressed scoring to multi-agent KV management. Every consumer (UC-05, UC-08) benefits.

---

### UC-15: AdaptiveBudget — ESP32 Attention Head Budgeting

**Target:** `compressed-scorer/src/adaptive_budget.rs`  
**Description:** `AdaptiveBudget` allocates per-layer/head token budgets based on fragility (cosine p05 at reference top-k). For ESP32-S3 with extremely limited memory, this is critical: some attention heads are robust to aggressive compression (high cosine p05), while others need full precision. The budget allocator minimizes total selected keys while keeping each layer's expected cosine above a target.

**Current defaults:** `ref_k=64, target_cosine=0.995, min_k=32, max_k=256, recent_guard=16`.

**ESP32 adaptation:** For ESP32-S3 with 512KB SRAM, use `min_k=8, max_k=64, recent_guard=4`. The allocator would determine which heads can use PerDim 4-bit (very aggressive) vs which need 8-bit or even full precision.

**Feasibility:** HIGH — already implemented, no_std compatible, compiles for ESP32-S3.

**Effort:** 1 day (tune for ESP32 parameters, integrate with AttentionCache usage).

**ROI:** HIGH for ESP32 — this is the brain that decides how aggressively to compress each attention head. Without it, all heads get the same compression level, wasting memory on robust heads and losing quality on fragile ones.

---

### UC-16: fib-quant KV — Compressed Attention Integration

**Target:** `fib-quant/src/kv/compressed_attention.rs`  
**Description:** fib-quant already has a `kv` module with `compressed_attention.rs` that implements compressed-domain attention using FibScorer's Gram-table lookups. The `AttentionCache<S: CompressedScorer>` in `compressed-scorer` is the codec-agnostic version of this. The fib-quant `kv` module could use `AttentionCache<FibScorerAdapter>` instead of its own implementation, reducing code duplication.

**Feasibility:** MEDIUM — would require refactoring fib-quant's `kv` module to use the `CompressedScorer` trait. The `FibScorerAdapter` already wraps fib-quant's `FibScorer`.

**Effort:** 2 days (refactor to use AttentionCache, ensure behavior parity, tests).

**ROI:** LOW — the fib-quant `kv` module already works. The benefit is code consolidation, not new functionality.

---

### UC-17: External — crates.io Publication as no_std Scoring Library

**Target:** External crates.io ecosystem  
**Description:** `compressed-scorer` is already structured for publication: MIT/Apache-2.0 dual license, `no_std` feature, minimal dependencies (heapless, libm). Publishing it would make compressed-domain scoring available to the broader Rust embedded and ML communities.

**Key differentiators vs existing crates:**
- `quantization` crate: scalar quantization only, no compressed-domain scoring trait
- `embed-text` / `simd-distance`: operate on f32 vectors, not compressed
- No existing crates.io crate provides codec-agnostic compressed-domain inner product scoring with `no_std` support

**Feasibility:** HIGH — the crate is well-structured, tested, documented. Needs: README polish, API docs, version stamping, `cargo publish` dry run.

**Effort:** 1 day (docs, README, publish).

**ROI:** MEDIUM — establishes RecursiveIntell as a contributor to the Rust ML/embedded ecosystem. The crate fills a genuine gap (no_std compressed-domain scoring). Low direct revenue but high visibility.

---

### UC-18: External — ESP32 Compressed Attention Demo

**Target:** ESP32-S3 hardware demo  
**Description:** The roadmap identifies this as the **highest-ROI move**: "ESP32 compressed attention demo" where memory savings translate directly to speed wins. Build a demonstration that:
1. Loads a small attention head (e.g., 128-dim, 256 tokens) onto ESP32-S3
2. Compresses keys with PerDimScorer (1 byte/dim = 128 bytes/key vs 512 bytes/key fp32)
3. Runs `AttentionCache::attention_topk()` to compute attention with only top-K value decompression
4. Measures latency and memory vs the existing `Int4KvCache` approach
5. Produces a receipt with quality (cosine vs exact) and performance (latency, memory)

**Feasibility:** HIGH — all Rust code compiles for ESP32-S3. The `AttentionCache` and `PerDimScorer` are no_std compatible. Need to wire into the `ri-esp-llm` crate and flash to hardware.

**Effort:** 3–5 days (integrate into ri-esp-llm, flash to hardware, measure, produce receipt).

**ROI:** HIGH — this is the one place where PerDim's memory savings become speed wins. It validates the entire compressed-scorer project with real hardware evidence. The demo is differentiating: "compressed-domain attention on ESP32-S3" is novel.

---

## Competitive Positioning: PerDim vs Turbo-quant vs Fib-quant

| Dimension | PerDimScorer | TurboScorerAdapter | FibScorerAdapter |
|-----------|-------------|-------------------|------------------|
| **Algorithm** | Per-dim uniform min/max quant | Seeded rotation + polar quant + QJL sketch | Fibonacci radial-angular codebook |
| **Compression** | 4x (8-bit), 8x (4-bit) | ~3x (polar), ~8x (QJL) | ~2.5x (codebook) |
| **Quality** | cosine >0.99 (8-bit) | Good (rotation preserves structure) | Best (Gram table exact for codebook) |
| **Scoring cost** | O(D) u8 multiply | O(projections) | O(1) table lookup |
| **Fit cost** | One pass (min/max) | Seeded rotation (no fit) | Lloyd's algorithm (expensive) |
| **no_std** | ✅ | ❌ (nalgebra) | ❌ (nalgebra, rand) |
| **ESP32-S3** | ✅ Compiles | ❌ (nalgebra dep) | ❌ (nalgebra dep) |
| **Decompression** | O(D) per vector | O(D) per vector | O(D) per vector |
| **Best for** | Embedded, CPU-only, large corpus | GPU (when not competing with cuBLAS), asymmetric scoring | Small fixed codebook, highest quality |
| **Worst for** | GPU (cuBLAS wins) | GPU (cuBLAS wins), embedded (nalgebra) | Large codebooks, embedded (nalgebra) |

### Key positioning insight

**PerDimScorer is the only scorer that compiles for ESP32-S3.** TurboScorerAdapter and FibScorerAdapter both depend on `nalgebra`, which is not `no_std` compatible without significant work. This makes PerDim the **only viable compressed scorer for embedded targets** in the current architecture.

For desktop/server retrieval, TurboQuant's polar coordinates and QJL sketches provide better quality at similar compression ratios, and the `turbo-quant-codec` feature in semantic-memory already wires this path.

For embedded attention, PerDim is the clear winner — it's already there, already no_std, and the memory savings translate to speed on memory-bound hardware.

---

## Priority Matrix

### Tier 1 — Ship First (HIGH ROI, HIGH Feasibility)

| # | Use Case | Effort | Why First |
|---|---------|--------|-----------|
| UC-18 | ESP32 compressed attention demo | 3–5 days | Validates entire project with hardware evidence |
| UC-02 | AttentionCache replacing Int4KvCache | 3–5 days | Core embedded integration, enables UC-18 |
| UC-15 | AdaptiveBudget for ESP32 | 1 day | Brain for UC-02, already built |
| UC-09 | hnsw-bench PerDim benchmark | 1–2 days | Evidence for retrieval positioning |
| UC-10 | quant-eval PerDim benchmark | 1 day | Receipt-bearing quality evidence |

### Tier 2 — Ship Next (MEDIUM ROI, HIGH Feasibility)

| # | Use Case | Effort | Why Next |
|---|---------|--------|----------|
| UC-01 | semantic-memory PerDim search path | 2–3 days | Unlocks UC-06, UC-07 automatically |
| UC-04 | quant-governor PerDim profile | 1 day | Enables governance routing |
| UC-06 | Gloss PerDim pre-filter | 1 day | Depends on UC-01 |
| UC-17 | crates.io publication | 1 day | External visibility |

### Tier 3 — When Needed (MEDIUM ROI, MEDIUM Feasibility)

| # | Use Case | Effort | Why Later |
|---|---------|--------|-----------|
| UC-08 | poly-kv PerDim tier | 2–3 days | Multi-agent pool, not yet deployed |
| UC-14 | CompressedWorkingSet consumers | varies | Infrastructure ready, needs users |
| UC-05 | context-governor compressed selection | 3–4 days | Complex integration |
| UC-11 | ri_bench PerDim benchmark | 1 day | Extends existing harness |
| UC-12 | semantic-memory multiscale compressed stage | 2–3 days | Opt-in pipeline, not yet used |

### Tier 4 — Skip or Deprioritize (LOW ROI)

| # | Use Case | Why Skip |
|---|---------|----------|
| UC-03 | RNN hidden state compression | Error compounds, not recommended |
| UC-13 | compression-governor PerDim levels | Governor not yet deployed |
| UC-16 | fib-quant KV AttentionCache refactor | Code consolidation only |
| UC-07 | Recall-Coding (direct work) | Automatic via UC-01, no direct work needed |

---

## Dependency Graph

```
UC-04 (governor profile) ──┐
                            ├─→ UC-01 (sm PerDim search) ──→ UC-06 (Gloss) ──→ UC-07 (Recall auto)
UC-10 (quant-eval bench) ──┘                                  │
                                                              │
UC-15 (adaptive budget) ──→ UC-02 (ESP32 AttentionCache) ──→ UC-18 (ESP32 demo)
                              │
UC-09 (hnsw-bench) ───────────┤
                              │
UC-14 (WorkingSet infra) ──→ UC-05 (context-gov) ──→ UC-08 (poly-kv)
                           ──→ UC-12 (multiscale stage)

UC-17 (crates.io publish) — independent, ship anytime after UC-10
```

---

## Summary

**Total use cases cataloged:** 18  
**Tier 1 (ship first):** 5 use cases, ~9–13 days total effort  
**Tier 2 (ship next):** 4 use cases, ~5–6 days total effort  
**Tier 3 (when needed):** 5 use cases, ~9–12 days total effort  
**Tier 4 (skip):** 4 use cases  

**Highest-ROI single action:** UC-18 (ESP32 compressed attention demo) — validates the entire compressed-scorer project with hardware evidence, leverages all the no_std work already done, and demonstrates the one domain where PerDim wins on speed (not just memory).

**Highest-ROI platform action:** UC-01 (semantic-memory PerDim search path) — unlocks Gloss (UC-06) and Recall-Coding (UC-07) automatically, adds PerDim as a first-class retrieval option in the semantic-memory search pipeline.