# Research-to-Libraries Implementation Plan (No HuggingFace, No CUDA)

> **For Hermes:** Use `subagent-driven-development` or one sequential coding agent to implement this plan phase-by-phase. Do not implement HuggingFace integrations or CUDA/GPU-kernel work in this pass.

**Goal:** Turn the recent research sweep into concrete, testable improvements across `fib-quant`, `turbo-quant`, `semantic-memory`, `knowledge-runtime`, `quant-eval`, `llm-output-parser`, and verification crates, while explicitly excluding HuggingFace and CUDA work for now.

**Architecture:** Keep each research result as an optional, receipt-bearing capability behind crate-local APIs. Do not widen public claims until local benchmarks reproduce the external paper direction. Implement algorithmic CPU/Rust surfaces first, then wire benchmark receipts and documentation. Keep semantic-memory as the retrieval authority; keep quantization codecs as sidecar/codec substrates.

**Tech Stack:** Rust 1.75 workspace, Cargo workspace, serde/schemars receipts, deterministic tests, optional benchmark fixtures, semantic-memory research facts.

---

## Scope Boundary

### In scope

1. RoPE-aware KV-cache bit allocation, CPU-side and deterministic.
2. HyperQuant/lattice-inspired quantization experiments where they can be implemented without HuggingFace runtime dependency.
3. Dynamic/vector-index research translated into semantic-memory admission, clustering, and benchmark surfaces.
4. Bitemporal property graph integration at the query/model layer.
5. Agent memory improvements: contextual reinstatement, perspective-bounded recall, forgetting/decay policies.
6. Text-to-Cypher / structured query generation as typed AST + parser utilities, not an LLM service.
7. TREC/RAG-style benchmark harness shape with local fixture support.
8. Formal verification / structured diagnostic localization hooks in verification and forge crates.
9. Documentation and claim boundaries.

### Explicitly out of scope for this pass

1. HuggingFace model loading, datasets API, transformers integration, tokenizers, safetensors, or HF Hub download logic.
2. CUDA kernels, CUDA profiling, GPU backend changes, FlashAttention paths, HEAL GPU reproducibility, H800/A100-specific claims.
3. Public performance claims based only on paper numbers.
4. Publishing crates unless the final validation phase separately verifies package state and dependency publication readiness.

---

## Verified Current State Snapshot

Date: 2026-06-26 16:01:09 local.
Repo path: `/home/sikmindz/Coding/Libraries`.
Git root verified by `git rev-parse --show-toplevel`: `/home/sikmindz/Coding/Libraries`.

Observed state:
- Workspace root `Cargo.toml` has 60+ members including `fib-quant`, `turbo-quant`, `semantic-memory`, `knowledge-runtime`, `quant-eval`, `llm-output-parser`, `bitemporal-runtime`, and verification crates.
- `git status --short` is heavily dirty. It includes many deleted stale root/archive files, modified `fib-quant`, `poly-kv`, `semantic-memory`, `semantic-memory-mcp`, and untracked new modules.
- Important modified/untracked files already present:
  - `fib-quant/src/rotation.rs`
  - `fib-quant/src/kv/codec.rs`
  - `fib-quant/src/kv/stream.rs`
  - `fib-quant/src/residual.rs`
  - `fib-quant/src/scoring.rs`
  - `fib-quant/src/sidecar.rs`
  - `fib-quant/src/wire.rs`
  - `semantic-memory/src/poly_kv_bridge.rs`
  - `semantic-memory-mcp/src/server.rs`
  - `semantic-memory-mcp/src/http_server.rs`
- I did not run `cargo check` or `cargo test` during plan writing. First implementation task must establish a fresh baseline.

Critical warning:
- Do not start feature work until the dirty tree is triaged. There are already many active changes from previous work. A new pass must avoid overwriting or silently mixing them.

---

## Research Inputs Mapped to Crates

| Research item | Meaning for workspace | Primary crates | Pass decision |
|---|---|---|---|
| Block-GTQ / RoPE-aware bit allocation, arXiv:2606.24033 | Key-cache quantization should allocate bits per RoPE 2D frequency block instead of uniform key bits | `fib-quant`, `turbo-quant`, `poly-kv` | Implement CPU/deterministic allocator + tests; no GPU serving claims |
| HyperQuant / lattice quantization, arXiv:2606.23406 | E8/D4/A2/Z lattice-style quantization can become an experimental codec profile | `fib-quant`, `quant-eval`, `turbo-quant` | Prototype codec/profile and local synthetic benchmarks; no HF model PPL |
| ACRONYM dynamic ANNS, arXiv:2606.03151 | Avoid full HNSW rebuild assumptions; add continuous-update benchmark/admission abstractions | `semantic-memory`, `hnsw-bench`, `quant-eval` | Implement portable algorithmic insights only; no CAM hardware claims |
| Helmsman cluster-first ANNS, arXiv:2606.13145 | Search cluster/scope centroids before local index search | `semantic-memory`, `knowledge-runtime` | Add namespace/scope centroid routing as optional path |
| Hubness-aware admission, arXiv:2606.19692 | Detect embeddings that become universal nearest-neighbor hubs and quarantine/downweight them | `semantic-memory` | High ROI; implement first in retrieval quality phase |
| Bitemporal property graphs, arXiv:2111.13499 | Temporal graph edges should be first-class queryable projections | `bitemporal-runtime`, `knowledge-runtime`, `semantic-memory` | Add model/tests/API shape; avoid fake facts |
| RaMem contextual reinstatement, arXiv:2606.22844 | Recall should use episode/context triggers, not only flat similarity | `semantic-memory`, `knowledge-runtime`, `agent-graph` | Implement deterministic context-key scoring and receipts |
| Perspective-bounded memory, arXiv:2606.25632 | ScopeKey is not enough; recall needs perspective/role boundary to prevent overreach | `knowledge-runtime`, `semantic-memory` | Add `PerspectiveKey`/policy layer; fail closed on widening |
| Text-to-Cypher grounded KG generation, arXiv:2606.14325 | Structured graph query generation should be AST-first and validated | `llm-output-parser`, `knowledge-runtime`, `boundary-compiler` | Build parser/AST/validation; no model integration |
| TREC 2025 RAG track | Need standardized retrieval benchmark shape | `quant-eval`, `semantic-memory` | Add fixture format and local harness; no remote dataset download |
| SHERLOC / structured diagnostic localization | Code-repair agents need structured diagnostic location receipts | `forge-pilot`, `verification-adjudication`, `claim-ledger` | Add receipt types and tests |
| Weave of Formal Thought | Formal checks should be part of reasoning/repair loop | `verification-*`, `spec-execution`, `forge-pilot` | Add adapters/gates, not a theorem-prover rewrite |

---

## Phase 0: Tree Safety and Baseline Gates

### Task 0.1: Capture dirty-tree receipt

**Objective:** Freeze the starting state before any feature edits.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/docs/research-implementation/2026-06-26_STARTING_TREE_RECEIPT.md`

**Steps:**
1. Run:
   ```bash
   cd /home/sikmindz/Coding/Libraries
   git status --short > /tmp/libraries-status-before-research-pass.txt
   git diff --stat > /tmp/libraries-diffstat-before-research-pass.txt
   git diff --name-only > /tmp/libraries-diffnames-before-research-pass.txt
   ```
2. Create the receipt markdown with:
   - date/time
   - exact command list
   - counts from `wc -l` for status/diffnames
   - copied diffstat
   - explicit note: no feature edits happened before this receipt.
3. Commit only if repo policy allows committing current state. If not, leave uncommitted but do not delete.

**Verification:**
```bash
test -s docs/research-implementation/2026-06-26_STARTING_TREE_RECEIPT.md
```
Expected: exit 0.

### Task 0.2: Establish compile/test baseline without fixing

**Objective:** Know whether failures are pre-existing.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/docs/research-implementation/2026-06-26_BASELINE_GATES.md`

**Steps:**
1. Run:
   ```bash
   cd /home/sikmindz/Coding/Libraries
   cargo check --workspace --all-targets 2>&1 | tee /tmp/libraries-cargo-check-baseline.log
   cargo test --workspace --all-targets 2>&1 | tee /tmp/libraries-cargo-test-baseline.log
   ```
2. If either command fails, do not fix in this task. Record first 50 error lines and affected crate names.
3. Add a baseline gate doc with pass/fail table.

**Verification:**
```bash
test -s /tmp/libraries-cargo-check-baseline.log
test -s /tmp/libraries-cargo-test-baseline.log
test -s docs/research-implementation/2026-06-26_BASELINE_GATES.md
```

### Task 0.3: Create research pass control document

**Objective:** Prevent scope creep and public-claim drift.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/docs/research-implementation/RESEARCH_PASS_CONTROL.md`

**Content requirements:**
- In-scope list from this plan.
- Out-of-scope list: HuggingFace and CUDA.
- Public claim boundary:
  - Safe: "implemented an experimental CPU-side prototype"
  - Unsafe until reproduced: paper-reported speedups, quality deltas, memory savings.
- Validation rule: every phase emits a receipt.

**Verification:**
```bash
grep -n "HuggingFace" docs/research-implementation/RESEARCH_PASS_CONTROL.md
grep -n "CUDA" docs/research-implementation/RESEARCH_PASS_CONTROL.md
grep -n "Unsafe until reproduced" docs/research-implementation/RESEARCH_PASS_CONTROL.md
```

---

## Phase 1: RoPE-Aware KV Quantization (Highest Direct ROI)

### Task 1.1: Add RoPE block metadata types

**Objective:** Represent 2D RoPE frequency blocks without changing existing public code structs.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/fib-quant/src/rope.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/lib.rs`
- Test: `/home/sikmindz/Coding/Libraries/fib-quant/src/rope.rs` unit tests

**Implementation shape:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RopeBlock {
    pub block_index: usize,
    pub dim_start: usize,
    pub dim_end: usize,
}

pub fn rope_blocks(head_dim: usize) -> Vec<RopeBlock> {
    (0..head_dim / 2)
        .map(|i| RopeBlock {
            block_index: i,
            dim_start: i * 2,
            dim_end: i * 2 + 2,
        })
        .collect()
}
```

**Tests:**
- `rope_blocks(8)` returns 4 blocks.
- Blocks are contiguous 2D spans.
- Odd `head_dim` either ignores trailing dim with explicit doc comment or returns error; choose one and test it.

**Verification:**
```bash
cargo test -p fib-quant rope_blocks -- --nocapture
```

### Task 1.2: Add label-free RoPE block energy scoring

**Objective:** Compute per-block energy from key vectors so bit allocation can be data-derived but label-free.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/rope.rs`
- Test: same file

**Implementation shape:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RopeBlockEnergy {
    pub block: RopeBlock,
    pub energy: f32,
}

pub fn rope_block_energies(keys: &[Vec<f32>], head_dim: usize) -> Vec<RopeBlockEnergy> {
    let blocks = rope_blocks(head_dim);
    blocks
        .into_iter()
        .map(|block| {
            let mut sum = 0.0f32;
            for key in keys {
                if key.len() >= block.dim_end {
                    let a = key[block.dim_start];
                    let b = key[block.dim_start + 1];
                    sum += a * a + b * b;
                }
            }
            RopeBlockEnergy { block, energy: sum }
        })
        .collect()
}
```

**Tests:**
- Higher-magnitude block receives higher energy.
- Empty key set returns zero energies.
- Short malformed keys are ignored or return error; choose one and document it.

**Verification:**
```bash
cargo test -p fib-quant rope_block_energies -- --nocapture
```

### Task 1.3: Add greedy bit allocator

**Objective:** Allocate integer bits per RoPE block under a total budget.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/rope.rs`

**Implementation shape:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RopeBitAllocation {
    pub bits_per_block: Vec<u8>,
    pub total_bits: usize,
}

pub fn allocate_rope_bits(
    energies: &[RopeBlockEnergy],
    min_bits: u8,
    max_bits: u8,
    total_bits: usize,
) -> RopeBitAllocation {
    let mut bits = vec![min_bits; energies.len()];
    let mut used = bits.iter().map(|b| *b as usize).sum::<usize>();
    while used < total_bits {
        let best = energies
            .iter()
            .enumerate()
            .filter(|(i, _)| bits[*i] < max_bits)
            .max_by(|(_, a), (_, b)| a.energy.total_cmp(&b.energy))
            .map(|(i, _)| i);
        match best {
            Some(i) => {
                bits[i] += 1;
                used += 1;
            }
            None => break,
        }
    }
    RopeBitAllocation { bits_per_block: bits, total_bits: used }
}
```

**Tests:**
- Honors min/max bits.
- Honors total budget when feasible.
- Higher-energy block receives at least as many bits as lower-energy block.

**Verification:**
```bash
cargo test -p fib-quant allocate_rope_bits -- --nocapture
```

### Task 1.4: Wire RoPE allocation into KV policy as optional profile

**Objective:** Add RoPE-aware mode without breaking existing `KvQuantPolicy` users.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/kv/mod.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/kv/codec.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/profile.rs`
- Test: crate unit/integration tests

**Rule:** Additive only. Do not remove or rename existing public fields.

**Implementation shape:**
- Add enum variant or optional config:
  ```rust
  pub enum KvBitAllocationMode {
      Uniform,
      RopeAware,
  }
  ```
- Default remains `Uniform`.
- RoPE-aware mode produces a receipt with per-block bit allocation.

**Verification:**
```bash
cargo test -p fib-quant kv -- --nocapture
cargo check -p fib-quant --all-targets
```

### Task 1.5: Add TurboQuant bridge note, not copy-paste implementation

**Objective:** Keep `turbo-quant` as sidecar codec substrate and avoid duplicating RoPE logic prematurely.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/turbo-quant/README.md`
- Optional Create: `/home/sikmindz/Coding/Libraries/turbo-quant/docs/ROPE_AWARE_KV_NOTES.md`

**Content:**
- State RoPE-aware KV allocation is implemented/prototyped in `fib-quant` first.
- State `turbo-quant` can consume receipts/sidecars later.
- Do not claim Block-GTQ-equivalent performance.

**Verification:**
```bash
grep -n "RoPE-aware" turbo-quant/README.md
```

---

## Phase 2: HyperQuant / Lattice Quantization CPU Prototype

### Task 2.1: Add lattice module behind experimental API

**Objective:** Provide deterministic E8/D4/A2/Z1-style quantizer building blocks without adding HF/model runtime dependencies.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/fib-quant/src/lattice.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/lib.rs`
- Test: lattice unit tests

**Implementation boundary:**
- Start with Z1 and A2 first.
- Leave E8/D4 as TODO only if implementing them risks too much math surface in one pass.
- Do not claim HyperQuant reproduction until benchmarked.

**Tests:**
- `z1_quantize_roundtrip_basic`
- `a2_quantize_preserves_dimension_pairs`
- deterministic output for same input/config

**Verification:**
```bash
cargo test -p fib-quant lattice -- --nocapture
```

### Task 2.2: Add lattice codec profile and receipt

**Objective:** Make the experimental codec measurable and auditable.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/profile.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/receipt.rs`
- Modify: `/home/sikmindz/Coding/Libraries/fib-quant/src/codec.rs`

**Implementation shape:**
```rust
pub enum ExperimentalCodecKind {
    Existing,
    LatticeZ1,
    LatticeA2,
}
```

**Receipt fields:**
- codec kind
- target bits/scalar
- observed bytes
- observed MSE/cosine drift on synthetic fixture
- `claim_status: ExperimentalOnly`

**Verification:**
```bash
cargo test -p fib-quant receipt codec -- --nocapture
cargo check -p fib-quant --all-targets
```

### Task 2.3: Add quant-eval synthetic lattice benchmark

**Objective:** Measure local synthetic behavior without model/HF dependency.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/quant-eval/src/lib.rs`
- Create: `/home/sikmindz/Coding/Libraries/quant-eval/src/lattice.rs`
- Create: `/home/sikmindz/Coding/Libraries/quant-eval/tests/lattice_benchmark.rs`

**Benchmark fixture:**
- deterministic Gaussian vectors from seeded RNG
- dimensions: 64, 128, 256
- metrics: MSE, cosine drift, encoded bytes/scalar

**Verification:**
```bash
cargo test -p quant-eval lattice -- --nocapture
```

---

## Phase 3: Semantic-Memory Retrieval Quality: Hubness, Clusters, Dynamic-Update Receipts

### Task 3.1: Add hubness score type and deterministic calculator

**Objective:** Detect embeddings that behave as universal neighbors.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/semantic-memory/src/hubness.rs`
- Modify: `/home/sikmindz/Coding/Libraries/semantic-memory/src/lib.rs`
- Test: hubness unit tests

**Implementation shape:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HubnessScore {
    pub item_id: String,
    pub neighbor_hits: usize,
    pub normalized_score: f32,
}
```

**Tests:**
- A vector close to many vectors has higher score.
- Identical vector cluster is detected.
- Empty input returns empty scores.

**Verification:**
```bash
cargo test -p semantic-memory hubness -- --nocapture
```

### Task 3.2: Add admission/downweight policy

**Objective:** Allow semantic-memory ingestion/search to quarantine or downweight hubs.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/semantic-memory/src/config.rs` or equivalent config module
- Modify: search/admission path in `/home/sikmindz/Coding/Libraries/semantic-memory/src/lib.rs` or actual ingestion module discovered during implementation
- Tests: relevant semantic-memory tests

**Policy:**
- `Off` default for compatibility.
- `RecordOnly` records hubness receipt but does not affect ranking.
- `Downweight { threshold, factor }` adjusts score.
- `Reject { threshold }` blocks new admission only when explicitly enabled.

**Verification:**
```bash
cargo test -p semantic-memory hubness admission -- --nocapture
cargo check -p semantic-memory --all-targets
```

### Task 3.3: Add namespace/scope centroid clustering prototype

**Objective:** Implement Helmsman-like cluster-first routing using existing namespace/scope boundaries.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/semantic-memory/src/centroids.rs`
- Modify: `/home/sikmindz/Coding/Libraries/semantic-memory/src/search.rs` or actual search module
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/adapters/semantic_memory.rs` if route planning needs scope hints

**Rules:**
- No replacement of current search path.
- Add optional route: centroid prefilter -> local search.
- Emit receipt fields: clusters_considered, clusters_selected, fallback_used.

**Verification:**
```bash
cargo test -p semantic-memory centroid -- --nocapture
cargo test -p knowledge-runtime semantic_memory -- --nocapture
```

### Task 3.4: Add dynamic-update benchmark receipt

**Objective:** Translate ACRONYM insight into measurable update/rebuild costs.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/hnsw-bench/src/lib.rs` or benchmark files discovered in crate
- Create: `/home/sikmindz/Coding/Libraries/hnsw-bench/tests/dynamic_update_receipt.rs`

**Metrics:**
- insert count
- rebuild count
- update latency distribution
- query latency before/after insert burst
- recall fixture metric if exact baseline exists

**Verification:**
```bash
cargo test -p hnsw-bench dynamic_update -- --nocapture
```

---

## Phase 4: Bitemporal Property Graph Query Layer

### Task 4.1: Add bitemporal graph query model

**Objective:** Represent graph edges with valid-time and recorded-time without turning projections into fake facts.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/bitemporal-runtime/src/lib.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/ids.rs` only if shared IDs are needed
- Create tests in bitemporal-runtime

**Implementation shape:**
```rust
pub struct BitemporalGraphEdge<T> {
    pub from: T,
    pub to: T,
    pub relation: String,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}
```

**Verification:**
```bash
cargo test -p bitemporal-runtime graph -- --nocapture
```

### Task 4.2: Add knowledge-runtime temporal graph route

**Objective:** Let query planning distinguish temporal graph traversal from flat semantic search.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/query/classify.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/runtime.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/adapters/semantic_memory.rs`
- Tests: `/home/sikmindz/Coding/Libraries/knowledge-runtime/tests/ugly_case_tests.rs`

**Rules:**
- Scope filters must push down where possible.
- If unsupported, route returns explicit degradation warning.
- No silent widening.

**Verification:**
```bash
cargo test -p knowledge-runtime temporal graph -- --nocapture
```

---

## Phase 5: Contextual Reinstatement and Perspective-Bounded Recall

### Task 5.1: Add perspective key type

**Objective:** Prevent factual overreach/persona drift by separating scope from perspective.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/stack-ids/src/lib.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/ids.rs`
- Tests in both crates

**Implementation shape:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PerspectiveKey(String);
```

**Examples:**
- `operator:josh`
- `agent:hermes-default`
- `project:semantic-memory-maintainer`

**Verification:**
```bash
cargo test -p stack-ids perspective -- --nocapture
cargo test -p knowledge-runtime perspective -- --nocapture
```

### Task 5.2: Add perspective-bound search request fields

**Objective:** Make perspective optional but explicit in query routes.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/runtime.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/query/mod.rs` or actual query request file
- Modify: semantic-memory adapter

**Rules:**
- Default behavior unchanged when no perspective provided.
- When perspective is provided, widening must be recorded.
- If a result crosses perspective boundary, include warning or exclude depending on policy.

**Verification:**
```bash
cargo test -p knowledge-runtime perspective_bound -- --nocapture
```

### Task 5.3: Add contextual reinstatement score

**Objective:** Bias recall toward memories sharing recent episode/context keys, not just embedding similarity.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/semantic-memory/src/reinstatement.rs`
- Modify: semantic-memory search merge/ranking path
- Tests in semantic-memory

**Implementation shape:**
```rust
pub struct ReinstatementContext {
    pub episode_ids: Vec<String>,
    pub entity_ids: Vec<String>,
    pub namespace: Option<String>,
}
```

**Scoring rule:**
- Add small bounded boost for matching episode/entity/context.
- Receipt must include whether boost fired.
- Boost must not override exact contradiction/provenance gates.

**Verification:**
```bash
cargo test -p semantic-memory reinstatement -- --nocapture
```

---

## Phase 6: Structured Graph Query Generation (Text-to-Cypher Without Model Dependency)

### Task 6.1: Add Cypher subset AST

**Objective:** Provide a safe representation for generated graph queries.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/query/cypher_ast.rs`
- Modify: `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/query/mod.rs`
- Tests in knowledge-runtime

**Supported subset:**
- MATCH node-edge-node pattern
- WHERE equality filters
- LIMIT
- RETURN variable list

**Forbidden:**
- write clauses
- DELETE/SET/MERGE/CREATE
- unbounded traversal unless explicitly allowed

**Verification:**
```bash
cargo test -p knowledge-runtime cypher_ast -- --nocapture
```

### Task 6.2: Add llm-output-parser helper for fenced Cypher extraction

**Objective:** Extract a Cypher candidate from LLM text without executing it.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/llm-output-parser/src/cypher.rs`
- Modify: `/home/sikmindz/Coding/Libraries/llm-output-parser/src/lib.rs`
- Tests in llm-output-parser

**Implementation shape:**
```rust
pub fn parse_cypher_block(response: &str) -> Result<String, ParseError> {
    // extract ```cypher ... ``` first, otherwise cleaned text
    // reject CREATE/MERGE/DELETE/SET by keyword scan
}
```

**Verification:**
```bash
cargo test -p llm-output-parser cypher -- --nocapture
```

### Task 6.3: Validate Cypher subset through boundary compiler

**Objective:** Ensure generated graph query text is parsed, bounded, and rejected on unsafe clauses.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/boundary-compiler/src/lib.rs`
- Tests: boundary-compiler unsafe query rejection tests

**Verification:**
```bash
cargo test -p boundary-compiler cypher -- --nocapture
```

---

## Phase 7: RAG Benchmark Harness Without Remote Dataset Dependency

### Task 7.1: Add TREC-style fixture schema

**Objective:** Create a local benchmark schema that can later ingest TREC RAG data, without downloading anything now.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/quant-eval/src/rag.rs`
- Modify: `/home/sikmindz/Coding/Libraries/quant-eval/src/lib.rs`
- Tests: `/home/sikmindz/Coding/Libraries/quant-eval/tests/rag_fixture.rs`

**Schema:**
```rust
pub struct RagQueryFixture {
    pub query_id: String,
    pub query: String,
    pub relevant_doc_ids: Vec<String>,
}

pub struct RagEvalResult {
    pub recall_at_k: f32,
    pub ndcg_at_k: f32,
    pub exact_rerank_recovery: f32,
}
```

**Verification:**
```bash
cargo test -p quant-eval rag_fixture -- --nocapture
```

### Task 7.2: Add semantic-memory adapter benchmark

**Objective:** Evaluate semantic-memory retrieval quality against local fixtures.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/quant-eval/src/rag.rs`
- Add tests with in-memory/mock retrieval results

**Rules:**
- No network.
- No external dataset download.
- No public benchmark claims.

**Verification:**
```bash
cargo test -p quant-eval rag -- --nocapture
```

---

## Phase 8: Formal Verification and Diagnostic Localization Hooks

### Task 8.1: Add diagnostic localization receipt

**Objective:** Capture structured code repair diagnostic locations for SHERLOC-style workflows.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/verification-adjudication/src/lib.rs`
- Modify: `/home/sikmindz/Coding/Libraries/forge-pilot/src/lib.rs` if repair loop owns receipts there
- Tests in affected crates

**Receipt shape:**
```rust
pub struct DiagnosticLocalizationReceiptV1 {
    pub source_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub diagnostic_code: Option<String>,
    pub rationale: String,
    pub confidence: f32,
}
```

**Verification:**
```bash
cargo test -p verification-adjudication diagnostic -- --nocapture
```

### Task 8.2: Add formal-check gate adapter

**Objective:** Let repair/execution workflows declare that a formal or deterministic check was required and whether it passed.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/spec-execution/src/lib.rs`
- Modify: `/home/sikmindz/Coding/Libraries/verification-policy/src/lib.rs`
- Tests in both crates

**Policy:**
- Formal check adapter records command/name/status.
- It does not embed a prover or LLM.
- Missing required check fails closed.

**Verification:**
```bash
cargo test -p spec-execution formal -- --nocapture
cargo test -p verification-policy formal -- --nocapture
```

---

## Phase 9: Docs, Claim Boundaries, and Memory Sync

### Task 9.1: Write research implementation status report

**Objective:** Summarize what was implemented, what was only prototyped, and what remains excluded.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/docs/research-implementation/STATUS_REPORT_2026-06-26.md`

**Required sections:**
- Implemented
- Experimental only
- Not implemented: HuggingFace
- Not implemented: CUDA
- Benchmarks run
- Claims safe to make
- Claims blocked

**Verification:**
```bash
grep -n "Claims blocked" docs/research-implementation/STATUS_REPORT_2026-06-26.md
grep -n "HuggingFace" docs/research-implementation/STATUS_REPORT_2026-06-26.md
grep -n "CUDA" docs/research-implementation/STATUS_REPORT_2026-06-26.md
```

### Task 9.2: Add semantic-memory research facts for implementation outcomes

**Objective:** Save durable, concise implementation findings to semantic memory after real work completes.

**Tool:** `mcp_semantic_memory_sm_add_fact`

**Rules:**
- Namespace: `research` for research integration findings.
- Namespace: `libraries` or `projects` for current implementation state.
- Do not save ephemeral "phase done" status.
- Save only stable facts: API shape, benchmark result, integration path, claim boundary.

**Verification:**
Run `mcp_semantic_memory_sm_search` for each saved fact topic and confirm retrieval.

### Task 9.3: Update READMEs with safe language only

**Objective:** Make crate docs reflect experimental features without overclaiming.

**Files likely to modify:**
- `/home/sikmindz/Coding/Libraries/fib-quant/README.md`
- `/home/sikmindz/Coding/Libraries/semantic-memory/README.md`
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/README.md`
- `/home/sikmindz/Coding/Libraries/quant-eval/README.md`
- `/home/sikmindz/Coding/Libraries/llm-output-parser/README.md`

**Forbidden words unless locally reproduced:**
- production KV runtime
- beats HNSW
- 400x faster
- Block-GTQ equivalent
- fp16-comparable
- zero loss

**Verification:**
```bash
python3 - <<'PY'
from pathlib import Path
forbidden = ["400x", "fp16-comparable", "zero loss", "production KV runtime", "beats HNSW", "Block-GTQ equivalent"]
for p in [Path("fib-quant/README.md"), Path("semantic-memory/README.md"), Path("knowledge-runtime/README.md"), Path("quant-eval/README.md"), Path("llm-output-parser/README.md")]:
    if not p.exists():
        continue
    text = p.read_text(errors="ignore")
    bad = [w for w in forbidden if w.lower() in text.lower()]
    if bad:
        raise SystemExit(f"{p}: forbidden claim language {bad}")
print("claim language check passed")
PY
```

---

## Phase 10: Final Validation Gate

### Task 10.1: Run targeted crate checks

**Objective:** Verify every touched crate independently.

**Commands:**
```bash
cd /home/sikmindz/Coding/Libraries
cargo check -p fib-quant --all-targets
cargo test -p fib-quant --all-targets
cargo check -p semantic-memory --all-targets
cargo test -p semantic-memory --all-targets
cargo check -p knowledge-runtime --all-targets
cargo test -p knowledge-runtime --all-targets
cargo check -p quant-eval --all-targets
cargo test -p quant-eval --all-targets
cargo check -p llm-output-parser --all-targets
cargo test -p llm-output-parser --all-targets
cargo check -p bitemporal-runtime --all-targets
cargo test -p bitemporal-runtime --all-targets
cargo check -p verification-adjudication --all-targets
cargo test -p verification-adjudication --all-targets
```

**Expected:** All pass, or failures are documented as pre-existing from Phase 0 baseline.

### Task 10.2: Run workspace gate if targeted checks pass

**Objective:** Confirm no cross-crate breakage.

**Commands:**
```bash
cd /home/sikmindz/Coding/Libraries
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

**Expected:** Pass, or documented pre-existing failures only.

### Task 10.3: Create final receipt

**Objective:** Produce the closeout artifact.

**Files:**
- Create: `/home/sikmindz/Coding/Libraries/docs/research-implementation/FINAL_RECEIPT_2026-06-26.md`

**Required content:**
- changed files grouped by phase
- tests run with pass/fail
- benchmark receipts generated
- semantic-memory facts saved
- out-of-scope confirmation: HuggingFace and CUDA untouched
- public claim boundary
- rollback notes

**Verification:**
```bash
grep -n "HuggingFace" docs/research-implementation/FINAL_RECEIPT_2026-06-26.md
grep -n "CUDA" docs/research-implementation/FINAL_RECEIPT_2026-06-26.md
grep -n "Tests run" docs/research-implementation/FINAL_RECEIPT_2026-06-26.md
```

---

## Recommended Execution Order

1. Phase 0: mandatory safety/baseline.
2. Phase 1: RoPE-aware KV allocation. Highest direct relevance to your current quantization work.
3. Phase 3 Task 3.1-3.2: hubness detection/downweighting. Highest retrieval-quality ROI.
4. Phase 7: RAG fixture harness. Needed to measure retrieval changes.
5. Phase 2: lattice prototype. Useful but keep experimental.
6. Phase 3 Task 3.3-3.4: clustering/dynamic update receipts.
7. Phase 5: perspective/contextual memory.
8. Phase 4: bitemporal property graph route.
9. Phase 6: Cypher AST/parser.
10. Phase 8: verification/diagnostic hooks.
11. Phase 9-10: docs, memory sync, final gates.

Blunt prioritization:
- Keep: RoPE-aware allocation, hubness policy, RAG harness.
- Prototype carefully: lattice quantization, centroid routing.
- Defer if time-constrained: Text-to-Cypher and formal-check adapter.
- Kill for this pass: HF and CUDA work.

---

## Open Questions for Implementation Agent

1. Does the current dirty `fib-quant` work already contain partial RoPE/residual/KV changes? Check before creating new modules.
2. Does `semantic-memory` have a single search pipeline or multiple MCP/HTTP-specific paths that must both be patched? Prior bug history says check both.
3. Are `stack-ids` and `knowledge-runtime` both using the same scope identity types? Avoid duplicate `PerspectiveKey` definitions if stack-ids owns identity.
4. Are workspace tests currently passing before this pass? If not, preserve Phase 0 baseline and avoid claiming this pass broke them.

---

## Definition of Done

This plan is complete only when:

1. Phase 0 baseline exists.
2. At least the top-three implementation priorities are done:
   - RoPE-aware KV allocation
   - hubness policy
   - RAG fixture harness
3. Every touched crate has targeted `cargo check` and `cargo test` receipts.
4. Final receipt says exactly what was implemented vs only prototyped.
5. Semantic memory contains durable facts for stable research integration outcomes.
6. No HuggingFace or CUDA code paths were added.
7. No paper performance claims appear in README/docs as local claims.
