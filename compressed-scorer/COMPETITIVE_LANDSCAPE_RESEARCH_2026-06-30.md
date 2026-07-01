# Competitive Landscape Research: Compressed-Domain Vector Scoring

**Date:** 2026-06-30  
**Scope:** Competitive positioning, technical context, and next-best-path analysis for `compressed-scorer` and the broader RecursiveIntell compressed scoring stack.

---

## 1. Competing Quantization Approaches (QServe, AWQ, GPTQ, QuaRot, SpinQuant, LLMLingua)

### What exists

| Approach | Stars | Core Idea | Relevance |
|----------|-------|-----------|-----------|
| **QServe** (MIT-Han Lab, MLSys'25) | 844 | W4A8KV4 quantization + system co-design. Weights 4-bit, activations 8-bit, KV cache 4-bit. GPU-optimized kernel fusion. | Production W4A8KV4 with co-designed CUDA kernels |
| **AWQ** (MIT-Han Lab, MLSys'24 Best Paper) | 3,577 | Activation-aware weight quantization. Preserves salient channels based on activation magnitudes. | Weight-only, GPU-focused, doesn't score in compressed domain |
| **GPTQ** (AutoGPTQ) | 5,071 | Post-training weight quantization via second-order error compensation. Layer-by-layer Hessian approximation. | Weight-only quantization, dequantize-then-matmul |
| **QuaRot** (NeurIPS'24) | 518 | End-to-end 4-bit inference via random Hadamard rotations to flatten outliers before quantization. | Rotation-based like turbo-quant, but for full model inference |
| **SpinQuant** (Meta) | 407 | Learned rotations for quantization — generalizes QuaRot's random rotations with optimized rotation matrices. | Optimization on the rotation matrix, same family as QuaRot |
| **FlatQuant** (ICML'25) | 221 | Flatness-aware quantization with learnable per-channel transforms. | Latest in rotation/transform-based quantization |
| **LLMLingua** (Microsoft, EMNLP'23) | 6,376 | Prompt/token-level compression via entropy-based scoring. Up to 20x prompt compression. | Token-level, not vector-level. Different layer of the stack. |

### How our work compares

**These are all GPU-server weight+activation quantization schemes.** Every one of them:
- Targets NVIDIA GPUs with Tensor Cores
- Uses dequantize-then-matmul (score after decompression)
- Operates at the model weight/activation level, not the retrieval/embedding level
- Requires significant memory for runtime (not embedded)

**Our per-dim compressed scoring is fundamentally different:**
- Scores uint8 codes **directly** (no dequantization for scoring)
- Only decodes top-K values (K << N)
- Operates at the embedding/retrieval level, not model weights
- no_std + ESP32 compatible — none of the above can run on a microcontroller

**Key learning from QuaRot/SpinQuant:** Random rotations (what turbo-quant does) are validated by top-tier research. QuaRot proves Hadamard rotations flatten outliers for better quantization — this is exactly turbo-quant's approach. The fact that Meta's SpinQuant optimized the rotation matrix suggests future improvement potential for turbo-quant.

**Key learning from QServe W4A8KV4:** KV cache 4-bit quantization is production-validated. Our approach is more aggressive (score in compressed domain) but narrower (attention cache, not full model inference).

### Next move

- **No competitive threat here.** These are orthogonal — weight quantization for GPU inference vs. our embedding/attention scoring for embedded/CPU. The rotation insight from QuaRot/SpinQuant validates turbo-quant's approach.
- Could borrow QuaRot's Hadamard rotation idea as a cheaper alternative to turbo-quant's seeded rotation for CPU-only paths.

---

## 2. ESP32/Edge AI Attention Implementations

### What exists

| Project | Stars | What it does |
|---------|-------|--------------|
| **Atome-LM** (TilelliLab) | 49 | Ternary (-1/0/1) microcontroller LM, runs on ESP32-WROOM-32 at ~1 tok/s. Zero-heap, bit-exact Python→C. |
| **xiaoclaw** (beancookie) | 35 | ESP32-S3 voice assistant with local LLM inference, tool calling, long-term memory. Uses cloud TTS. |
| **synapse** (eren23) | 1 | Rust+Zig LLM engine targeting ESP32 with INT8/Q4 quantization. Modular multi-model support. |
| **ESP32 Voice Assistant** | 42 | End-to-end conversational AI on ESP32 with cloud escalation. |
| **TinyML ecosystem** (Edge Impulse, TFLite Micro, microflow) | 5,221 (microflow) | Small ML models (classification, KWS, object detection) on ESP32-S3 via INT8 quantization. |
| **ruvllm-esp32** (crates.io) | 110 downloads | "Tiny LLM inference for ESP32 with INT8/INT4 quantization, multi-chip federation, SNN-gated energy" — appears to be our ecosystem. |
| **ruvllm-sparse-attention** (crates.io) | 25 downloads | Subquadratic O(N log N) sparse attention for edge LLM inference. Also our ecosystem. |

### What does NOT exist

- **No compressed attention cache on any microcontroller.** GitHub searches for "ESP32 KV cache", "microcontroller compressed attention", "ESP32 compressed KV" return zero relevant results.
- **No compressed-domain scoring on embedded.** No one is scoring uint8 codes directly without dequantization on ESP32.
- The TinyML ecosystem is all about small CNNs/classifiers, not transformer attention.
- Atome-LM uses ternary weights but full-precision attention computation.

### How our work compares

**We are the only project with:**
- A `CompressedAttentionCache` that scores uint8 keys directly (no dequantization)
- const-generic heapless operation (no alloc needed)
- no_std compatibility verified for xtensa-esp32s3-none-elf
- 9 passing tests for compressed attention on ESP32-S3

The `CompressedAttentionCache<D, N, K>` in `ri-esp-llm` already:
- Stores keys as `[u8; D]` (1 byte/dim) vs f32 (4 bytes/dim) = 3.56x compression
- Scores with `sum(codes[d] * scaled_query[d])` — pure integer multiply-add
- Only decodes top-K values for output aggregation
- Has `fit()` for per-dim calibration

### Next move

**This is the strongest gap we fill.** No one else has compressed-domain attention on microcontrollers. The next move is to **flash it to real hardware and produce a benchmark receipt**. This is UC-18 (ESP32 compressed attention demo) — the highest-ROI action in the use-case catalog.

---

## 3. Semantic-Memory Retrieval Acceleration (PQ, ScaNN, diskANN, FAISS IVF)

### What exists

| System | Stars | Approach | Compressed scoring? |
|--------|-------|----------|---------------------|
| **FAISS** (Facebook) | 40,425 | IVF, PQ, HNSW, exhaustive search. Industry standard. | PQ uses ADC (Asymmetric Distance Computation) — query is f32, database is PQ codes, distance computed via lookup tables |
| **ScaNN** (Google) | Part of google-research | Anisotropic quantization + tree partitioning. Learned rotation + quantization. | Yes — scores compressed codes using asymmetric distance tables |
| **DiskANN** (Microsoft) | 1,860 | Disk-based graph + PQ for compression. Vamana graph. | PQ for compressed representation, graph for search |
| **usearch** (Unum) | 4,198 | HNSW + compressed storage. C++ core with Rust bindings. | No compressed-domain scoring — stores f32 |
| **diskann-rs** (crates.io) | 1,992 downloads | Rust DiskANN implementation | No compressed scoring |
| **ruvector-pq-search** (crates.io) | 12 downloads | Product Quantization with ADC — Flat PQ, IVF+PQ, Residual PQ | Yes — closest competitor to compressed-scorer on crates.io |

### How our work compares

**Product Quantization (PQ)** is the established compressed-domain scoring approach:
- PQ divides vectors into sub-vectors, quantizes each independently with k-means
- ADC (Asymmetric Distance Computation): query stays f32, database is PQ codes, distance via precomputed lookup tables
- This is conceptually identical to compressed-scorer's approach!

**Key difference:** PQ uses codebook lookup tables (O(1) per sub-vector), while PerDim uses per-dimension integer multiply (O(D) per vector). PQ is more aggressive compression (8x-64x) but requires trained codebooks. PerDim is simpler (no training, just min/max) but lower compression (4x).

**ScaNN's anisotropic quantization** is the state of the art: it learns the quantization to preserve dot-product ordering, not just reconstruction. This is the same insight compressed-scorer uses — approximate scores for candidate generation, exact rerank for final results.

### Competitive positioning

| Dimension | PQ/ADC | PerDim (ours) | ScaNN |
|-----------|--------|---------------|-------|
| Compression | 8-64x | 4x (8-bit), 8x (4-bit) | ~8-16x |
| Training | k-means (expensive) | min/max (one pass) | Learned (expensive) |
| Scoring cost | O(M) table lookups (M=subvectors) | O(D) integer multiply | O(M) lookups |
| no_std | No (existing impls) | ✅ | No |
| ESP32 | ❌ | ✅ | ❌ |
| Quality | Good (codebook captures structure) | >0.995 cosine (8-bit) | Best (anisotropic) |
| Simplicity | Medium | High (no training) | Low (complex learning) |

### Next move

- **We are not competitive with PQ on compression ratio**, but we win on simplicity and embedded compatibility.
- The **hnsw-bench benchmark (UC-09)** is critical here — we need to show where PerDim wins vs ANN. The hypothesis: PerDim's O(N) compressed scan beats HNSW's O(log N) when N is small enough and memory traffic dominates (embedded).
- Consider adding a PQ codec to compressed-scorer (it would be another `CompressedScorer` implementation) — this would give us both the simple PerDim path and the higher-compression PQ path.

---

## 4. KV Cache Compression (2024-2025) — Production Systems

### What exists

| System | Stars | Approach | KV cache compression |
|--------|-------|----------|---------------------|
| **StreamingLLM** (MIT-Han, ICLR'24) | 7,238 | Attention sinks + sliding window | Keep first 4 + last N tokens, evict everything else |
| **H2O** (FMInference, NeurIPS'23) | 523 | Heavy-hitter oracle | Evict KV pairs with low attention contribution |
| **KIVI** (ICML'24) | 413 | 2-bit KV cache quantization | Per-channel 2-bit quantization of K, per-token 2-bit for V. Tuning-free. |
| **KVPress** (NVIDIA) | 1,120 | Framework for KV cache compression | Pluggable compression policies (SnapKV, StreamingLLM, etc.) |
| **QServe** (MLSys'25) | 844 | W4A8KV4 | KV cache at 4-bit as part of full system co-design |
| **SnapKV** (NeurIPS'24) | — | Observation window compression | Keep important KV pairs based on attention patterns |
| **tq-kv** (crates.io) | 203 downloads | TurboQuant KV Cache Compression for LLMs | Per-head adaptive bitwidth, 4-bit value compression, SRHT QJL. GGUF Q4_K_M. 114 tests. |
| **qatq** (crates.io) | 67 downloads | Quaternion tensor-aware KV compression | Runtime migration, exact tensor-aware compression |
| **quillcache** (crates.io) | 26 downloads | Vendor-neutral KV cache control plane | Evaluation platform for LLM serving |

### How our work compares

**Production KV cache compression falls into three categories:**

1. **Eviction-based** (StreamingLLM, H2O, SnapKV): Reduce cache size by throwing away tokens. Our work is orthogonal — we compress the remaining tokens, not decide which to keep.

2. **Quantization-based** (KIVI, QServe KV4): Quantize K/V to 2-4 bits. This is the direct competitor. Key difference:
   - KIVI/QServe quantize then **dequantize** for attention computation
   - We score **without dequantization** — the compressed codes are the scoring input
   - KIVI uses per-channel quantization (smart), we use per-dimension (simpler)
   - KIVI targets GPU, we target ESP32-S3

3. **Framework-based** (KVPress): Pluggable compression. Our approach could be a KVPress policy (in theory), but KVPress is Python/PyTorch and we're Rust/no_std.

**tq-kv on crates.io is the closest Rust competitor** — it does TurboQuant KV cache compression with per-head adaptive bitwidth and AVX2 SIMD. However:
- tq-kv is x86/AVX2 only — not no_std, not ESP32
- tq-kv still dequantizes for attention (uses SIMD fused dequant+matmul)
- Our CompressedAttentionCache scores without dequantizing at all

### Next move

- **The "score without decompression" angle is our unique differentiator** — nobody else does this. KIVI quantizes to 2-bit but still dequantizes. We score uint8 codes directly.
- The AdaptiveBudget module (UC-15) is our equivalent of per-head adaptive bitwidth — it's already built, just needs ESP32 tuning.
- **Blog/paper title opportunity:** "Compressed-Domain Attention: Scoring uint8 Keys Without Dequantization on ESP32-S3"

---

## 5. Rust Crates — Are We Filling a Gap on crates.io?

### What exists on crates.io

| Crate | Downloads | What it does | Compressed scoring? | no_std? |
|-------|-----------|--------------|---------------------|---------|
| **compressed-scorer** (ours) | 2 | Codec-agnostic compressed-domain scoring | ✅ Direct scoring without decompression | ✅ |
| **turbo-quant** (ours) | 5,843 | Polar/QJL vector compression | ✅ (via our adapter) | ❌ (nalgebra) |
| **fib-quant** (ours) | 111 | Radial-angular codebook quantization | ✅ (via our adapter) | ❌ (nalgebra) |
| **tq-kv** | 203 | TurboQuant KV cache compression | ❌ Dequantize-then-matmul | ❌ |
| **qatq** | 67 | Quaternion tensor KV compression | ❌ | ❌ |
| **quillcache** | 26 | KV cache control plane | ❌ | ❌ |
| **ruvector-pq-search** | 12 | PQ with ADC for ANN search | ✅ PQ lookup tables | ❌ |
| **qjl-sketch** | 71 | QJL sign-based vector compression + scoring | ✅ Sign-based scoring | ❌ |
| **diskann-rs** | 1,992 | DiskANN graph search | ❌ | ❌ |
| **simsimd** | 1,546,161 | Mixed-precision BLAS-like vector math | ❌ Operates on f32 | ❌ |
| **tinyml** | 1,542 | ML model deployment to microcontrollers | ❌ | ❌ |
| **microflow** | 5,221 | TinyML inference engine | ❌ | ❌ |
| **ruvllm-esp32** | 110 | Tiny LLM for ESP32 with INT8/INT4 | ❌ | ❌ |

### Are we filling a gap?

**YES — we are the only crate on crates.io that:**
1. Provides compressed-domain scoring (inner product estimation without decompression)
2. Is no_std compatible
3. Works on ESP32-S3 (xtensa-esp32s3-none-elf)
4. Supports multiple codecs via a trait interface

**ruvector-pq-search** is the only other crate doing compressed-domain scoring (PQ with ADC), but it's not no_std and doesn't target embedded.

**qjl-sketch** does sign-based scoring but is not no_std.

### Download counts

| Crate | Downloads | Notes |
|-------|-----------|-------|
| compressed-scorer | **2** | Just published, essentially zero visibility |
| turbo-quant | 5,843 | Established, decent traction |
| fib-quant | 111 | New |
| tq-kv | 203 | Competitor in KV cache space |

**The 2 downloads on compressed-scorer indicate it was just published and has no marketing/documentation presence yet.** The crate fills a genuine gap but nobody knows it exists.

### Next move

- **The gap is real but downloads are near-zero.** Publishing alone doesn't generate adoption.
- Need: README polish, blog post, and ideally a hardware demo to drive visibility.
- Consider cross-linking from turbo-quant (5,843 downloads) to compressed-scorer.

---

## 6. Edge AI Inference Benchmarks on ESP32-S3

### What people are running on ESP32-S3

| Model/Task | Size | Inference time | Source |
|------------|------|---------------|--------|
| Atome-LM (ternary LM) | ~1M params | ~1 tok/s | TilelliLab/atome-lm |
| TFLite Micro classifiers | <1MB | 10-100ms | Edge Impulse ecosystem |
| Keyword spotting (KWS) | <100KB | ~18ms | Navya-215 (INT8, CMSIS-NN) |
| Fall detection | <500KB | Real-time | TinyML wearable |
| Object detection (ESP32-S3-CAM) | ~1MB | ~100ms | Edge Impulse |
| Fault classification | <100KB | 18ms | INT8 quantized, 91.4% accuracy |

### What's feasible for attention

**ESP32-S3 constraints:**
- 512KB SRAM (internal)
- 8MB PSRAM (external, ~100ns access)
- 240MHz dual-core Xtensa LX7
- No FPU for SIMD vector ops (ESP-NN C kernels exist but Rust lacks intrinsics)
- No cuBLAS, no Tensor Cores

**Feasible attention sizes:**
- Head dim (D): 32-64 (typical small model)
- Sequence length (N): 64-256 (short context)
- Top-K: 4-16
- Memory per key (compressed): D bytes (32-64 bytes) vs D*4 bytes (128-256 bytes) for f32
- Total attention cache (256 tokens, 64-dim): 16KB compressed vs 64KB f32

### How our work compares

**Nobody is running transformer attention on ESP32-S3 in production.** The TinyML ecosystem is all CNNs and small classifiers. Atome-LM runs a tiny ternary LM but doesn't use compressed attention.

**Our CompressedAttentionCache<64, 256, 8> would use:**
- Keys: 256 × (64 + 4) = 17,408 bytes (17KB) — fits in SRAM
- Values: 256 × 64 × 4 = 65,536 bytes (64KB) — needs PSRAM
- Total: ~82KB — feasible on ESP32-S3 with PSRAM

vs. dense f32:
- Keys: 256 × 64 × 4 = 65,536 bytes (64KB)
- Values: 256 × 64 × 4 = 65,536 bytes (64KB)
- Total: ~128KB — also feasible but 1.56x more memory traffic from PSRAM

### Next move

- **The ESP32-S3 can handle attention at 64-dim, 256 tokens.** This is the demo to build.
- The memory savings (3.56x on keys) translate to speed on PSRAM-bound hardware because PSRAM reads at ~100ns dominate.
- **Build UC-18: Flash CompressedAttentionCache<64, 256, 8> to ESP32-S3, benchmark latency vs Int4KvCache, produce a receipt.**

---

## 7. Competitive Positioning of compressed-scorer on crates.io

### Direct competitors

| Crate | Downloads | Relationship |
|-------|-----------|-------------|
| **compressed-scorer** (ours) | 2 | The crate in question |
| **ruvector-pq-search** | 12 | Closest functional competitor — PQ with ADC, but not no_std |
| **qjl-sketch** | 71 | QJL scoring, but not no_std |
| **tq-kv** | 203 | KV cache compression, but dequantizes before scoring |
| **qatq** | 67 | KV cache compression, but not compressed-domain scoring |

### Indirect competitors

| Crate | Downloads | Relationship |
|-------|-----------|-------------|
| **turbo-quant** (ours) | 5,843 | Our own codec — compressed-scorer wraps it |
| **simsimd** | 1,546,161 | Fast f32 vector math — not compressed, but sets the bar for speed |
| **diskann-rs** | 1,992 | Graph-based search — not compressed scoring |
| **embedvec** | 1,296 | Vector DB with HNSW — not compressed scoring |

### Positioning assessment

**We are filling a genuine gap.** No crates.io crate provides:
1. Codec-agnostic compressed-domain scoring (trait-based)
2. no_std / ESP32 compatibility
3. Attention cache with top-K-only decompression

**But the gap is invisible** (2 downloads). The crate needs:
- A compelling README with benchmark numbers
- A blog post or Twitter/Mastodon announcement
- Cross-references from turbo-quant (5,843 downloads) and semantic-memory
- Ideally, a hardware demo that demonstrates the unique value proposition

---

## Summary: What Exists vs What We Have

### The competitive landscape map

```
                    GPU Server Quantization          Edge/Microcontroller
                    ┌─────────────────────┐         ┌─────────────────────┐
 Weight quant:      │ AWQ, GPTQ, QServe   │         │ Atome-LM (ternary)  │
                    │ QuaRot, SpinQuant    │         │ (nothing compressed) │
                    ├─────────────────────┤         ├─────────────────────┤
 KV cache quant:    │ KIVI (2-bit)         │         │ ❌ NOTHING EXISTS   │
                    │ QServe (KV4)         │         │   (our gap)          │
                    │ tq-kv (TurboQuant)   │         │                     │
                    ├─────────────────────┤         ├─────────────────────┤
 Compressed-domain  │ ScaNN (ADC)          │         │ ❌ NOTHING EXISTS   │
 scoring:           │ FAISS PQ (ADC)       │         │   (our gap)          │
                    │ ruvector-pq-search   │         │ CompressedAttention  │
                    │                      │         │   Cache (OURS)       │
                    ├─────────────────────┤         ├─────────────────────┤
 Token/prompt:      │ LLMLingua            │         │ (nothing)            │
                    └─────────────────────┘         └─────────────────────┘
```

### Key findings

1. **No one scores compressed vectors without decompression on embedded.** This is our unique contribution.
2. **No one has compressed attention caches on microcontrollers.** CompressedAttentionCache is novel.
3. **The GPU path is a dead end** — cuBLAS beats us 4-6x. Don't compete there.
4. **PQ/ADC is the established approach for compressed retrieval** but no one has made it no_std.
5. **KIVI and QServe validate 2-4 bit KV cache quantization** — our approach is more radical (no dequantization at all).
6. **crates.io has a real gap** — no no_std compressed-domain scoring crate exists besides ours.
7. **The crate has 2 downloads** — the gap is real but invisible. Needs marketing/demo.

---

## Recommendation: Next Best Path Forward

### Priority 1: ESP32 Hardware Demo (UC-18, 3-5 days)

**Why:** This is the single highest-ROI action. It validates the entire project with hardware evidence, demonstrates the one domain where PerDim wins on speed (not just memory), and produces a compelling artifact for marketing the crate.

**What to do:**
1. Wire `CompressedAttentionCache<64, 256, 8>` into `ri-esp-llm`
2. Flash to ESP32-S3 hardware
3. Benchmark: compressed attention latency vs Int4KvCache (dequantize-then-score)
4. Measure: PSRAM reads, cycle count, attention output cosine vs f32 ground truth
5. Produce a receipt with quality + performance numbers

**Expected outcome:** "Compressed-domain attention on ESP32-S3: 3.56x less memory traffic, X% faster, cosine > 0.995"

### Priority 2: Retrieval Benchmark (UC-09, 1-2 days)

**Why:** The claim "PerDim for retrieval" needs evidence. A head-to-head benchmark against HNSW and brute-force will show exactly where compressed-domain scoring wins.

**What to do:**
1. Add PerDim path to `hnsw-bench`
2. Benchmark on synthetic + real embedding corpora (768-dim, 1K-100K vectors)
3. Compare: recall@10, latency, memory
4. Produce receipt

### Priority 3: semantic-memory PerDim Integration (UC-01, 2-3 days)

**Why:** The `PerDimCandidateOnly` policy exists but falls back to brute-force. Making it actually work would unlock Gloss (UC-06) and Recall-Coding (UC-07) automatically. This is the "platform play."

**What to do:**
1. Store per-dim compressed artifacts in the SQLite DB (min/max + uint8 codes per embedding)
2. Wire `per_dim_vector_outcome()` to use `CompressedScorerAdapter::per_dim()` for candidate generation
3. Exact f32 rerank on top-K (same pattern as TurboQuant path)

### Priority 4: Blog Post + README Polish (1 day)

**Why:** 2 downloads means nobody knows the crate exists. A blog post with the ESP32 demo numbers would generate visibility.

**Title:** "Compressed-Domain Attention: Scoring uint8 Keys Without Dequantization on ESP32-S3"

### What NOT to do

- **Don't try to compete on GPU.** cuBLAS is unbeatable for small matmuls. The per-dim scorer loses 4-6x.
- **Don't build PQ support yet.** It's a different compression paradigm and adds complexity. PerDim's simplicity is a feature for embedded.
- **Don't optimize the scoring kernel further without hardware evidence.** Get the demo first, then optimize what's actually slow.
- **Don't add quant-governor PerDim profile (UC-04) before the demo.** It's 1 day of work but produces no evidence. Ship the demo first.

### The 10-day plan

| Day | Action | Output |
|-----|--------|--------|
| 1-2 | Wire CompressedAttentionCache into ri-esp-llm, flash to ESP32-S3 | Working hardware demo |
| 3 | Benchmark: compressed vs Int4KvCache latency, memory, quality | Receipt with numbers |
| 4-5 | Add PerDim to hnsw-bench, benchmark vs HNSW + brute-force | Retrieval benchmark receipt |
| 6-8 | Implement semantic-memory PerDim artifacts in DB, wire candidate generation | Working PerDimCandidateOnly path |
| 9 | Write blog post with ESP32 demo + retrieval benchmark | Published blog post |
| 10 | Polish README, cross-link from turbo-quant, update crate metadata | crates.io visibility |