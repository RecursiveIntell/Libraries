# Compressed-Domain Scoring: Application Domain Analysis

Date: 2026-06-30

## The Core Question

For each application, identify the bottleneck:
- **Compute-bound**: Speedup only if INT8 Tensor Core > FP16 cuBLAS (H100 test needed)
- **Memory-bandwidth-bound**: Speedup from fewer bytes read (int8 = 4x less than FP32)
- **Memory-capacity-bound**: More vectors fit in same RAM/VRAM (compression ratio)

## Domain Analysis

### 1. LLM Inference — KV Cache Compression
**Bottleneck: Memory capacity + bandwidth**

For a 7B model, 32K context, FP16 KV cache:
- 32 layers × 32 heads × 128 dim × 2 (K+V) × 32000 tokens × 2 bytes = ~512 MB per user
- H100 80GB: ~156 concurrent users
- At int8 KV: ~256 MB per user → 312 users (2x more)
- At 4-bit KV: ~128 MB per user → 625 users (4x more)
- At 2-bit KV: ~64 MB per user → 1250 users (8x more)

**How compressed scoring helps:**
- Current approaches (KIVI, QServe) quantize KV but dequantize before attention
- Compressed-domain scoring skips dequantization entirely
- On H100: INT8 Tensor Core matmul for scoring → potentially 2x faster scoring + 2x less memory
- Combined with paged attention (vLLM): compressed pages = more pages in same VRAM

**What exists:**
- KIVI (ICML'24, 413 stars): 2-bit KV quantization, dequantizes before attention
- QServe (W4A8KV4): INT4 KV, dequantizes for FP16 attention
- vLLM PagedAttention: memory management, no compression
- SGLang: RADIX tree attention, no KV compression

**What we'd change:**
- Score KV cache codes directly (int8/4/2/1-bit) via Tensor Core matmul
- Only decode top-k values for exact softmax
- Nobody does this — every existing system dequantizes before attention

**Expected improvement:**
- 2-8x more concurrent users (compression ratio dependent)
- 1.5-2x faster scoring if INT8 Tensor Core works (H100 test needed)
- 4-8x longer context on same hardware

**What's needed to prove:**
- H100 benchmark: INT8 TC kernel vs FP16 cuBLAS for KV scoring
- End-to-end generation quality test (not just forward-pass PPL)
- Integration with vLLM or SGLang

**Competitive landscape:**
- Direct: KIVI, QServe, KVPress — all dequantize before scoring
- Indirect: Flash Attention (reduces memory but doesn't compress)
- Nobody scores compressed KV without decompression

---

### 2. RAG / Semantic Retrieval
**Bottleneck: Memory capacity (large corpora) + bandwidth (scan speed)**

For 1M documents at dim=768:
- FP32: 1M × 768 × 4 = 3 GB
- FP16: 1.5 GB
- int8: 768 MB
- 4-bit: 384 MB
- 1-bit: 96 MB

**How compressed scoring helps:**
- Pre-filter: compressed-score all 1M docs, take top-1000, exact-rerank
- 4-bit: 8x less memory, scan 8x faster (memory-bandwidth-bound)
- 1-bit: 32x less memory, fits entire corpus in L1/L2 cache territory

**What exists:**
- FAISS: Product Quantization (PQ) with ADC (Asymmetric Distance Computation)
  - PQ IS compressed-domain scoring — they score PQ codes without full decompression
  - Our approach is different: per-dimension quantization vs subvector quantization
- ScaNN: Anisotropic quantization, also scores compressed
- HNSW: Graph-based, uses FP32/FP16 for scoring
- Qdrant: Scalar quantization (int8), scores compressed
- Milvus: PQ + IVF, scores compressed

**What we'd change:**
- INT8 Tensor Core acceleration for scoring (FAISS/Qdrant don't use Tensor Cores)
- no_std version for embedded RAG
- Simpler than PQ (no codebook training needed for per-dim)

**Expected improvement:**
- Tensor Core scoring: 2x faster than FP16 scan (if H100 test works)
- No codebook training needed (per-dim is calibration-free for the scoring path)
- Embedded RAG: only viable compressed scorer for ESP32

**What's needed to prove:**
- Benchmark vs FAISS PQ on same corpus (recall@10, latency, memory)
- Benchmark vs Qdrant scalar quantization
- H100 Tensor Core scoring benchmark

**Competitive landscape:**
- FAISS PQ: mature, widely used, scores compressed — but no Tensor Core acceleration
- Qdrant: int8 scalar quantization, scores compressed — but Rust server, not no_std
- ScaNN: Google's SOTA, anisotropic quantization — not open source for scoring path
- Our differentiator: Tensor Core acceleration + no_std for embedded

---

### 3. Recommendation Systems
**Bottleneck: Compute (millions of items) + latency (real-time)**

For user-item matching:
- User embedding (dim=256) scored against 10M item embeddings
- Must return top-100 in <50ms
- Currently: approximate NN (HNSW, IVF) with FP32 scoring

**How compressed scoring helps:**
- Compressed item embeddings in RAM: 10M × 256 × 1 byte = 2.5 GB (int8) vs 10 GB (FP32)
- Tensor Core batched scoring: 10M items in one matmul launch
- Pre-filter top-1000 compressed, exact-rerank

**What exists:**
- Meta's embedding-based retrieval: PQ + IVF
- Pinterest's PinSage: graph + embeddings, FP32 scoring
- Spotify's ANNOY: tree-based, FP32 scoring
- Netflix: HNSW with FP32

**What we'd change:**
- INT8 Tensor Core scoring could replace the ANN scan step
- Simpler than PQ (no codebook training)
- 4x less memory for item embeddings

**Expected improvement:**
- 2x faster scan (Tensor Core INT8 vs FP16)
- 4x less item embedding memory
- But: HNSW/IVF already reduce scan set — compressed scoring of full corpus may not be faster than ANN + FP32 scoring of small candidate set

**What's needed to prove:**
- Benchmark vs HNSW + FP32 on 10M item corpus
- Latency measurement (p50, p95, p99)
- H100 Tensor Core benchmark

**Competitive landscape:**
- FAISS IVF-PQ: the standard for large-scale recsys
- HNSW: the standard for low-latency recsys
- Our advantage: Tensor Core acceleration, no codebook training
- Our disadvantage: no graph index (HNSW) or partitioning (IVF)

---

### 4. Agent Memory
**Bottleneck: Memory capacity (grows over time) + latency (real-time recall)**

An AI agent accumulating experiences:
- Each experience: embedding (dim=768) + metadata + content
- After 1 year: 100K experiences × 768 × 4 = 307 MB (FP32)
- At int8: 77 MB. At 4-bit: 38 MB. At 1-bit: 10 MB.

**How compressed scoring helps:**
- Store all experience embeddings compressed
- Score compressed for initial candidate selection
- Exact-rerank top-k for final selection
- Agent can run longer without memory overflow

**What exists:**
- semantic-memory (RecursiveIntell): SQLite + HNSW + brute-force, SQ8 quantization
- MemGPT: memory management for LLMs, no compression
- LangChain vector stores: FAISS/Chroma/Pinecone, FP32 or PQ

**What we'd change:**
- Compressed candidate generation in semantic-memory (already wired: PerDimCandidateOnly)
- Store compressed codes alongside FP32 in SQLite
- Score compressed for initial scan, exact-rerank for final

**Expected improvement:**
- 4-8x more memories in same storage
- Faster initial scan (fewer bytes)
- Already integrated in code (DerivedVectorBackendPolicy::PerDimCandidateOnly)

**What's needed to prove:**
- Benchmark on real semantic-memory fact store
- Recall@10 vs brute-force FP32
- Latency vs HNSW

---

### 5. Deduplication
**Bottleneck: Compute (O(n²) comparisons) + memory capacity**

For 10M documents:
- Pairwise comparison: 10M × 10M / 2 = 5×10¹³ comparisons — infeasible at FP32
- With LSH or clustering: reduce to O(n) candidates
- Compressed scoring: scan compressed embeddings, find near-duplicates

**How compressed scoring helps:**
- Store all 10M embeddings compressed (int8: 7.7 GB vs 30.7 GB FP32)
- Score compressed for candidate generation
- Only decode near-duplicates for exact comparison

**What exists:**
- MinHash / LSH: standard for dedup, doesn't use embeddings
- SimHash: bit-level similarity, very fast but less accurate
- Embedding-based: FP32 cosine, expensive at scale

**What we'd change:**
- Embedding-based dedup at scale via compressed scoring
- More accurate than MinHash/LSH (uses full embedding)
- Faster than FP32 cosine (fewer bytes, Tensor Core)

---

### 6. Clustering (k-means assignment)
**Bottleneck: Compute (n × k comparisons per iteration)**

For n=10M points, k=1000 clusters, dim=256:
- Each iteration: 10M × 1000 = 10 billion inner products
- FP32: 10B × 256 × 4 bytes = 10 TB memory traffic per iteration
- int8: 2.5 TB (4x less)
- 4-bit: 1.25 TB (8x less)

**How compressed scoring helps:**
- Store points compressed, centroids compressed
- Score compressed for assignment step
- Only decode for centroid update step (once per iteration)

**What exists:**
- FAISS k-means: FP32 with mini-batch
- scikit-learn: FP32, limited to small datasets
- GPU k-means: FP16 Tensor Core (cuML)

**What we'd change:**
- INT8 Tensor Core k-means assignment: 2x faster than FP16
- 4-8x less memory for point storage

---

### 7. Multi-Modal Retrieval (CLIP)
**Bottleneck: Memory capacity (image corpus) + latency**

CLIP: image embeddings (dim=512) + text embeddings (dim=512)
- 100M images × 512 × 4 = 200 GB (FP32) — needs SSD with page faults
- int8: 50 GB — fits in RAM on large server
- 4-bit: 25 GB — fits in RAM on commodity server
- 1-bit: 6.25 GB — fits on GPU

**How compressed scoring helps:**
- Compressed image embeddings in GPU VRAM
- Text query scored against compressed images
- Top-k decoded for exact rerank
- 1-bit: entire 100M image corpus fits on single GPU

---

### 8. Federated / Edge Search
**Bottleneck: Network bandwidth + device memory**

Multiple ESP32/mobile nodes with local embeddings:
- Each node stores compressed embeddings (int8/4-bit)
- Query scored locally on compressed codes
- Only top-k results sent to server (not all embeddings)
- Reduces network traffic by Nx (N = number of nodes)

**How compressed scoring helps:**
- On-device: compressed scoring on ESP32 (proven: 3.64x faster)
- Network: only top-k results sent, not raw embeddings
- Privacy: compressed codes are harder to reverse than FP32 vectors

---

### 9. Mixture-of-Experts Routing
**Bottleneck: Compute (per-token routing decision)**

MoE: each token scored against N expert embeddings
- Mixtral 8x7B: 8 experts, dim=4096 — small N, not a bottleneck
- DeepSeek-MoE: 64+ experts — compressed routing could help
- Future: 1000+ expert models — compressed routing essential

**How compressed scoring helps:**
- Expert embeddings compressed (int8)
- Token embedding scored against compressed expert codes
- Tensor Core batched scoring for large expert counts

---

### 10. Vector Database Compression
**Bottleneck: Memory capacity + query latency**

| System | Compression | Scores compressed? | Tensor Core? | no_std? |
|--------|-------------|-------------------|--------------|---------|
| FAISS PQ | 8-64x | Yes (ADC) | No | No |
| FAISS SQ8 | 4x | Yes | No | No |
| Qdrant | 4x (int8) | Yes | No | No |
| Milvus | 8x (PQ) | Yes | No | No |
| Ours | 4-25x | Yes | Yes (H100) | Yes |

**What we'd change:**
- Tensor Core accelerated compressed scoring (nobody does this)
- no_std for embedded vector search
- Simpler than PQ (no codebook training for per-dim)

---

### 11. Scientific Computing
**Bottleneck: Memory capacity (high-dimensional data)**

- Genomics: gene expression vectors (dim=20K+), find similar profiles
- Chemistry: molecular fingerprints (dim=1024+), find similar compounds
- Physics: particle collision embeddings, find similar events

All involve: store many high-dimensional vectors, find nearest neighbors.
Compressed scoring = more vectors in memory, faster scan.

---

### 12. Long-Running Conversations
**Bottleneck: Memory capacity (grows with conversation length)**

An LLM agent in a 10K-turn conversation:
- Each turn generates KV cache entries
- After 10K turns: KV cache = 10K × 32 layers × 32 heads × 128 dim × 2 bytes = ~2.5 GB
- Compressed: 625 MB (int8), 312 MB (4-bit), 156 MB (2-bit)
- Agent can run 4-16x longer before hitting memory limits

---

## Summary: Where It Helps Most

### Tier 1 — Transforms the field (if H100 test works):
1. **LLM serving**: 2-8x more users per GPU → direct cost reduction
2. **Vector databases**: Tensor Core compressed scoring → faster queries
3. **Long-context LLM**: 4-16x longer context on same hardware

### Tier 2 — Significant improvement:
4. **RAG at scale**: 4-32x more documents in RAM
5. **Agent memory**: 4-8x more experiences stored
6. **Recommendation systems**: 2x faster scan, 4x less memory
7. **Multi-modal retrieval**: 100M image corpus on single GPU

### Tier 3 — Useful but not transformative:
8. **Deduplication**: faster embedding-based dedup
9. **Clustering**: faster k-means assignment
10. **MoE routing**: helps for 1000+ expert models
11. **Scientific computing**: more vectors in memory
12. **Federated/edge**: privacy + bandwidth savings

### Already proven:
- **ESP32/embedded**: 3.64x faster (only viable compressed scorer)
- **Memory compression**: 4-25x (proven at all bit widths)
- **Quality**: cosine > 0.99 at 8-bit (proven)

### Needs H100 to prove:
- **INT8 Tensor Core speed**: 2x faster than FP16 cuBLAS (theoretical)
- **End-to-end LLM serving**: compressed KV + top-k + decode pipeline
- **Large-scale retrieval**: 1M+ document scan with Tensor Core

### The honest truth:
If H100 INT8 Tensor Core scoring beats FP16 cuBLAS, this changes LLM serving economics. If it doesn't, this is "just" a memory optimization with an ESP32 niche. The H100 test is the pivotal experiment.