# proveKV Broader Library Integration Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Wire proveKV/poly-kv into the broader `~/Coding/Libraries` stack as a semantic-memory derived candidate-artifact backend, excluding Recall / Recall-Coding integration.

**Architecture:** `semantic-memory` remains the only authoritative memory substrate: SQLite text/metadata/projections/f32 embeddings are source truth. proveKV/poly-kv is a rebuildable generation-level compressed candidate artifact used only for candidate generation; exact f32 rerank remains mandatory. Downstream crates consume normal semantic-memory APIs plus receipt/trace metadata instead of directly depending on proveKV.

**Tech Stack:** Rust workspace at `/home/sikmindz/Coding/Libraries`; primary crates: `semantic-memory`, `knowledge-runtime`, `forge-memory-bridge`, `llm-tool-runtime`, `agent-graph`, `llm-pipeline`, `kernel-execution`, `kernel-oracles`, `semantic-memory-forge`, `claim-ledger`, AiDENs profile crates; compression crates: `poly-kv/crates/poly-kv`, `fib-quant`, `turbo-quant`, `quant-governor`, `scr-runtime-compression`.

---

## Scope

Implement high-ROI wiring items from the prior list, except item 5 (`Recall / Recall-Coding`).

Included:
1. `semantic-memory`: real proveKV/poly-kv pool generation and candidate backend.
2. `Gloss`: already has a guarded setting; this plan includes only library-facing contracts and optional follow-up verification hooks. Product UI work can stay in the Gloss repo.
3. `knowledge-runtime`: consume proveKV through semantic-memory only; add route-aware hints and trace metadata.
4. `forge-memory-bridge`: import lifecycle observes/triggers semantic-memory pool generation.
6. `llm-tool-runtime`: store/retrieve tool observations through semantic-memory with proveKV-backed candidates.
7. `agent-graph`: graph execution memory receipts and shared-state retrieval via semantic-memory.
8. `llm-pipeline`: provider-call receipts carry retrieved context/proveKV generation provenance; no direct KV-cache claim.
9. `kernel-execution` / `kernel-oracles`: candidate-only retrieval for bounded oracle inputs.
10. `semantic-memory-forge`: audit/explain-only candidate search surfaces.
11. `claim-ledger`: similar-claim/evidence packet discovery, not verification authority.
12. AiDENs profiles: standardized semantic-memory backend profile config, not bespoke proveKV integration.

Excluded:
- Direct Recall / Recall-Coding wiring.
- Direct provider/inference KV-cache reuse unless a local inference runtime explicitly supports it later.
- Any claim that proveKV makes approximate similarity authoritative.
- Any PPL benchmark for this integration; this is retrieval/artifact plumbing, not new codec math.

---

## Non-negotiable invariants

1. `semantic-memory` authoritative f32 embeddings remain stored and queryable.
2. proveKV/poly-kv compressed artifacts are rebuildable derived generations.
3. All derived candidate backends require exact f32 rerank.
4. Downstream crates do not directly depend on proveKV unless they are compression/benchmark crates.
5. All downstream traces distinguish:
   - retrieved candidate
   - exact-reranked final result
   - verified premise / claim truth
6. Scope, temporal, and projection rules stay enforced by the owning crate.
7. Missing/stale pool generation is a receipted fallback, not a silent behavior change.
8. No `Recall` or `Recall-Coding` file path is modified under this plan.

---

## Current known anchor points

Workspace root:
- `/home/sikmindz/Coding/Libraries/Cargo.toml`

semantic-memory:
- `/home/sikmindz/Coding/Libraries/semantic-memory/Cargo.toml`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/config.rs`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/lib.rs`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/types.rs`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/db.rs`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/vector_backend.rs`
- `/home/sikmindz/Coding/Libraries/semantic-memory/src/vector_codec.rs`
- existing policy: `DerivedVectorBackendPolicy::ProveKvPoolCandidateOnly`
- existing rebuild hook: `MemoryStore::rebuild_vector_artifacts()` behind `turbo-quant-codec`

knowledge-runtime:
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/adapters/semantic_memory.rs`
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/runtime/core.rs`
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/query/route.rs`
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/config.rs`
- `/home/sikmindz/Coding/Libraries/knowledge-runtime/src/obs/trace.rs`

bridge / forge:
- `/home/sikmindz/Coding/Libraries/forge-memory-bridge/src`
- `/home/sikmindz/Coding/Libraries/semantic-memory-forge/src`

orchestration / pipeline:
- `/home/sikmindz/Coding/Libraries/llm-tool-runtime/src`
- `/home/sikmindz/Coding/Libraries/agent-graph/src`
- `/home/sikmindz/Coding/Libraries/llm-pipeline/src`

verification / claim:
- `/home/sikmindz/Coding/Libraries/kernel-execution/src`
- `/home/sikmindz/Coding/Libraries/kernel-oracles/src`
- `/home/sikmindz/Coding/Libraries/claim-ledger/src`

AiDENs:
- `/home/sikmindz/Coding/Libraries/AiDENs/crates/*`

---

## Phase 0: Baseline discovery and branch hygiene

### Task 0.1: Create implementation branch

**Objective:** Keep the broad integration isolated.

**Files:** None.

**Command:**
```bash
cd /home/sikmindz/Coding/Libraries
git status --short
git switch -c feat/provekv-derived-candidate-stack
```

**Expected:** Branch created. If working tree is dirty, inspect before switching and do not overwrite unrelated work.

**Commit:** none.

### Task 0.2: Establish no-Recall guard

**Objective:** Ensure the implementation does not touch Recall / Recall-Coding.

**Files:** None initially.

**Command:**
```bash
cd /home/sikmindz/Coding/Libraries
git status --short
```

**Implementation rule:** Throughout the plan, reject any diff touching:
- `/home/sikmindz/Coding/Recall/**`
- `/home/sikmindz/Coding/Recall-Coding/**`
- `/home/sikmindz/Coding/Libraries/**/_vendor/**/Recall*`

**Verification after every phase:**
```bash
git diff --name-only | grep -E '(^|/)Recall(-Coding)?/' && exit 1 || true
```

### Task 0.3: Capture current compile state for target crates

**Objective:** Know whether failures are introduced by this work.

**Commands:**
```bash
cd /home/sikmindz/Coding/Libraries
cargo check -p semantic-memory
cargo check -p knowledge-runtime
cargo check -p forge-memory-bridge
cargo check -p llm-tool-runtime
cargo check -p agent-graph
cargo check -p llm-pipeline
cargo check -p kernel-execution
cargo check -p kernel-oracles
cargo check -p semantic-memory-forge
cargo check -p claim-ledger
```

**Expected:** Ideally all pass. If some fail before changes, record exact failures in `docs/plans/provekv-integration-baseline.md` and avoid claiming this plan introduced them.

---

## Phase 1: semantic-memory real pool-generation substrate

### Task 1.1: Add semantic-memory pool artifact schema types

**Objective:** Add typed receipts and status types for generation-level proveKV/poly-kv pool artifacts.

**Files:**
- Modify: `semantic-memory/src/types.rs`
- Test: `semantic-memory/tests/pool_generation_types.rs`

**Add types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProveKvPoolGenerationV1 {
    pub schema_version: String,
    pub generation_id: String,
    pub embedding_snapshot_digest: String,
    pub source_digest: String,
    pub pool_manifest_digest: String,
    pub codec_family: String,
    pub codec_profile: String,
    pub vector_dim: usize,
    pub item_count: usize,
    pub payload_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProveKvPoolItemMapEntryV1 {
    pub generation_id: String,
    pub item_id: String,
    pub source_type: String,
    pub pool_index: usize,
    pub embedding_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProveKvPoolGenerationStatus {
    Disabled,
    Missing,
    Building,
    Ready,
    Stale,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProveKvPoolArtifactStatusV1 {
    pub status: ProveKvPoolGenerationStatus,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub pool_manifest_digest: Option<String>,
    pub item_count: usize,
    pub payload_bytes: u64,
    pub reason: Option<String>,
}
```

**Test:** serialize/deserialize roundtrip and `schema_version` equality.

**Command:**
```bash
cargo test -p semantic-memory --test pool_generation_types
```

**Commit:**
```bash
git add semantic-memory/src/types.rs semantic-memory/tests/pool_generation_types.rs
git commit -m "feat(semantic-memory): add provekv pool generation types"
```

### Task 1.2: Add SQLite tables for pool generations and item maps

**Objective:** Persist generation metadata and item→pool-index mapping in semantic-memory.

**Files:**
- Modify: `semantic-memory/src/db.rs`
- Possibly modify migration helpers in `semantic-memory/src/lib.rs` or current migrations module if split out.
- Test: `semantic-memory/tests/pool_generation_db.rs`

**Schema intent:**
```sql
CREATE TABLE IF NOT EXISTS provekv_pool_generations (
  generation_id TEXT PRIMARY KEY,
  embedding_snapshot_digest TEXT NOT NULL,
  source_digest TEXT NOT NULL,
  pool_manifest_digest TEXT NOT NULL,
  codec_family TEXT NOT NULL,
  codec_profile TEXT NOT NULL,
  vector_dim INTEGER NOT NULL,
  item_count INTEGER NOT NULL,
  payload_bytes INTEGER NOT NULL,
  payload BLOB NOT NULL,
  status TEXT NOT NULL,
  failure_reason TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provekv_pool_item_map (
  generation_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  source_type TEXT NOT NULL,
  pool_index INTEGER NOT NULL,
  embedding_digest TEXT NOT NULL,
  PRIMARY KEY (generation_id, item_id),
  FOREIGN KEY (generation_id) REFERENCES provekv_pool_generations(generation_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_provekv_pool_item_map_generation_index
  ON provekv_pool_item_map(generation_id, pool_index);
```

**Functions to add:**
- `insert_provekv_pool_generation(conn, generation, payload, item_map)`
- `latest_ready_provekv_pool_generation(conn) -> Option<...>`
- `load_provekv_pool_payload(conn, generation_id) -> Vec<u8>`
- `load_provekv_pool_item_map(conn, generation_id) -> Vec<...>`
- `mark_provekv_pool_generation_failed(conn, generation_id, reason)`
- `provekv_pool_artifact_status(conn) -> ProveKvPoolArtifactStatusV1`

**Test:**
- insert generation + two map rows
- latest ready returns it
- deleting generation removes map rows
- failed generation is visible with status `Failed`

**Command:**
```bash
cargo test -p semantic-memory --test pool_generation_db
```

**Commit:**
```bash
git add semantic-memory/src/db.rs semantic-memory/tests/pool_generation_db.rs
git commit -m "feat(semantic-memory): persist provekv pool generations"
```

### Task 1.3: Add deterministic embedding snapshot digest builder

**Objective:** Build stable snapshot/source digests over authoritative f32 embeddings.

**Files:**
- Modify: `semantic-memory/src/db.rs` or create `semantic-memory/src/vector_snapshot.rs`
- Modify: `semantic-memory/src/lib.rs` module exports if new file.
- Test: `semantic-memory/tests/vector_snapshot_digest.rs`

**Design:**
- Snapshot rows sorted by stable item identity: source type + id.
- Digest includes: schema version, embedding dim, item id, source type, embedding bytes, recorded update timestamp if available.
- Use SHA-256. If no existing hash dependency, add `sha2` via workspace dependency only after checking existing workspace deps.

**API:**
```rust
pub struct EmbeddingSnapshotRow {
    pub item_id: String,
    pub source_type: String,
    pub embedding: Vec<f32>,
}

pub struct EmbeddingSnapshotV1 {
    pub embedding_snapshot_digest: String,
    pub source_digest: String,
    pub vector_dim: usize,
    pub rows: Vec<EmbeddingSnapshotRow>,
}
```

**Test:**
- same rows in different order produce same digest
- changed embedding changes digest
- changed item id changes digest

**Command:**
```bash
cargo test -p semantic-memory --test vector_snapshot_digest
```

**Commit:**
```bash
git add semantic-memory/src semantic-memory/tests/vector_snapshot_digest.rs semantic-memory/Cargo.toml Cargo.toml
git commit -m "feat(semantic-memory): add deterministic embedding snapshots"
```

### Task 1.4: Add poly-kv/proveKV pool builder adapter

**Objective:** Convert an embedding snapshot into a generation-level pool payload and item map.

**Files:**
- Create: `semantic-memory/src/provekv_pool.rs`
- Modify: `semantic-memory/src/lib.rs`
- Modify: `semantic-memory/Cargo.toml`
- Test: `semantic-memory/tests/provekv_pool_adapter.rs`

**Feature gates:**
- Existing config references `poly-kv-pool`; ensure `semantic-memory/Cargo.toml` has:
```toml
poly-kv-pool = ["dep:poly-kv-core", "poly-kv-core/fib"]
poly-kv-core = { version = "0.1.0-alpha.1", package = "poly-kv", path = "../poly-kv/crates/poly-kv", default-features = false, optional = true }
```
Adjust path based on actual current Cargo.toml; do not point at excluded workspace root `../poly-kv` if the member crate is `poly-kv/crates/poly-kv`.

**API:**
```rust
#[cfg(feature = "poly-kv-pool")]
pub fn build_provekv_pool_generation(
    snapshot: EmbeddingSnapshotV1,
    seed: u64,
) -> Result<(ProveKvPoolGenerationV1, Vec<u8>, Vec<ProveKvPoolItemMapEntryV1>), MemoryError>
```

**Rules:**
- Do not call per-vector `VectorCodec::encode(vector)` for this integration.
- Use generation-level/batched pool mechanics available in poly-kv/proveKV.
- If the poly-kv API cannot yet encode arbitrary embedding vectors directly, implement a minimal internal adapter layer in semantic-memory that clearly emits `fallback = generation_not_materialized` until the API is present. Do not fake compressed payloads.

**Test:**
- with feature enabled, small deterministic snapshot creates non-empty payload and complete item map
- generation fields are non-empty
- every map entry pool_index matches row order

**Command:**
```bash
cargo test -p semantic-memory --features poly-kv-pool --test provekv_pool_adapter
```

**Commit:**
```bash
git add semantic-memory/src/provekv_pool.rs semantic-memory/src/lib.rs semantic-memory/Cargo.toml semantic-memory/tests/provekv_pool_adapter.rs Cargo.toml
git commit -m "feat(semantic-memory): build provekv pool artifacts from embedding snapshots"
```

### Task 1.5: Add `MemoryStore::rebuild_provekv_pool_artifacts()`

**Objective:** Expose a rebuild API analogous to existing vector artifact rebuilds.

**Files:**
- Modify: `semantic-memory/src/lib.rs`
- Test: `semantic-memory/tests/provekv_pool_rebuild.rs`

**API:**
```rust
#[cfg(feature = "poly-kv-pool")]
pub async fn rebuild_provekv_pool_artifacts(
    &self,
) -> Result<ProveKvPoolArtifactBuildReceiptV1, MemoryError>
```

**Receipt fields:**
```rust
pub struct ProveKvPoolArtifactBuildReceiptV1 {
    pub schema_version: String,
    pub generation_id: String,
    pub embedding_snapshot_digest: String,
    pub source_digest: String,
    pub pool_manifest_digest: String,
    pub codec_family: String,
    pub codec_profile: String,
    pub vector_dim: usize,
    pub item_count: usize,
    pub payload_bytes: u64,
    pub exact_rerank_required: bool,
    pub created_at: DateTime<Utc>,
}
```

**Validation:**
- reject empty digests
- reject `item_count == 0` unless store has no embeddings and status explicitly says `Missing`
- reject mismatch between map length and item_count

**Command:**
```bash
cargo test -p semantic-memory --features poly-kv-pool --test provekv_pool_rebuild
```

**Commit:**
```bash
git add semantic-memory/src/lib.rs semantic-memory/src/types.rs semantic-memory/tests/provekv_pool_rebuild.rs
git commit -m "feat(semantic-memory): expose provekv pool rebuild API"
```

### Task 1.6: Add proveKV pool candidate search path

**Objective:** When configured, use latest ready proveKV pool generation to generate vector candidates, then exact-rerank against f32 embeddings.

**Files:**
- Modify: `semantic-memory/src/lib.rs`
- Modify: `semantic-memory/src/vector_backend.rs` or add `semantic-memory/src/provekv_candidate_backend.rs`
- Test: `semantic-memory/tests/provekv_pool_candidate_search.rs`

**Behavior:**
- If `SearchConfig.derived_vector_backend == ProveKvPoolCandidateOnly`:
  - require `turbo_quant_require_exact_rerank == true` already enforced by config
  - load latest ready pool generation
  - produce approximate candidate ids
  - load authoritative f32 embeddings for those ids
  - exact cosine rerank
  - feed results into existing hybrid/BM25/RRF pipeline
- If no generation exists:
  - search falls back to authoritative f32/HNSW/exact backend
  - receipt reports `fallback = provekv_pool_generation_not_materialized`
  - `approximate = false`
  - `exact_rerank = true`

**Test cases:**
1. configured backend with no generation returns results and receipt fallback.
2. configured backend with ready generation returns same top-k as exact f32 on tiny fixture.
3. disabling exact rerank fails config validation.
4. source/namespace filters are applied after candidate generation and before final return.

**Command:**
```bash
cargo test -p semantic-memory --features poly-kv-pool --test provekv_pool_candidate_search
```

**Commit:**
```bash
git add semantic-memory/src semantic-memory/tests/provekv_pool_candidate_search.rs
git commit -m "feat(semantic-memory): use provekv pool as exact-reranked candidate backend"
```

### Task 1.7: Extend semantic-memory search receipts

**Objective:** Make proveKV backend behavior visible to downstream crates.

**Files:**
- Modify: `semantic-memory/src/types.rs`
- Modify: search/explain builders in `semantic-memory/src/lib.rs`
- Test: `semantic-memory/tests/provekv_pool_receipts.rs`

**Receipt fields to add to explained/search metadata:**
```rust
pub struct DerivedCandidateReceiptV1 {
    pub candidate_backend: String,
    pub codec_family: Option<String>,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub pool_manifest_digest: Option<String>,
    pub exact_rerank: bool,
    pub approximate: bool,
    pub fallback: Option<String>,
    pub raw_candidate_count: usize,
    pub post_filter_count: usize,
    pub final_result_count: usize,
}
```

**Required values for no-generation fallback:**
- `candidate_backend = "provekv_pool_candidate_then_exact_f32"`
- `codec_family = Some("provekv_pool")`
- `fallback = Some("provekv_pool_generation_not_materialized")`
- `exact_rerank = true`
- `approximate = false`

**Command:**
```bash
cargo test -p semantic-memory --features poly-kv-pool --test provekv_pool_receipts
```

**Commit:**
```bash
git add semantic-memory/src/types.rs semantic-memory/src/lib.rs semantic-memory/tests/provekv_pool_receipts.rs
git commit -m "feat(semantic-memory): receipt provekv candidate backend behavior"
```

### Phase 1 gate

**Commands:**
```bash
cd /home/sikmindz/Coding/Libraries
cargo test -p semantic-memory
cargo test -p semantic-memory --features poly-kv-pool
cargo check -p semantic-memory --features poly-kv-pool
```

**Expected:** pass, or only documented pre-existing failures.

---

## Phase 2: benchmark and receipt harness for retrieval scale

### Task 2.1: Add semantic-memory backend parity benchmark example

**Objective:** Measure exact/HNSW vs TurboQuant vs proveKV pool candidate behavior without making PPL claims.

**Files:**
- Create: `semantic-memory/examples/derived_candidate_benchmark.rs`
- Create: `semantic-memory/tests/derived_candidate_benchmark_smoke.rs`

**Benchmark emits JSON:**
```json
{
  "schema_version": "semantic_memory_candidate_bench_v1",
  "corpus_size": 1000,
  "query_count": 50,
  "embedding_dim": 384,
  "backends": {
    "exact_f32": {"p50_ms": 0.0, "p95_ms": 0.0},
    "turbo_quant_candidate_then_exact_f32": {"p50_ms": 0.0, "p95_ms": 0.0, "recall_at_10": 1.0},
    "provekv_pool_candidate_then_exact_f32": {"p50_ms": 0.0, "p95_ms": 0.0, "recall_at_10": 1.0}
  },
  "artifact_bytes": {
    "provekv_pool": 0,
    "turbo_quant": 0,
    "f32_embeddings": 0
  }
}
```

**Command:**
```bash
cargo run -p semantic-memory --features poly-kv-pool,turbo-quant-codec --example derived_candidate_benchmark -- --corpus-size 1000 --queries 50
```

**Commit:**
```bash
git add semantic-memory/examples/derived_candidate_benchmark.rs semantic-memory/tests/derived_candidate_benchmark_smoke.rs
git commit -m "bench(semantic-memory): add derived candidate backend benchmark"
```

### Task 2.2: Add machine-readable benchmark validation script

**Objective:** Prevent benchmarks from claiming scale wins without exact-rerank parity.

**Files:**
- Create: `semantic-memory/scripts/validate_candidate_bench.py`

**Validation rules:**
- JSON parses.
- proveKV backend has `recall_at_10 >= 0.99` on deterministic fixture.
- proveKV `exact_rerank == true` if included.
- artifact byte counts are non-zero when generation is materialized.
- no field says PPL.

**Command:**
```bash
python semantic-memory/scripts/validate_candidate_bench.py /tmp/semantic_memory_candidate_bench.json
```

**Commit:**
```bash
git add semantic-memory/scripts/validate_candidate_bench.py
git commit -m "test(semantic-memory): validate derived candidate benchmark receipts"
```

### Phase 2 gate

```bash
cargo run -p semantic-memory --features poly-kv-pool,turbo-quant-codec --example derived_candidate_benchmark -- --corpus-size 100 --queries 5 > /tmp/sm_candidate_bench.json
python semantic-memory/scripts/validate_candidate_bench.py /tmp/sm_candidate_bench.json
```

---

## Phase 3: Knowledge Runtime route-aware integration

### Task 3.1: Add runtime config for derived candidate backend hint

**Objective:** Let KR request semantic-memory default/proveKV behavior without depending on proveKV.

**Files:**
- Modify: `knowledge-runtime/src/config.rs`
- Test: `knowledge-runtime/tests/provekv_backend_config.rs`

**Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCandidateBackendHint {
    SemanticMemoryDefault,
    ProveKvPoolCandidate,
}

impl Default for RuntimeCandidateBackendHint {
    fn default() -> Self { Self::SemanticMemoryDefault }
}
```

Add to `QueryConfig`:
```rust
#[serde(default)]
pub candidate_backend_hint: RuntimeCandidateBackendHint,
```

**Rules:**
- This is a hint/trace setting only.
- KR does not construct semantic-memory `MemoryConfig` here unless an existing constructor path does so.

**Command:**
```bash
cargo test -p knowledge-runtime --test provekv_backend_config
```

**Commit:**
```bash
git add knowledge-runtime/src/config.rs knowledge-runtime/tests/provekv_backend_config.rs
git commit -m "feat(knowledge-runtime): add candidate backend hint config"
```

### Task 3.2: Extend QueryTrace with derived candidate receipt summary

**Objective:** Propagate semantic-memory backend receipts into KR trace.

**Files:**
- Modify: `knowledge-runtime/src/obs/trace.rs`
- Modify: `knowledge-runtime/src/adapters/semantic_memory.rs`
- Test: `knowledge-runtime/tests/provekv_trace_receipts.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDerivedCandidateTraceV1 {
    pub candidate_backend: String,
    pub codec_family: Option<String>,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub pool_manifest_digest: Option<String>,
    pub exact_rerank: bool,
    pub approximate: bool,
    pub fallback: Option<String>,
    pub raw_candidate_count: usize,
    pub post_filter_count: usize,
    pub final_result_count: usize,
}
```

Add to `QueryTrace`:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub derived_candidate_receipts: Vec<RuntimeDerivedCandidateTraceV1>,
```

**Adapter:**
- Extend `ExplainedSearchArtifacts` to carry `derived_candidate_receipt: Option<...>` if semantic-memory exposes it.
- If current semantic-memory `ExplainedResult` embeds metadata per-result, consolidate into one leg receipt.

**Command:**
```bash
cargo test -p knowledge-runtime --test provekv_trace_receipts
```

**Commit:**
```bash
git add knowledge-runtime/src/obs/trace.rs knowledge-runtime/src/adapters/semantic_memory.rs knowledge-runtime/tests/provekv_trace_receipts.rs
git commit -m "feat(knowledge-runtime): trace derived candidate backend receipts"
```

### Task 3.3: Add route policy for when proveKV candidates are allowed

**Objective:** Encode KR’s safe usage rules per route.

**Files:**
- Modify: `knowledge-runtime/src/query/route.rs`
- Modify: `knowledge-runtime/src/runtime/core.rs`
- Test: `knowledge-runtime/tests/provekv_route_policy.rs`

**Policy rules:**
- `HybridSearch`: allow proveKV candidate hint.
- `EntitySearch`: allow only as bounded semantic fallback after exact/alias candidate path.
- `TemporalSearch`: allow only when temporal route degrades or when semantic-memory receipt confirms temporally scoped generation; otherwise warn.
- Evidence/audit methods: no default ranking use.

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateBackendUse {
    Allowed,
    BoundedFallbackOnly,
    DisabledForRoute,
}
```

**Test cases:**
- semantic query gets `Allowed`
- entity query gets `BoundedFallbackOnly`
- temporal query without explicit supported as-of gets warning if fallback occurs
- strict temporal + unsupported proveKV fallback returns existing temporal error

**Command:**
```bash
cargo test -p knowledge-runtime --test provekv_route_policy
```

**Commit:**
```bash
git add knowledge-runtime/src/query/route.rs knowledge-runtime/src/runtime/core.rs knowledge-runtime/tests/provekv_route_policy.rs
git commit -m "feat(knowledge-runtime): add route-aware provekv candidate policy"
```

### Task 3.4: Add KR integration test against semantic-memory no-generation fallback

**Objective:** Prove KR can query with semantic-memory configured for proveKV candidate backend even before a generation is built.

**Files:**
- Modify/Create: `knowledge-runtime/tests/cross_crate_proof.rs`

**Test:**
- create temp semantic-memory store with `DerivedVectorBackendPolicy::ProveKvPoolCandidateOnly`
- do not rebuild pool
- run KR semantic query
- assert results are returned through fallback
- assert KR trace contains fallback `provekv_pool_generation_not_materialized`
- assert `exact_rerank == true`

**Command:**
```bash
cargo test -p knowledge-runtime --features semantic-memory/poly-kv-pool --test cross_crate_proof provekv
```

**Commit:**
```bash
git add knowledge-runtime/tests/cross_crate_proof.rs
git commit -m "test(knowledge-runtime): prove semantic-memory provekv fallback trace"
```

### Phase 3 gate

```bash
cargo test -p knowledge-runtime
cargo test -p knowledge-runtime --features semantic-memory/poly-kv-pool
```

---

## Phase 4: forge-memory-bridge import lifecycle integration

### Task 4.1: Add bridge projection artifact lifecycle type

**Objective:** Bridge import receipts can report semantic-memory vector/pool generation status.

**Files:**
- Inspect and modify receipt types under `forge-memory-bridge/src`
- Test: `forge-memory-bridge/tests/provekv_lifecycle_receipts.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeDerivedArtifactStatusV1 {
    pub artifact_family: String,
    pub requested: bool,
    pub status: String,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub manifest_digest: Option<String>,
    pub reason: Option<String>,
}
```

**Receipt field:**
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub derived_artifacts: Vec<BridgeDerivedArtifactStatusV1>,
```

**Command:**
```bash
cargo test -p forge-memory-bridge --test provekv_lifecycle_receipts
```

**Commit:**
```bash
git add forge-memory-bridge/src forge-memory-bridge/tests/provekv_lifecycle_receipts.rs
git commit -m "feat(forge-memory-bridge): receipt semantic-memory derived artifacts"
```

### Task 4.2: Add post-import artifact rebuild request hook

**Objective:** After a projection import completes, optionally trigger semantic-memory proveKV pool rebuild or mark it pending.

**Files:**
- Modify relevant import executor in `forge-memory-bridge/src`
- Test: `forge-memory-bridge/tests/provekv_post_import_hook.rs`

**Design:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeImportOptions {
    #[serde(default)]
    pub rebuild_semantic_vector_artifacts: bool,
    #[serde(default)]
    pub rebuild_provekv_pool_artifacts: bool,
}
```

**Rules:**
- If bridge owns a `MemoryStore` handle and feature is enabled, call `store.rebuild_provekv_pool_artifacts().await`.
- If not feature-enabled, receipt `status = disabled`.
- If caller wants async orchestration, receipt `status = requested` and external orchestrator does the rebuild.
- Do not query raw Forge receipts during normal retrieval.

**Command:**
```bash
cargo test -p forge-memory-bridge --features semantic-memory/poly-kv-pool --test provekv_post_import_hook
```

**Commit:**
```bash
git add forge-memory-bridge/src forge-memory-bridge/tests/provekv_post_import_hook.rs
git commit -m "feat(forge-memory-bridge): request provekv pool rebuild after imports"
```

### Phase 4 gate

```bash
cargo test -p forge-memory-bridge
cargo test -p forge-memory-bridge --features semantic-memory/poly-kv-pool
```

---

## Phase 5 intentionally skipped

Recall / Recall-Coding integration is intentionally excluded.

Verification:
```bash
git diff --name-only | grep -E '(^|/)Recall(-Coding)?/' && exit 1 || true
```

---

## Phase 6: llm-tool-runtime memory surface

### Task 6.1: Add tool observation memory record type

**Objective:** Tool runtime can write/search tool observations through semantic-memory without direct proveKV dependency.

**Files:**
- Modify: `llm-tool-runtime/src`
- Test: `llm-tool-runtime/tests/tool_observation_memory.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolObservationMemoryRecordV1 {
    pub tool_name: String,
    pub invocation_id: String,
    pub session_id: Option<String>,
    pub scope: String,
    pub summary: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub receipt_id: Option<String>,
}
```

**Rules:**
- Use semantic-memory APIs for storage/retrieval if existing dependency exists; if not, define a trait interface to avoid hard dependency sprawl.
- Do not use proveKV for authorization, tool permission, or capability decisions.

**Command:**
```bash
cargo test -p llm-tool-runtime --test tool_observation_memory
```

**Commit:**
```bash
git add llm-tool-runtime/src llm-tool-runtime/tests/tool_observation_memory.rs
git commit -m "feat(llm-tool-runtime): define searchable tool observation memory records"
```

### Task 6.2: Add similar prior tool run query API

**Objective:** Retrieve similar errors/commands/tool runs through semantic-memory, benefiting from proveKV backend when configured.

**Files:**
- Modify: `llm-tool-runtime/src`
- Test: `llm-tool-runtime/tests/similar_tool_run_search.rs`

**API:**
```rust
pub async fn find_similar_tool_observations(
    &self,
    query: &str,
    scope: &str,
    limit: usize,
) -> Result<Vec<ToolObservationMemoryRecordV1>, ToolRuntimeError>
```

**Trace fields:** include semantic-memory derived candidate receipt if available.

**Command:**
```bash
cargo test -p llm-tool-runtime --test similar_tool_run_search
```

**Commit:**
```bash
git add llm-tool-runtime/src llm-tool-runtime/tests/similar_tool_run_search.rs
git commit -m "feat(llm-tool-runtime): search similar prior tool observations"
```

### Phase 6 gate

```bash
cargo test -p llm-tool-runtime
```

---

## Phase 7: agent-graph execution memory and receipts

### Task 7.1: Add memory generation provenance to graph execution receipts

**Objective:** Agent graph executions can state which semantic-memory/proveKV generation influenced retrieved context.

**Files:**
- Modify receipt types under `agent-graph/src`
- Test: `agent-graph/tests/memory_generation_receipts.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMemoryGenerationRefV1 {
    pub memory_backend: String,
    pub candidate_backend: Option<String>,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub manifest_digest: Option<String>,
    pub exact_rerank: bool,
    pub fallback: Option<String>,
}
```

**Command:**
```bash
cargo test -p agent-graph --test memory_generation_receipts
```

**Commit:**
```bash
git add agent-graph/src agent-graph/tests/memory_generation_receipts.rs
git commit -m "feat(agent-graph): receipt memory generation provenance"
```

### Task 7.2: Add shared graph-state retrieval trait

**Objective:** Agent graph can retrieve relevant graph state/history through semantic-memory, not direct proveKV.

**Files:**
- Modify: `agent-graph/src`
- Test: `agent-graph/tests/shared_state_retrieval.rs`

**Trait:**
```rust
#[async_trait::async_trait]
pub trait GraphMemoryRetriever: Send + Sync {
    async fn retrieve_graph_context(
        &self,
        graph_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<GraphMemoryRetrievalV1, AgentGraphError>;
}
```

**Rules:**
- Trait result includes `Vec<GraphMemoryGenerationRefV1>`.
- No direct proveKV dependency.
- Multi-agent shared prefix KV compression is future work, not claimed here.

**Command:**
```bash
cargo test -p agent-graph --test shared_state_retrieval
```

**Commit:**
```bash
git add agent-graph/src agent-graph/tests/shared_state_retrieval.rs
git commit -m "feat(agent-graph): add graph memory retrieval interface"
```

### Phase 7 gate

```bash
cargo test -p agent-graph
```

---

## Phase 8: llm-pipeline context provenance

### Task 8.1: Add retrieved context provenance to ProviderCallReceiptV1

**Objective:** Provider calls record which semantic-memory/proveKV candidate generation produced prompt context.

**Files:**
- Modify receipt types under `llm-pipeline/src`
- Test: `llm-pipeline/tests/provider_context_provenance.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievedContextProvenanceV1 {
    pub retrieval_system: String,
    pub candidate_backend: Option<String>,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub manifest_digest: Option<String>,
    pub exact_rerank: bool,
    pub result_count: usize,
    pub fallback: Option<String>,
}
```

Add to provider call receipt:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub retrieved_context: Vec<RetrievedContextProvenanceV1>,
```

**Do not claim:** provider KV-cache compression, reduced framework cache bytes, or PPL impact.

**Command:**
```bash
cargo test -p llm-pipeline --test provider_context_provenance
```

**Commit:**
```bash
git add llm-pipeline/src llm-pipeline/tests/provider_context_provenance.rs
git commit -m "feat(llm-pipeline): receipt retrieved context provenance"
```

### Task 8.2: Thread context provenance through prompt assembly path

**Objective:** When a caller supplies retrieved context metadata, provider receipts preserve it.

**Files:**
- Modify prompt/call builders in `llm-pipeline/src`
- Test: `llm-pipeline/tests/context_provenance_threading.rs`

**Command:**
```bash
cargo test -p llm-pipeline --test context_provenance_threading
```

**Commit:**
```bash
git add llm-pipeline/src llm-pipeline/tests/context_provenance_threading.rs
git commit -m "feat(llm-pipeline): thread retrieved context provenance into calls"
```

### Phase 8 gate

```bash
cargo test -p llm-pipeline
```

---

## Phase 9: kernel-execution / kernel-oracles candidate-only retrieval

### Task 9.1: Define candidate evidence retrieval input contract

**Objective:** Kernel systems can request bounded candidate inputs from semantic-memory while preserving proof boundaries.

**Files:**
- Modify: `kernel-execution/src`
- Modify if needed: `kernel-oracles/src`
- Test: `kernel-execution/tests/candidate_retrieval_contract.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateEvidenceRetrievalRefV1 {
    pub retrieval_system: String,
    pub candidate_backend: Option<String>,
    pub generation_id: Option<String>,
    pub evidence_ref: String,
    pub candidate_only: bool,
    pub exact_rerank: bool,
    pub verified_by_oracle: bool,
}
```

**Rules:**
- `candidate_only` must be true before oracle evaluation.
- `verified_by_oracle` false until oracle explicitly evaluates.
- Similarity score cannot satisfy a constraint.

**Command:**
```bash
cargo test -p kernel-execution --test candidate_retrieval_contract
```

**Commit:**
```bash
git add kernel-execution/src kernel-execution/tests/candidate_retrieval_contract.rs
git commit -m "feat(kernel-execution): add candidate evidence retrieval contract"
```

### Task 9.2: Make oracle reports distinguish retrieved vs verified premises

**Objective:** Prevent proveKV candidate retrieval from being interpreted as verification.

**Files:**
- Modify: `kernel-oracles/src`
- Test: `kernel-oracles/tests/retrieved_vs_verified.rs`

**Report fields:**
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub retrieved_candidates: Vec<CandidateEvidenceRetrievalRefV1>,
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub verified_premises: Vec<String>,
```

**Test:**
- report with retrieved candidate but no oracle evaluation is not accepted as verified
- report after oracle evaluation moves/links candidate into verified premise list

**Command:**
```bash
cargo test -p kernel-oracles --test retrieved_vs_verified
```

**Commit:**
```bash
git add kernel-oracles/src kernel-oracles/tests/retrieved_vs_verified.rs
git commit -m "feat(kernel-oracles): distinguish retrieved candidates from verified premises"
```

### Phase 9 gate

```bash
cargo test -p kernel-execution
cargo test -p kernel-oracles
```

---

## Phase 10: semantic-memory-forge audit/explain-only search

### Task 10.1: Add audit candidate search request/response types

**Objective:** Forge can expose candidate evidence search for audit/explain workflows only.

**Files:**
- Modify: `semantic-memory-forge/src`
- Test: `semantic-memory-forge/tests/audit_candidate_search.rs`

**Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForgeAuditCandidateSearchRequestV1 {
    pub query: String,
    pub scope: String,
    pub limit: usize,
    pub explain_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForgeAuditCandidateSearchResultV1 {
    pub evidence_ref: String,
    pub summary: String,
    pub retrieval_backend: String,
    pub candidate_backend: Option<String>,
    pub candidate_only: bool,
}
```

**Rule:** reject request unless `explain_only == true`.

**Command:**
```bash
cargo test -p semantic-memory-forge --test audit_candidate_search
```

**Commit:**
```bash
git add semantic-memory-forge/src semantic-memory-forge/tests/audit_candidate_search.rs
git commit -m "feat(semantic-memory-forge): add explain-only audit candidate search types"
```

### Task 10.2: Add boundary docs between Forge, semantic-memory, and proveKV

**Objective:** Prevent future agents from collapsing source truth and compressed candidate artifacts.

**Files:**
- Create: `semantic-memory-forge/docs/provekv-boundary.md`
- Possibly update: `semantic-memory-forge/README.md`

**Required text:**
- Forge owns raw evidence/export/fixity.
- semantic-memory owns projected query substrate.
- proveKV/poly-kv owns rebuildable compressed candidate generations through semantic-memory.
- Audit candidate search is not normal ranking.

**Commit:**
```bash
git add semantic-memory-forge/docs/provekv-boundary.md semantic-memory-forge/README.md
git commit -m "docs(semantic-memory-forge): document provekv boundary"
```

### Phase 10 gate

```bash
cargo test -p semantic-memory-forge
```

---

## Phase 11: claim-ledger candidate discovery

### Task 11.1: Add similar-claim candidate types

**Objective:** ClaimLedger can discover similar claims/evidence packets via semantic-memory without changing claim status.

**Files:**
- Modify: `claim-ledger/src`
- Test: `claim-ledger/tests/similar_claim_candidates.rs`

**Type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimilarClaimCandidateV1 {
    pub claim_id: String,
    pub claim_version_id: Option<String>,
    pub retrieval_backend: String,
    pub candidate_backend: Option<String>,
    pub generation_id: Option<String>,
    pub exact_rerank: bool,
    pub candidate_only: bool,
}
```

**Rule:** `SimilarClaimCandidateV1` cannot mutate verification state or promotion state.

**Command:**
```bash
cargo test -p claim-ledger --test similar_claim_candidates
```

**Commit:**
```bash
git add claim-ledger/src claim-ledger/tests/similar_claim_candidates.rs
git commit -m "feat(claim-ledger): add similar claim candidate discovery types"
```

### Task 11.2: Add proof-packet assembly provenance hook

**Objective:** When candidate search helps assemble a proof packet, the packet records candidate provenance separately from verified evidence.

**Files:**
- Modify: `claim-ledger/src`
- Test: `claim-ledger/tests/proof_packet_candidate_provenance.rs`

**Fields:**
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub candidate_discovery: Vec<SimilarClaimCandidateV1>,
```

**Test:** proof packet with candidate discovery but no verified evidence does not pass verification gates.

**Command:**
```bash
cargo test -p claim-ledger --test proof_packet_candidate_provenance
```

**Commit:**
```bash
git add claim-ledger/src claim-ledger/tests/proof_packet_candidate_provenance.rs
git commit -m "feat(claim-ledger): track proof packet candidate provenance"
```

### Phase 11 gate

```bash
cargo test -p claim-ledger
```

---

## Phase 12: AiDENs profile-level standardization

### Task 12.1: Add shared memory backend profile spec doc

**Objective:** AiDENs uses semantic-memory profile choices instead of bespoke proveKV integration.

**Files:**
- Create: `AiDENs/docs/semantic-memory-backend-profiles.md`

**Profiles:**
```text
small:
  derived_vector_backend = disabled
  use for tiny local memories / tests

medium:
  derived_vector_backend = turbo_quant_candidate_only
  exact_rerank = true
  use for moderate corpora where per-vector artifacts are enough

large:
  derived_vector_backend = provekv_pool_candidate_only
  exact_rerank = true
  use for large project/session/corpus memories
```

**Commit:**
```bash
git add AiDENs/docs/semantic-memory-backend-profiles.md
git commit -m "docs(aidens): define semantic-memory backend profiles"
```

### Task 12.2: Add config constants or builders in AiDENs memory/profile crates

**Objective:** AiDENs profile crates can request the same semantic-memory backend profiles consistently.

**Files:**
- Inspect and modify relevant crates:
  - `AiDENs/crates/aidens-memory-kit/src`
  - `AiDENs/crates/aidens-profile-memory/src`
  - other active profile crates only if they already configure semantic-memory
- Test: matching tests under those crates.

**API shape:**
```rust
pub enum AidensSemanticMemoryScaleProfile {
    Small,
    Medium,
    Large,
}
```

**Mapping:**
- Small -> disabled
- Medium -> TurboQuant candidate + exact rerank
- Large -> proveKV pool candidate + exact rerank

**Rule:** If AiDENs crates do not currently depend on semantic-memory config types, keep this as serializable config data rather than adding a hard dependency.

**Command:**
```bash
cargo test -p aidens-memory-kit
cargo test -p aidens-profile-memory
```

**Commit:**
```bash
git add AiDENs/crates/aidens-memory-kit AiDENs/crates/aidens-profile-memory
git commit -m "feat(aidens): add semantic-memory scale profile builders"
```

### Phase 12 gate

```bash
cargo test -p aidens-memory-kit
cargo test -p aidens-profile-memory
```

---

## Phase 13: Documentation and cross-crate contract tests

### Task 13.1: Add global architecture doc

**Objective:** Document the stack-level integration once.

**Files:**
- Create: `docs/provekv-derived-candidate-architecture.md`

**Required sections:**
- Ownership boundaries.
- Data flow diagram.
- Candidate vs exact rerank vs verified premise.
- Which crates may depend directly on compression crates.
- Receipt fields and propagation map.
- Excluded Recall / Recall-Coding note.

**Data flow:**
```text
source systems / forge
  -> semantic-memory authoritative projections + f32 embeddings
  -> semantic-memory proveKV/poly-kv pool generations
  -> knowledge-runtime / llm-tool-runtime / agent-graph retrieval
  -> llm-pipeline prompt/provider receipts
  -> kernel/claim systems verify bounded inputs
```

**Commit:**
```bash
git add docs/provekv-derived-candidate-architecture.md
git commit -m "docs: add provekv derived candidate architecture"
```

### Task 13.2: Add workspace guard script for proveKV boundary

**Objective:** Prevent direct proveKV dependency sprawl and Recall changes.

**Files:**
- Create: `scripts/validate_provekv_integration_boundaries.py`

**Script checks:**
1. No `Recall` / `Recall-Coding` paths in current diff.
2. Only allowed crates depend on `poly-kv`, `fib-quant`, `turbo-quant` directly:
   - `semantic-memory`
   - `quant-governor`
   - `scr-runtime-compression`
   - compression crates/benches
3. Downstream crates use receipt/provenance types or traits, not direct compression APIs.
4. `DerivedVectorBackendPolicy::ProveKvPoolCandidateOnly` still requires exact rerank.
5. No docs claim proveKV reduces provider/framework KV cache bytes directly.

**Command:**
```bash
python scripts/validate_provekv_integration_boundaries.py --root /home/sikmindz/Coding/Libraries
```

**Commit:**
```bash
git add scripts/validate_provekv_integration_boundaries.py
git commit -m "test: add provekv integration boundary validator"
```

### Task 13.3: Add cross-crate smoke test package or script

**Objective:** Validate semantic-memory -> KR -> pipeline/claim receipt flow at a high level.

**Files:**
- Create: `scripts/provekv_stack_smoke.sh`

**Script:**
```bash
#!/usr/bin/env bash
set -euo pipefail
cargo test -p semantic-memory --features poly-kv-pool --test provekv_pool_receipts
cargo test -p knowledge-runtime --features semantic-memory/poly-kv-pool --test cross_crate_proof provekv
cargo test -p llm-pipeline --test provider_context_provenance
cargo test -p kernel-oracles --test retrieved_vs_verified
cargo test -p claim-ledger --test proof_packet_candidate_provenance
python scripts/validate_provekv_integration_boundaries.py --root .
```

**Commit:**
```bash
git add scripts/provekv_stack_smoke.sh
git commit -m "test: add provekv stack smoke gate"
```

### Phase 13 gate

```bash
bash scripts/provekv_stack_smoke.sh
```

---

## Final validation gate

Run from `/home/sikmindz/Coding/Libraries`:

```bash
# Boundary guard first
python scripts/validate_provekv_integration_boundaries.py --root .

# Core functionality
cargo test -p semantic-memory --features poly-kv-pool,turbo-quant-codec
cargo test -p knowledge-runtime --features semantic-memory/poly-kv-pool
cargo test -p forge-memory-bridge --features semantic-memory/poly-kv-pool
cargo test -p llm-tool-runtime
cargo test -p agent-graph
cargo test -p llm-pipeline
cargo test -p kernel-execution
cargo test -p kernel-oracles
cargo test -p semantic-memory-forge
cargo test -p claim-ledger

# AiDENs profile crates if present in active workspace
cargo test -p aidens-memory-kit || true
cargo test -p aidens-profile-memory || true

# Smoke all selected cross-crate guarantees
bash scripts/provekv_stack_smoke.sh

# No excluded app changes
git diff --name-only | grep -E '(^|/)Recall(-Coding)?/' && exit 1 || true
```

Expected final state:
- `semantic-memory` can materialize, persist, status-check, receipt, and search through proveKV/poly-kv pool generations.
- `knowledge-runtime` traces proveKV backend use and applies route-aware safety policy.
- `forge-memory-bridge` records/requests derived artifact generation after imports.
- `llm-tool-runtime`, `agent-graph`, and `llm-pipeline` preserve retrieval/generation provenance without direct proveKV dependency.
- `kernel-*` and `claim-ledger` distinguish candidate discovery from verification.
- `semantic-memory-forge` supports audit/explain-only candidate search boundaries.
- AiDENs has standard semantic-memory scale profiles.
- Recall / Recall-Coding remain untouched.

---

## Implementation order summary

1. semantic-memory types/schema/snapshot/pool builder/rebuild/search/receipts.
2. semantic-memory retrieval benchmark receipts.
3. knowledge-runtime config/trace/route policy/cross-crate fallback test.
4. forge-memory-bridge import lifecycle.
5. skip Recall / Recall-Coding.
6. llm-tool-runtime searchable tool observations.
7. agent-graph graph memory provenance and retriever trait.
8. llm-pipeline retrieved context provenance.
9. kernel-execution/kernel-oracles candidate-vs-verified boundary.
10. semantic-memory-forge audit-only candidate search.
11. claim-ledger similar claim/proof-packet candidate provenance.
12. AiDENs profile standardization.
13. global docs and boundary smoke gates.

---

## Common failure modes to watch

- Accidentally implementing proveKV as per-vector `VectorCodec`; this loses the pool economics.
- Adding direct `poly-kv` dependency to every downstream crate; this creates architecture sprawl.
- Returning approximate candidates without exact rerank.
- Treating semantic similarity as identity resolution or claim verification.
- Silent fallback when pool generation is missing; must be receipted.
- Forgetting feature gates for `poly-kv-pool`.
- Cargo path pointing at excluded `poly-kv` workspace root instead of `poly-kv/crates/poly-kv` if that is the actual crate member.
- Docs implying provider/framework KV-cache memory reduction.
- Touching Recall / Recall-Coding despite explicit exclusion.

---

## Commit policy

Commit after each task. Use prefixes:
- `feat(semantic-memory): ...`
- `test(semantic-memory): ...`
- `feat(knowledge-runtime): ...`
- `feat(forge-memory-bridge): ...`
- `feat(llm-tool-runtime): ...`
- `feat(agent-graph): ...`
- `feat(llm-pipeline): ...`
- `feat(kernel-oracles): ...`
- `feat(claim-ledger): ...`
- `docs: ...`

Before merging, squash only if all intermediate test receipts are preserved in PR/body or final commit message.
