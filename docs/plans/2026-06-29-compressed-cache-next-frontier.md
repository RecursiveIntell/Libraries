# Compressed Cache Next Frontier — 2026-06-29

## Bottom line

The stack is not unique because it compresses KV cache. That exists in KIVI, KVQuant, H2O, PyramidKV, SnapKV, RazorAttention, Quest, CacheGen, HyperQuant, Block-GTQ, KVarN, vLLM, llama.cpp, TensorRT-LLM, and related work.

The stack is different because it treats compressed cache as a queryable scoring substrate, not merely a storage format. The same Rust trait can score compressed semantic-memory candidates and compressed attention/KV entries, select top-k in compressed space, and decode only selected values with exact fallback policy.

This gives a research path that most runtime-specific systems do not expose cleanly: cache/retrieval/attention can share one compressed-domain ranking API.

## Existing public systems and what they cover

- KIVI / KVQuant / HyperQuant / Block-GTQ / KVarN: low-bit KV quantization.
- H2O / StreamingLLM / PyramidKV / SnapKV / CompressKV / RazorAttention: token/head/layer retention policies and retrieval-head observations.
- Quest: query-aware sparse long-context attention.
- CacheGen / InfiniGen-style work: compressed/offloaded KV reuse and transport.
- FAISS/PQ family: compressed-domain approximate vector search, but not transformer KV cache.
- llama.cpp / vLLM / TensorRT-LLM: runtime-integrated quantized KV paths, but not a portable Rust scorer substrate shared with semantic retrieval and no_std embedded paths.

## Core opportunity

Move from:

```text
compressed cache -> dequant inside runtime -> normal attention/search
```

to:

```text
compressed cache/index -> compressed-domain score -> uncertainty/top-k/refine -> decode only necessary values -> exact fallback if policy requires
```

This makes the compressed representation active. It becomes the thing you query, not just the thing you store.

## Highest-ROI extensions

### 1. Progressive coarse-to-fine compressed scoring

Implement a multi-stage scorer pipeline:

1. q2/coarse score over all compressed keys/candidates.
2. refine only the margin band near top-k using q4/q6 or residuals.
3. decode only final top-k values.
4. exact rerank only if configured or uncertainty is high.

Why it matters:
- Most systems choose one quantization level.
- Your trait can expose staged scoring without changing callers.
- This directly reduces memory bandwidth on GTX 1070 / UNO Q and decode work on ESP32-S3.

Files likely touched:
- `compressed-scorer/src/trait_def.rs`
- `compressed-scorer/src/candidate.rs`
- `compressed-scorer/src/attention_cache.rs`
- `scr-runtime-compression/src/compressed_scorer_adapter.rs`
- `quant-governor/src/policy.rs`

Suggested API:

```rust
pub enum ScoreStage { Coarse, Refined, ExactFallback }

pub struct ScoreWithUncertainty {
    pub score: f32,
    pub error_bound: Option<f32>,
    pub stage: ScoreStage,
}

pub trait ProgressiveCompressedScorer: CompressedScorer {
    fn score_coarse(...);
    fn refine_candidates(...);
}
```

Gate:
- Same exact top-1 recovery as current full compressed scorer with fewer decoded/refined candidates.
- Receipt reports coarse_count, refined_count, decoded_count, exact_count.

### 2. RoPE-aware / position-aware bit allocation

Block-GTQ shows that RoPE makes key-cache quantization block-sensitive. High-energy RoPE frequency blocks need more bits.

Why it matters:
- Current fib/turbo paths are mostly RoPE-agnostic.
- Key-cache score quality is what decides whether top-k decode selects the right tokens.
- Position-aware scoring is likely the cleanest quality jump.

Files likely touched:
- `turbo-quant/src/kv.rs`
- `fib-quant/src/kv/compressed_attention.rs`
- `compressed-scorer/src/trait_def.rs`
- maybe new `turbo-quant/src/rope_alloc.rs`

Suggested feature:

```rust
pub struct RopeBlockBudget {
    pub layer: usize,
    pub head: usize,
    pub block_bits: Vec<u8>,
    pub energy: Vec<f32>,
}
```

Gate:
- Compare uniform bits vs RoPE-block bits on synthetic RoPE fixtures and real captured KV if available.
- Metrics: logit MAE, top-k overlap, NIAH-like retrieval hit rate, decode count.

### 3. Head/layer-aware cache policy

RazorAttention, SnapKV, PyramidKV, and CompressKV all point to non-uniform importance:
- lower layers need broader cache;
- upper layers can be smaller;
- retrieval heads matter more for long-range evidence;
- local heads can keep only recent tokens.

Your stack can represent this as policy, not a hardcoded runtime kernel.

Files likely touched:
- `quant-governor/src/policy.rs`
- `turbo-quant/src/kv.rs`
- `compressed-scorer/src/attention_cache.rs`
- possible new `poly-kv` policy module

Suggested policy:

```rust
pub enum HeadRole { Local, Retrieval, Sink, Semantic, Unknown }

pub struct CacheBudgetPolicy {
    pub layer_budget: Vec<usize>,
    pub head_roles: Vec<Vec<HeadRole>>,
    pub bits_by_role: BTreeMap<HeadRole, u8>,
    pub retention_by_role: BTreeMap<HeadRole, RetentionPolicy>,
}
```

Gate:
- Same or better quality at fixed byte budget vs uniform per-layer/per-head cache.

### 4. Query-aware compressed sparse attention

Quest says token criticality depends on query. Your scorer already prepares the query once and scores compressed keys. That is the right seam.

Take it further:
- build a per-layer compressed sidecar over KV keys;
- for each query, retrieve top blocks/tokens from compressed keys;
- run attention only on selected tokens plus recent/sink guard set;
- decode values only for selected tokens.

This is the most direct “read from still-compressed cache” extension.

Gate:
- Attention output cosine/MSE vs full attention.
- Token/s improvement at 4K/8K/16K context.
- KV bytes loaded per token, not just KV bytes stored.

### 5. Error-bounded compressed scoring / safe fallback

Current approximate scoring can select wrong top-k. The next frontier is knowing when it is unsafe.

Add:
- score error calibration tables per codec/profile;
- per-query margin test: if top_k score gap > estimated error bound, skip exact rerank;
- otherwise refine/decode/exact fallback.

Why it matters:
- This converts approximate scoring from a blind heuristic into a governed runtime decision.
- It matches Josh’s provenance-first/evidence-first design language.

Files likely touched:
- `quant-eval`
- `quant-governor`
- `compressed-scorer`
- `semantic-memory/src/search.rs`

Gate:
- False-safe rate measured: cases where policy skipped exact rerank but exact top-k disagreed.
- Must be below declared threshold.

### 6. Cache as a vector database: AlayaDB-like direction, but Rust/local-first

AlayaDB-style work decouples KV cache and attention computation into a vector database-like layer. Your stack is already close from the other direction: semantic-memory retrieval is gaining compressed cache semantics.

Take it further:
- represent KV tokens as typed memory rows with layer/head/position/time metadata;
- use compressed-scorer for both semantic retrieval and KV retrieval;
- persist compressed KV pages with bitemporal receipts;
- allow reuse across prompts/agents if prefix/source authority matches.

This overlaps with PolyKV and semantic-memory. The novelty is local-first provenance plus compressed scoring substrate.

Gate:
- Reuse repeated prompt-prefix KV pages across sessions.
- Report bytes saved, load time saved, quality drift.

### 7. Hardware-specific scoring kernels

The Rust loop proves the architecture. The speed win needs kernels.

Targets:
- CPU SIMD for UNO Q.
- CUDA C kernel for GTX 1070 Pascal: bandwidth-aware, no tensor-core assumptions.
- no_std bounded arrays for ESP32-S3.

Do not start with full model runtime integration. Start with the scorer microkernel:

```text
query prepared once + compressed key page -> top-k indices + approximate scores
```

Gate:
- compare Rust scalar vs CPU SIMD vs CUDA scorer over identical compressed pages.
- report effective GB/s, candidates/s, top-k agreement.

## Implementation order

1. Measurement harness first.
   - Add a captured/synthetic attention-cache benchmark in `quant-eval`.
   - It must report exact attention output error, top-k overlap, decoded_count, bytes_read, latency.

2. Progressive scorer API.
   - Add coarse/refined stages and uncertainty receipts.

3. Query-aware sparse attention.
   - Extend `AttentionCache` to return selected token set plus guard tokens.

4. RoPE-aware bit allocation.
   - Add key-block budgets and compare uniform vs block-aware.

5. Head/layer policy.
   - Add role-aware cache policy after there is a harness to test it.

6. Hardware kernels.
   - Only after scalar receipts prove the algorithm wins.

## What not to claim yet

Do not claim:
- better than vLLM/TensorRT-LLM;
- universal KV cache compression superiority;
- production quality;
- 7B/13B speedup on GTX 1070;
- model-quality preservation.

Safe current claim:
- The stack now has a Rust compressed-domain scoring abstraction that can rank compressed retrieval vectors and compressed attention/KV entries, decode only selected values, and preserve exact fallback policy.

Safe next claim after benchmarks:
- On workload X, compressed-domain query-aware top-k attention reduced value decodes from N to K and preserved output quality within threshold Y.

## Kill criteria

Stop or demote the approach if:
- top-k selected by compressed logits misses exact top-k too often even after refinement;
- exact attention output error remains high at useful byte ratios;
- latency is dominated by Rust allocation/sorting and kernels are required before any measurable win;
- quality only survives on synthetic random vectors.

## Concrete benchmark receipt schema

```json
{
  "schema": "compressed_cache_attention_bench_v1",
  "model_or_fixture": "synthetic_rope_gqa_v1 | captured_llama_kv",
  "codec": "fib_quant | turbo_quant | hyperquant | raw_q4",
  "context_len": 8192,
  "layers": 32,
  "heads": 8,
  "head_dim": 128,
  "cache_bytes_raw_fp16": 0,
  "cache_bytes_compressed": 0,
  "bytes_loaded_per_query": 0,
  "decoded_values": 0,
  "refined_candidates": 0,
  "exact_fallbacks": 0,
  "topk_overlap": 0.0,
  "attention_output_cosine": 0.0,
  "attention_output_mse": 0.0,
  "logit_mae": 0.0,
  "latency_p50_us": 0,
  "latency_p95_us": 0,
  "passed": false,
  "blockers": []
}
```

## Verdict

The next frontier is not another standalone quantizer. It is an evidence-backed compressed cache runtime:

- query-aware;
- progressive coarse-to-fine;
- RoPE/head/layer aware;
- uncertainty-gated;
- exact-fallback capable;
- hardware-kernel backed only after scalar receipts pass.

That is where the stack is different enough to plausibly find gains others miss.
