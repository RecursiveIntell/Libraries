# Compressed-Domain Scoring: Complete Use Case Catalog

Date: 2026-06-30

## What We Have

**Rust crates:**
- `compressed-scorer` — `CompressedScorer` trait, `PerDimScorer`, `AttentionCache`, adapters for fib-quant and turbo-quant. no_std/ESP32 compatible.
- `scr-runtime-compression` — `CompressedScorerAdapter`, `CodecId::PerDim`, runtime dispatch
- `turbo-quant` — Polar/QJL compressed vector codec (crates.io published)
- `fib-quant` — Gram-table compressed scoring (crates.io published)
- `quant-governor` — `CodecProfile` enum, policy-driven codec routing
- `semantic-memory` — SQLite + HNSW + brute-force vector search, `Quantizer` (SQ8), `vector_codec` profiles
- `gpu-backend` — `page_scorer` for FibGram page-level scoring (CUDA + CPU fallback)
- `quant-eval` — compressed attention benchmark harness
- `poly-kv` — compressed KV-cache pool with q8 key path
- `hyperquant` — experimental lattice quantization

**Python:**
- `compressed_attention_forward_ppl.py` — forward-pass quality gate with per-dim/quantized/fib-gram scorers
- Triton fused kernels for per-dim and per-key scoring (correct, 4-6x slower than cuBLAS)

**ESP32:**
- `esp32-reusable` crates: `ri-esp-proof`, `ri-esp-tiered`, `ri-esp-llm`, `spec-core`
- `esp32-sensor-hub` — DHT/OLED/WiFi/HTTP sensor endpoint
- `tiered-edge-ai` — ESP32-S3 sentinel + heavier inference tier
- All compressed-scorer code compiles for xtensa-esp32s3-none-elf

## Use Case Catalog (Ranked by ROI)

### Tier 1: Highest ROI — Ship Now

---

**1. ESP32 Compressed Attention Demo**
- Feasibility: HIGH — code already compiles for ESP32-S3, AttentionCache is no_std
- Effort: 1-2 days
- ROI: Proves compressed scoring wins on memory-bound hardware (no cuBLAS, PSRAM-limited)
- What to build: Small attention head (dim=32 or 64), pre-quantized keys in flash, PerDimScorer scores compressed, AttentionCache selects top-k, decode only selected values, output to OLED
- Evidence needed: Latency comparison vs brute-force fp32 on ESP32 (should win: 4x less PSRAM traffic)
- Files: `esp32-reusable/crates/ri-esp-llm/` or new `ri-esp-attention` crate

---

**2. Publish compressed-scorer to crates.io**
- Feasibility: HIGH — 21 tests pass, no_std compatible, clean API, workspace compiles
- Effort: 1-2 hours
- ROI: Public verifiable artifact, fills gap (no other crate provides codec-agnostic compressed-domain scoring)
- Positioning: "Codec-agnostic compressed-domain scoring for retrieval and attention. no_std compatible for embedded."
- Prerequisites: `cargo package`, `cargo publish --dry-run`, verify no warnings

---

**3. Semantic-Memory Brute-Force Compressed Candidate Generation**
- Feasibility: HIGH — semantic-memory already has brute-force search path, already has SQ8 Quantizer
- Effort: 3-4 hours
- ROI: Speed up brute-force vector search by scoring compressed vectors first, then decoding only top-k
- What to build: Adapter that uses `CompressedScorerAdapter<PerDimScorer>` or `TurboScorerAdapter` in the brute-force scan path of `semantic-memory/src/search.rs`
- Current state: brute-force scans all f32 embeddings. Compressed scoring would scan uint8 codes (4x less memory) and decode only top-k results
- Evidence needed: Retrieval benchmark on real facts (recall@10, latency) vs current brute-force

---

### Tier 2: Medium ROI — Worth Building

---

**4. Quant-Governor Routing to PerDim**
- Feasibility: HIGH — `CodecProfile` enum exists, just needs `PerDim` variant
- Effort: 2-3 hours
- ROI: Completes the policy-driven codec selection story; quant-governor can route to per-dim for memory-constrained scenarios
- What to build: Add `CodecProfile::PerDim` to quant-governor, add expected_loss value, wire `codec_profile_to_codec_id` mapping (already done in scr-runtime-compression)
- Current state: `CodecProfile` has Raw/Q8/Q4/Turbo/Fib/Polar/Qjl/Hyperquant but no PerDim

---

**5. GPU-Backend Page Scorer with PerDim**
- Feasibility: MEDIUM — gpu-backend already has `page_scorer` for FibGram, adding per-dim is straightforward
- Effort: 3-4 hours
- ROI: Alternative page-level scoring method for compressed KV-cache pages
- What to build: Add `PerDimPageScoreInput` and `score_per_dim_pages` function alongside existing `score_fib_gram_pages`
- Current state: `gpu-backend/src/page_scorer.rs` only supports FibGram scoring

---

**6. Quant-Eval Compressed Attention Benchmark with PerDim**
- Feasibility: HIGH — quant-eval already has `compressed_attention` module
- Effort: 2-3 hours
- ROI: Honest benchmark comparing per-dim vs turbo vs fib in the quant-eval harness
- What to build: Add per-dim as a scorer option in `quant-eval/src/compressed_attention.rs`, run synthetic benchmark
- Current state: quant-eval has compressed attention bench but only tests turbo/fib

---

**7. Semantic-Memory Vector Codec Profile for PerDim**
- Feasibility: MEDIUM — `vector_codec.rs` has `VectorCodecProfileV1` with codec field
- Effort: 4-6 hours
- ROI: PerDim as a first-class vector codec in semantic-memory, stored alongside SQ8
- What to build: Add `PerDimCodec` implementing `VectorCodec` trait, create profile with per-dim parameters, store compressed codes in SQLite
- Current state: semantic-memory has `TurboQuantCodec`, `Sq8Codec`, `RawF32Codec`. PerDim would be additive.

---

### Tier 3: Lower ROI — Interesting But Not Urgent

---

**8. Poly-KV Compressed KV-Cache Pool with PerDim Keys**
- Feasibility: MEDIUM — poly-kv has q8 key path, per-dim would be an alternative
- Effort: 1 day
- ROI: Another codec option for the KV-cache pool; per-dim may be better for certain key distributions
- What to build: Add per-dim as a key codec option in poly-kv's pool, alongside q8
- Current state: poly-kv has q8 key codec and pluggable value codec boundary

---

**9. Context-Governor Compressed Context Retrieval**
- Feasibility: LOW — context-governor does prompt compaction, not vector search
- Effort: 1-2 days
- ROI: Could use compressed scoring to select which conversation messages to keep vs compress
- What to build: Use PerDimScorer to score message embeddings, select top-k most relevant messages before compaction
- Current state: context-governor does rule-based compaction, not embedding-based selection

---

**10. Forge-Engine Compressed Retrieval**
- Feasibility: LOW — forge-engine doesn't currently use vector search
- Effort: 2+ days
- ROI: Unclear; forge-engine is a memory/forge tool, not a retrieval engine
- What to build: Add compressed vector search to forge-engine's memory operations
- Current state: No vector search in forge-engine

---

**11. Gloss/Tauri App Compressed Search**
- Feasibility: LOW — Gloss is a Tauri desktop app, adding compressed search would be a feature
- Effort: 2-3 days
- ROI: Desktop app with fast compressed search would be a nice demo but not a differentiator
- What to build: Add compressed-scorer as a dependency in Gloss, use for document search
- Current state: Gloss uses semantic-memory for search; compressed scoring would be transparent

---

**12. ESP32 Sensor Anomaly Detection with Compressed HDC**
- Feasibility: MEDIUM — research doc mentions HDC/sentinel for ESP32
- Effort: 2-3 days
- ROI: Compressed scoring for sensor pattern matching on ESP32 (Hamming distance on binary codes)
- What to build: Encode sensor windows as binary hypervectors, use compressed scoring for anomaly detection
- Current state: Not built; `nimblecube` external project shows HDC on ESP32-S3

---

**13. AiDENs Agent Attention with Compressed Memory**
- Feasibility: LOW — AiDENs is an agent framework, not a retrieval system
- Effort: 3+ days
- ROI: Agents could use compressed scoring to select relevant memories
- What to build: Add CompressedScorerAdapter to AiDENs memory module
- Current state: AiDENs doesn't have vector-based memory selection

---

## Competitive Positioning

### PerDim vs Turbo (turbo-quant)
- Turbo: Data-oblivious, no calibration needed, O(1) scoring via polar projections
- PerDim: Needs calibration (fit min/max), O(dim) scoring, but simpler and more interpretable
- When to use PerDim: When you want simple, auditable quantization with no random projections
- When to use Turbo: When you need O(1) scoring or data-oblivious compression

### PerDim vs Fib (fib-quant)
- Fib: Gram-table lookup, O(1) per scored vector, requires codebook
- PerDim: Direct dot product, O(dim) per scored vector, no codebook
- When to use PerDim: When codebook construction is too expensive or data changes frequently
- When to use Fib: When you can precompute the Gram table and want O(1) scoring

### PerDim vs SQ8 (semantic-memory's Quantizer)
- SQ8: Per-vector affine quantization (256 levels), 4x compression, per-vector scale/zero_point
- PerDim: Per-dimension uniform quantization (15-255 levels), 12-57x compression (with unit normalization), shared per-dim stats
- Key difference: SQ8 quantizes each vector independently; PerDim quantizes all vectors with shared per-dim statistics
- When to use PerDim: When you have a batch of vectors and want better compression via shared statistics
- When to use SQ8: When each vector must be self-contained (no shared stats)

## Recommended Execution Order

1. **Publish compressed-scorer to crates.io** (1-2 hr) — public artifact
2. **Add PerDim to quant-governor CodecProfile** (2-3 hr) — completes routing
3. **ESP32 compressed attention demo** (1-2 days) — highest ROI proof
4. **Semantic-memory compressed candidate generation** (3-4 hr) — real retrieval speedup
5. **Quant-eval benchmark with per-dim** (2-3 hr) — honest comparison
6. **GPU-backend page scorer with per-dim** (3-4 hr) — alternative scoring path

Total: 3-4 days for all Tier 1 + Tier 2 items.