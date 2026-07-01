# ProveKV-Informed Compressed Cache Architecture V2 Plan

> For Hermes: this is a research + architecture plan, not an implementation completion claim. Use `subagent-driven-development` or TDD only after the operator explicitly asks to implement.

Goal: take the compressed-cache architecture beyond the current `CompressedScorer` / semantic-memory candidate path by reusing the strongest ProveKV/poly-kv ideas: shared immutable pools, hot agent shells, receipts, PPL validation, and multi-agent amortization.

Architecture: treat compressed KV/cache/memory artifacts as active, queryable substrates. The system should score directly over compressed shared pools and per-agent shells, progressively refine only ambiguous candidates, decode only selected values, and emit receipts that separate candidate selection, exact fallback, and verified model-quality claims.

Primary repos inspected:
- Active workspace: `/home/sikmindz/Coding/Libraries`
- Active poly-kv: `/home/sikmindz/Coding/Libraries/poly-kv`
- Archived proveKV evidence: `/home/sikmindz/kv-lossless-11x`
- Current compressed scorer: `/home/sikmindz/Coding/Libraries/compressed-scorer`
- Prior next-frontier note: `/home/sikmindz/Coding/Libraries/docs/plans/2026-06-29-compressed-cache-next-frontier.md`

---

## 1. Current-state evidence

Fresh checks run 2026-06-29:

```bash
cargo check --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml
# passed

cargo check -p compressed-scorer --no-default-features --features no_std
# passed

cargo check -p quant-eval
# passed

cargo check -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'
# passed
```

Recently completed in the current session:

```bash
cargo test -p compressed-scorer
# passed: 11 tests

cargo test -p scr-runtime-compression
# passed: 26 tests, 1 doctest passed, 1 ignored

cargo test -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec'
# passed, including search_tests: 45 passed

cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
# passed
```

Important current code facts:
- `compressed-scorer/src/attention_cache.rs` already scores compressed keys through `CompressedScorer`, softmaxes approximate logits, and decodes only top-k values.
- `semantic-memory/src/search.rs` now supports compressed candidate scoring before exact rerank and allows explicit compressed-only mode when configured.
- `poly-kv/src/shell.rs::AgentShell::attention_topk` still decompresses pool and shell keys before scoring. This is the highest-ROI wiring gap because it contradicts the newer compressed-scorer direction.
- `poly-kv/src/pool.rs` already builds compact batched layer blocks through `encode_batch_compact` when available.
- `poly-kv/src/policy.rs` is still hard-coded around `fib_k4_n32` shared tier and `turbo_8bit` shell tier.
- `poly-kv/src/manifest.rs` and `poly-kv/src/receipt.rs` already provide the right receipt/manifests foundation but need richer quality/uncertainty/runtime receipts.

---

## 2. ProveKV lessons worth preserving

### 2.1 Two-tier memory is real

ProveKV/poly-kv’s strongest idea is not just compression. It is tiering:

| Tier | Role | Old implementation | Why keep it |
|---|---|---|---|
| shared/cold pool | common prompt/context/KV | fib-quant | amortizes across agents/sessions |
| hot shell | agent-private recent tokens | turbo-quant | preserves correctness-sensitive deltas |
| exact fallback | authority path | raw/f32 reconstruction | protects claims and correctness |

This maps cleanly to the new stack:
- shared pool = long-lived compressed memory/cache page;
- agent shell = writable overlay / recent context;
- scorer = unified query path over both;
- fallback = exact rerank or exact model-quality oracle depending layer.

### 2.2 ProveKV had real PPL evidence

Archived `kv-lossless-11x` receipts are stronger than many synthetic vector claims:

Primary SmolLM2 run:
- Model: `HuggingFaceTB/SmolLM2-1.7B-Instruct`
- Corpus: `wikitext-2`, 1024 tokens
- Oracle PPL: `4.7607620871`
- Roundtrip PPL: `4.7607620871`
- Delta PPL: `+0.00%`
- Compression ratio: `11.1304x`
- Pool size: `36,175,872` bytes
- Artifact: `/home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json`

Multi-agent Qwen2.5-0.5B scaling:
- N=2: 1.8x memory reduction
- N=3: 2.69x
- N=4: 3.59x
- N=6: 5.39x
- N=8: 7.19x
- Deltas were reported as bit-exact/lossless within fp16 numerical noise in the summary.
- Artifact: `/home/sikmindz/kv-lossless-11x/results/bench/multi_agent/qwen2.5-0.5b/scaling_summary.json`

This should become the standard evidence shape for the new architecture. Synthetic top-k overlap is useful, but PPL / forward-pass equivalence is the model-quality receipt.

### 2.3 The old hot-tier problem is not algorithmic; it is wire format

Archived `hot_tier_summary.json` says the hot shell quality was PPL-invariant across b=2/4/8, but shell storage was bloated by JSON envelope overhead.

Key finding from archived summary:
- Shell tier quality was invariant on Qwen0.5B and SmolLM2-1.7B.
- Long agent tails became memory losses because shell blocks used a bloated JSON wire format.
- When shared fraction was high (95%), two-agent SmolLM2 had 4.97x memory reduction.

Interpretation:
- Do not redesign hot tier first.
- Pack hot-tier payloads first.
- Then benchmark again.

### 2.4 GPU lesson: batch or don’t bother

Archived GPU path on GTX 1070:
- Hadamard-only win: 2.5–2.7%.
- Hadamard + codebook GPU win: 1.5–2.4%.
- Kernel parity passed; integration lost to H2D/D2H overhead.

Implication for V2:
- Do not offload tiny codec calls.
- Build page/batch-level scorer kernels: prepared query + compressed page -> top-k indices/scores.
- Keep data resident on GPU or do not use GPU.

### 2.5 Receipts were the right instinct

ProveKV already had:
- pool manifests;
- build receipts;
- shell materialization receipts;
- fallback receipts;
- injection receipts.

V2 should not throw this away. It should add:
- compressed-attention quality receipts;
- progressive scoring receipts;
- uncertainty/fallback receipts;
- side-channel/isolation receipts;
- PPL replay receipts.

---

## 3. External research synthesis

Relevant public work:

| Area | Systems/papers | What they prove | What they do not give us |
|---|---|---|---|
| KV quantization | KIVI, KVQuant, HyperQuant, Block-GTQ, KVarN, TensorRT-LLM TurboQuant4, llama.cpp KV qtypes | low-bit KV is real | portable Rust scorer shared with retrieval/edge |
| retention/eviction | H2O, StreamingLLM, PyramidKV, SnapKV, RazorAttention, CompressKV | token/head/layer importance is non-uniform | ProveKV-style receipted shared pool + shell semantics |
| query-aware sparse attention | Quest | token importance depends on query | codec-agnostic compressed scorer substrate |
| offload/reuse | CacheGen, InfiniGen-like systems | KV reuse and transport matter | local-first bitemporal/provenance integration |
| compressed vector search | FAISS/PQ/IVF-PQ | compressed-domain candidate generation is mature | transformer KV + semantic-memory unified path |
| side-channel risk | KV cache/RAG side-channel work | cache access can leak retrieved docs | single-tenant assumptions must be explicit |

The unique opportunity remains:

```text
shared compressed pool + hot shell
        -> query-aware compressed scorer
        -> progressive refine / top-k decode
        -> exact fallback / PPL replay receipts
        -> semantic-memory / agent runtime provenance
```

---

## 4. New architecture: ProveKV V2 as a compressed working-memory runtime

### 4.1 Core abstractions

#### `CompressedPage`

A page is the unit of compressed scoring and hardware transfer.

```rust
pub struct CompressedPage<P> {
    pub page_id: PageId,
    pub layer: u32,
    pub head: u32,
    pub token_range: core::ops::Range<u32>,
    pub role: PageRole,
    pub codec_profile_digest: String,
    pub payload: P,
    pub exact_shadow_digest: Option<String>,
}
```

Roles:
- `SharedCold`
- `AgentHot`
- `RecentGuard`
- `SinkGuard`
- `RetrievalHeadLongRange`
- `ExactFallback`

#### `CompressedWorkingSet`

A unified view over shared pool + shell + guards.

```rust
pub struct CompressedWorkingSet<S: CompressedScorer> {
    pub shared_pages: Vec<CompressedPage<S::Compressed>>,
    pub shell_pages: Vec<CompressedPage<S::Compressed>>,
    pub guard_pages: Vec<CompressedPage<S::Compressed>>,
    pub scorer: S,
    pub policy: CacheRuntimePolicy,
}
```

#### `ProgressiveCompressedScorer`

Current `CompressedScorer` is one-stage. V2 needs staged confidence.

```rust
pub enum ScoreStage {
    Coarse,
    Refined,
    ExactFallback,
}

pub struct ScoreWithUncertainty {
    pub score: f32,
    pub error_bound: Option<f32>,
    pub stage: ScoreStage,
}

pub trait ProgressiveCompressedScorer: CompressedScorer {
    fn score_coarse(&self, prepared: &Self::Prepared, compressed: &Self::Compressed) -> ScorerResult<ScoreWithUncertainty>;
    fn refine_candidates(&self, prepared: &Self::Prepared, candidates: &mut [ScoredCandidate]) -> ScorerResult<()>;
}
```

#### `CacheRuntimePolicy`

Move beyond hard-coded fib/turbo policy.

```rust
pub struct CacheRuntimePolicy {
    pub shared_policy: TierPolicy,
    pub shell_policy: TierPolicy,
    pub guard_policy: GuardPolicy,
    pub fallback_policy: FallbackPolicy,
    pub security_policy: CacheSecurityPolicy,
}

pub struct TierPolicy {
    pub codec: String,
    pub default_bits: u8,
    pub max_decode_per_query: usize,
    pub role_budgets: Vec<RoleBudget>,
}
```

### 4.2 Query path

New target path:

```text
query vector
  -> prepare query once
  -> score shared compressed pages coarsely
  -> score hot shell compressed pages coarsely
  -> include guard tokens/pages unconditionally
  -> refine only margin-band candidates
  -> softmax over selected/refined logits
  -> decode only selected values
  -> exact fallback if uncertainty threshold fails
  -> emit receipt
```

### 4.3 Why this is stronger than current systems

Public runtimes usually integrate compressed KV inside one inference engine.

V2 instead makes compressed working memory a standalone substrate:
- usable by semantic-memory retrieval;
- usable by local inference KV/cache;
- usable by multi-agent shared context;
- portable to ESP32-S3 / UNO Q / GTX 1070;
- governed by receipts and exact fallback.

---

## 5. Implementation plan

## Phase 0 — Preserve and upgrade ProveKV evidence

Objective: import the ProveKV evidence shape into active Libraries without claiming new runtime wins.

### Task 0.1: Add ProveKV evidence index

Files:
- Create: `docs/provekv/PROVEKV_EVIDENCE_INDEX.md`
- Reference existing archived paths only; do not copy large artifacts.

Contents:
- SmolLM2 PPL receipt summary.
- Qwen multi-agent scaling receipt summary.
- Hot-tier JSON bloat warning.
- GPU dispatch lesson.
- Claim boundaries.

Gate:
```bash
python3 - <<'PY'
from pathlib import Path
p=Path('/home/sikmindz/Coding/Libraries/docs/provekv/PROVEKV_EVIDENCE_INDEX.md')
assert p.exists()
s=p.read_text()
for term in ['11.1304x','4.7607620871','7.19x','JSON','GTX 1070']:
    assert term in s, term
PY
```

### Task 0.2: Add active architecture glossary

Files:
- Create: `docs/provekv/COMPRESSED_WORKING_MEMORY_GLOSSARY.md`

Define:
- shared pool;
- shell;
- compressed page;
- candidate;
- exact fallback;
- PPL replay;
- score uncertainty;
- guard token;
- retrieval head;
- side-channel boundary.

Gate:
- glossary distinguishes retrieval candidate vs model-quality proof.

---

## Phase 1 — Replace decompress-then-score in `poly-kv::AgentShell::attention_topk`

Objective: make active poly-kv use the same compressed-domain scorer path as `compressed-scorer` instead of decompressing all keys first.

Current issue:
- `/home/sikmindz/Coding/Libraries/poly-kv/src/shell.rs:45-125` decompresses pool keys and shell keys, then dot-products raw vectors.
- This loses the central V2 advantage.

### Task 1.1: Add a failing test proving no full-key decode in attention candidate mode

Files:
- Create or modify: `poly-kv/tests/compressed_attention_path.rs`

Test intent:
- Build a tiny pool + shell.
- Call new `attention_topk_compressed`.
- Assert receipt shows `decoded_keys == 0` and `decoded_values <= top_k`.

Expected fail:
- method/receipt does not exist.

### Task 1.2: Add `AttentionSelectionReceiptV1`

Files:
- Modify: `poly-kv/src/receipt.rs`

Fields:
```rust
pub struct AttentionSelectionReceiptV1 {
    pub schema_version: String,
    pub pool_digest: Digest,
    pub shell_digest: Option<Digest>,
    pub layer: u32,
    pub head: u32,
    pub candidate_count: usize,
    pub refined_count: usize,
    pub decoded_keys: usize,
    pub decoded_values: usize,
    pub exact_fallback: bool,
    pub max_score_error_bound: Option<f32>,
}
```

Gate:
- serde roundtrip test.

### Task 1.3: Add compressed scoring adapter bridge

Files:
- Modify: `poly-kv/src/codec.rs`
- Modify: `poly-kv/src/shell.rs`
- Possibly add: `poly-kv/src/scoring.rs`

Approach:
- Do not reimplement fib/turbo math.
- Reuse `compressed-scorer` where possible.
- For compact batch payloads, either expose page-level score APIs from fib/turbo, or build a temporary page adapter that iterates compressed blocks without decoding.

Gate:
```bash
cargo test --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml --test compressed_attention_path
```

---

## Phase 2 — Compact hot shell wire format

Objective: fix the known ProveKV hot-tier JSON bloat before chasing new algorithms.

Current evidence:
- `hot_tier_summary.json` shows shell quality survived b=2/4/8, but shell bytes were 4.3x larger than raw for long tails due to JSON envelope overhead.

### Task 2.1: Add shell compact payload test

Files:
- Add: `poly-kv/tests/shell_compact_payload.rs`

Test:
- materialize shell with enough tokens;
- assert compact payload path exists;
- assert `shell_size_bytes < raw_size_bytes` for b=8 and b=4.

Expected fail before implementation.

### Task 2.2: Route shell materialization through compact turbo payloads

Files:
- Modify: `poly-kv/src/shell.rs`
- Possibly modify: `poly-kv/src/codec.rs`
- Reuse `turbo-quant::PackedTurboCode` if already sufficient.

Gate:
```bash
cargo test --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml --test shell_compact_payload
```

Acceptance:
- hot tier is not larger than raw for 1024-token shell fixture;
- receipt reports actual serialized byte count, not theoretical ratio.

---

## Phase 3 — Progressive coarse-to-fine scoring

Objective: exploit your different architecture. Do not fully refine/decode every compressed candidate.

### Task 3.1: Extend `compressed-scorer` with staged scoring

Files:
- Modify: `compressed-scorer/src/trait_def.rs`
- Modify: `compressed-scorer/src/candidate.rs`
- Add tests in `compressed-scorer/src/tests.rs`

API:
```rust
pub enum ScoreStage { Coarse, Refined, ExactFallback }

pub struct ScoreWithUncertainty {
    pub score: f32,
    pub error_bound: Option<f32>,
    pub stage: ScoreStage,
}
```

Gate:
```bash
cargo test -p compressed-scorer
cargo check -p compressed-scorer --no-default-features --features no_std
```

### Task 3.2: Add margin-band refinement

Algorithm:
- coarse score all candidates;
- keep top `k * oversample` plus all candidates within `epsilon` of the kth score;
- refine only that set;
- exact fallback only if uncertainty overlaps top-k boundary.

Receipt fields:
- coarse_count;
- margin_band_count;
- refined_count;
- exact_fallback_count;
- decoded_value_count;
- skipped_exact_reason.

Gate:
- deterministic top-k on toy scorer;
- false-safe test where uncertainty forces exact fallback.

---

## Phase 4 — Query-aware compressed sparse attention

Objective: turn compressed cache into queryable attention memory, not just storage.

Research basis:
- Quest: token criticality is query-dependent.
- CompressKV/SnapKV/RazorAttention: retrieval heads and semantic heads matter.
- ProveKV: shared pool + shell lets us score common and private context separately.

### Task 4.1: Add `CompressedWorkingSet`

Files:
- Add: `compressed-scorer/src/working_set.rs`
- Export from `compressed-scorer/src/lib.rs`

Features:
- shared pages;
- shell pages;
- guard pages;
- scorer;
- role metadata;
- `select(query, k, policy) -> SelectionReceipt`.

Gate:
- no_std build still passes.

### Task 4.2: Add guard-token policy

Rules:
- always include recent N tokens;
- always include sink tokens;
- include compressed top-k query-selected tokens;
- optionally include retrieval-head protected tokens.

Gate:
- exact tests prove guard tokens are always selected even when score is low.

### Task 4.3: Add exact attention reference benchmark

Files:
- Add: `quant-eval/src/compressed_attention.rs`
- Add: `quant-eval/tests/compressed_attention_receipt.rs`

Receipt schema:
```json
{
  "schema": "compressed_cache_attention_bench_v1",
  "model_or_fixture": "synthetic_rope_gqa_v1 | captured_llama_kv",
  "codec": "fib_quant | turbo_quant | hyperquant | raw_q4",
  "context_len": 8192,
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

Gate:
- exact full attention reference exists;
- compressed path receipt reports quality and byte savings;
- no pass unless thresholds are declared before running.

---

## Phase 5 — RoPE-aware and role-aware policy

Objective: apply research-backed non-uniform budgets.

### Task 5.1: Add RoPE block budget type

Files:
- Add: `turbo-quant/src/rope_alloc.rs`
- Add tests.

API:
```rust
pub struct RopeBlockBudget {
    pub layer: usize,
    pub head: usize,
    pub block_bits: Vec<u8>,
    pub energy: Vec<f32>,
}
```

Gate:
- deterministic allocation;
- total bit budget matches target;
- high-energy blocks receive >= low-energy blocks.

### Task 5.2: Add `HeadRole` and layer budget policy

Files:
- Modify: `poly-kv/src/policy.rs`
- Modify or add: `quant-governor/src/cache_policy.rs`

Roles:
```rust
pub enum HeadRole { Local, Retrieval, Sink, Semantic, Unknown }
```

Gate:
- policy validation rejects missing budgets;
- retrieval/sink heads cannot be assigned zero budget unless exact fallback is forced.

### Task 5.3: Compare uniform vs role-aware in `quant-eval`

Gate:
- run synthetic RoPE/GQA fixture;
- report logit MAE, top-k overlap, attention output cosine, bytes.

---

## Phase 6 — PPL replay harness revival

Objective: make model-quality receipts first-class again.

### Task 6.1: Port archived `ppl_validate.py` into active tools

Files:
- Add: `tools/provekv_ppl/ppl_validate.py`
- Add: `tools/provekv_ppl/README.md`

Scope:
- no huge artifacts committed;
- script emits state.json + report.md;
- supports SmolLM2/Qwen/TinyLlama small models.

Gate:
- smoke mode runs without compression and writes Phase 0 receipt;
- full mode can be skipped if GPU/model unavailable, but command is documented exactly.

### Task 6.2: Add active state schema

Files:
- Add: `schemas/provekv-ppl-state-v2.schema.json`
- Add validator script: `scripts/validate_provekv_ppl_state.py`

Gate:
```bash
python3 scripts/validate_provekv_ppl_state.py /home/sikmindz/kv-lossless-11x/results/bench/ppl/smollm2-1.7b/wikitext-2/state.json
```

### Task 6.3: Add compressed-attention PPL replay

New experimental path:
- compare full fp16 KV replay;
- compressed roundtrip replay;
- compressed query-aware top-k replay;
- progressive scorer replay.

Acceptance:
- do not claim quality preservation unless PPL delta is within predeclared threshold across at least 3 model/corpus pairs.

---

## Phase 7 — Security/isolation receipts

Objective: bake in the side-channel lesson before multi-agent claims expand.

Risk basis:
- KV cache access patterns can leak retrieval/context in RAG or multi-tenant settings.
- Current architecture is safe only as local single-tenant unless isolation is explicit.

### Task 7.1: Add `CacheSecurityPolicy`

Fields:
- tenant/scope id;
- shared-pool allowed?;
- shell isolation required?;
- access-pattern logging mode;
- flush/noise settings;
- cross-agent reuse constraints.

Gate:
- multi-tenant sharing rejected unless policy explicitly allows it.

### Task 7.2: Add access receipt

Receipt fields:
- page ids touched;
- candidate counts;
- exact fallback count;
- tenant/scope;
- side-channel mode.

Gate:
- tests prove receipt records page access without leaking raw content.

---

## Phase 8 — Hardware scorer kernels, but only after scalar receipts pass

Objective: avoid repeating old GPU transfer-overhead mistake.

Do not start here.

### Task 8.1: CPU SIMD page scorer

Target:
- UNO Q / x86 host first.

Input:
```text
prepared query + compressed page -> top-k scores/indices
```

Gate:
- same top-k as scalar;
- candidates/sec improvement;
- no extra allocations in hot loop.

### Task 8.2: GTX 1070 CUDA page scorer

Rule:
- page/batch-level only;
- keep compressed pages resident;
- no per-token H2D/D2H.

Gate:
- compare scalar vs CUDA over 4K/8K/32K candidate pages;
- report effective GB/s and latency p50/p95;
- kill if transfer dominates.

### Task 8.3: ESP32-S3 static-buffer scorer

Rule:
- no heap in hot path after initialization;
- bounded capacity;
- libm only for f32 exp/sqrt where needed.

Gate:
```bash
cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
```

---

## 6. Priority order

Do in this order:

1. Evidence index + glossary.
2. Fix `poly-kv::AgentShell::attention_topk` to stop decompressing all keys.
3. Compact hot shell wire format.
4. Quant-eval compressed-attention receipt harness.
5. Progressive coarse-to-fine scoring.
6. Query-aware compressed sparse attention working set.
7. RoPE/head/layer policy.
8. PPL replay harness revival.
9. Security/isolation receipts.
10. Hardware kernels.

Reasoning:
- The biggest immediate contradiction is decompress-then-score in active poly-kv.
- The biggest known memory blocker is shell JSON bloat.
- The biggest claim gap is lack of current active PPL replay receipts.
- Hardware acceleration should wait until scalar/page-level receipts prove the algorithm wins.

---

## 7. Claim boundaries

Safe now:
- Active code has a no_std-compatible compressed scorer and attention-cache abstraction.
- Active semantic-memory can use compressed candidate scoring before exact rerank.
- Active poly-kv/proveKV evidence shows prior two-tier shared-pool PPL replay and multi-agent amortization receipts.

Not safe yet:
- better than vLLM/TensorRT/llama.cpp;
- universal KV quality preservation;
- production runtime quality;
- GTX 1070 speedup from V2;
- 7B/13B model improvement;
- multi-tenant safety.

Safe after Phase 4/6 if receipts pass:
- “On fixture/model X, query-aware compressed-cache attention reduced decoded values from N to K while preserving attention output/PPL within threshold Y.”

---

## 8. Kill criteria

Kill or demote a branch if:
- compressed top-k misses exact top-k too often after refinement;
- attention output cosine/MSE fails predeclared thresholds;
- PPL replay degrades beyond threshold on small real models;
- compact shell bytes remain larger than raw after packed payload work;
- CUDA path is still transfer-bound at page-level batch sizes;
- side-channel policy cannot prevent cross-agent leakage in shared-pool mode.

---

## 9. Strategic conclusion

The path forward is not “make another quantizer.”

It is:

```text
ProveKV two-tier pool semantics
+ compressed-scorer queryable compressed substrate
+ semantic-memory exact-fallback/provenance discipline
+ progressive/uncertainty-gated selection
+ PPL replay receipts
= compressed working-memory runtime
```

That is the real architecture. It is different enough from standard KV quantization because it can unify:
- retrieval candidates;
- attention/KV cache candidates;
- shared multi-agent prefix pools;
- agent-private overlays;
- edge/no_std scorer paths;
- exact fallback and bitemporal receipts.

If this wins, the defensible claim is not “we compress KV cache.”

The defensible claim is:

“RecursiveIntell builds a receipted compressed working-memory runtime: shared pools and private shells are queried directly while compressed, refined only when uncertain, decoded only when selected, and replay-validated against exact model-quality receipts.”
