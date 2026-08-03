# PolyKV/FibQuant Live Stack Differentiator — Implementation Plan

> **For Hermes:** Execute phase-sequenced with delegate_task for independent workstreams. Do not commit until all gates pass.

**Goal:** Turn PolyKV/FibQuant into a live, actively used capability in the RecursiveIntell stack with shared immutable prefix state across agent branches, tiered compression, quality governance, durable artifacts, automatic exact fallback, and measured end-to-end evidence.

**Architecture:** One canonical Rust crate set (nested `poly-kv/crates/poly-kv` + `quant-codec-core`), one narrow first runtime (Transformers DynamicCache adapter for SmolLM2/Llama-family), durable pool persistence, quality-governed compressed cold pool + exact hot tails, and a local `llm-pipeline` insertion point.

**Tech Stack:** Rust 2021 (MSRV 1.75), Hugging Face Transformers Python, FibQuant, PolyKV, quant-governor, scr-runtime-compression, quant-eval.

---

## Phase 0 — Canonicalize implementation authority

### Task 0.1: Create isolated integration worktree
**Files:** New worktree at `/home/sikmindz/Coding/Libraries-polykv-integration`
```bash
git worktree add /home/sikmindz/Coding/Libraries-polykv-integration fix/hostile-remediation-20260715
cd /home/sikmindz/Coding/Libraries-polykv-integration
git status --short | wc -l  # must be 0
```

### Task 0.2: Preserve raw patches of current FibQuant/PolyKV changes
**Files:** Create `docs/patches/2026-08-01-pre-integration-baseline/`
```bash
mkdir -p docs/patches/2026-08-01-pre-integration-baseline/
git diff HEAD -- poly-kv/ fib-quant/ > docs/patches/2026-08-01-pre-integration-baseline/fibquant-polykv.patch
git status --short -- poly-kv/ fib-quant/ > docs/patches/2026-08-01-pre-integration-baseline/status.txt
```

### Task 0.3: Amend AGENTS.md - close no-system-integration phase
**Files:** `poly-kv/AGENTS.md`
Replace the rule "No semantic-memory, Gloss, Recall, AiDENs, ClaimLedger, or scr-runtime integration in this pass" with:
> Integration is now authorized under strict ownership rules. PolyKV owns pool lifecycle/manifests/readers/persistence only. Runtime adapters own extraction/injection only. Semantic-memory owns durable references only. Integration must preserve exact fallback, immutable shared pool blocks, per-reader state isolation, and receipt-bound claim boundaries.

### Task 0.4: Build capability migration ledger from divergent worktree
**Files:** Create `poly-kv/docs/migration-ledger-2026-08-01.md`
Inventory from `/home/sikmindz/Coding/Libraries-context-governor-fix/poly-kv/`:
- Compressed candidate scoring (`attention_topk_compressed`)
- `AgentShell` and `CompressedShellAttentionSelection`
- Prepared/prefetched indices (`FullyPreparedCompressedIndex`, `PrefetchedGramRows`)
- Adaptive per-head budgets
- Real-model replay/PPL harnesses (`scripts/ppl_validate.py`, `tools/real_corpus_ppl.py`)
- C kernels (`c-kernels/`)
- GPU backend experiments (`gpu-backend/`)
- Real receipts (`docs/codex-runs/P3/`)

Each entry: source file, capability, what to port, what to discard, owner boundary check.

**Gate 0:** Format/check/tests/strict Clippy pass from clean worktree. All four tasks complete with written artifacts.

---

## Phase 1 — Durable pool storage and recovery

### Task 1.1: Add `KvPoolStore` struct with persistence primitives
**Files:** Create `poly-kv/crates/poly-kv/src/store.rs`
```rust
pub struct KvPoolStore {
    root: PathBuf,
}

pub struct PersistedPool {
    pub manifest: KvPoolManifestV1,
    pub blocks: Vec<PathBuf>,
    pub exact_fallback_path: Option<PathBuf>,
}

impl KvPoolStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self>;
    pub fn persist(&self, pool: &SharedKvPool) -> Result<PersistedPool>;
    pub fn load(&self, manifest_digest: &ArtifactDigest) -> Result<(KvPoolManifestV1, Vec<Vec<u8>>)>;
    pub fn list_pools(&self) -> Result<Vec<KvPoolManifestV1>>;
    pub fn delete_pool(&self, manifest_digest: &ArtifactDigest) -> Result<()>;
    pub fn gc_unreferenced(&self, keep: &HashSet<ArtifactDigest>) -> Result<u64>;
}
```

### Task 1.2: Implement atomic write primitives
**Files:** `poly-kv/crates/poly-kv/src/store.rs`
Write to temp file, fsync, rename. Content-addressed by BLAKE3 digest.

### Task 1.3: Add manifest journal and startup replay
**Files:** `poly-kv/crates/poly-kv/src/store.rs`
Append-only journal: `{root}/journal/`. Each entry: action, manifest_digest, timestamp, status.

### Task 1.4: Add pool restart/recovery tests
**Files:** Create `poly-kv/crates/poly-kv/tests/pool_persistence.rs`
Tests:
- `persist_and_reload_pool` — round-trip through filesystem
- `restart_recovery_replay` — journal replay restores pool list
- `interrupted_write_recovery` — torn write detected and rejected
- `truncated_block_rejected` — missing bytes rejected
- `corrupted_block_digest_mismatch` — digest mismatch rejected
- `substituted_manifest_rejected` — manifest tampering detected
- `concurrent_readers_during_persistence` — readers unaffected
- `gc_removes_unreferenced` — unreferenced blocks cleaned
- `gc_preserves_referenced` — referenced blocks kept

**Gate 1:** All persistence tests pass. Format/check/Clippy green.

---

## Phase 2 — Transformers correctness adapter

### Task 2.1: Create Python adapter for DynamicCache extraction/injection
**Files:** Create `poly-kv/python/poly_kv/adapters/transformers_cache.py`
```python
class TransformersKVExtractor:
    """Extract genuine KV tensors from a Hugging Face DynamicCache."""
    def extract(self, past_key_values, model_config, tokenizer, input_ids, position_ids=None) -> KvCacheBundle;
    def verify_fingerprint(self, bundle: KvCacheBundle) -> bool;

class TransformersKVInjector:
    """Restore KV tensors into a DynamicCache."""
    def inject(self, bundle: KvCacheBundle, target_cache=None) -> DynamicCache;
    def compare_logits(self, model, token_ids, cache_a, cache_b, atol=1e-5) -> LogitComparison;
```

### Task 2.2: Add Rust-side adapter types
**Files:** Create `poly-kv/crates/poly-kv/src/adapters/transformers.rs`
```rust
pub struct TransformersCacheBundle {
    pub model_fingerprint: ModelFingerprint,
    pub tokenizer_fingerprint: TokenizerFingerprint,
    pub revision: String,
    pub config_digest: ArtifactDigest,
    pub shape: KvTensorShape,
    pub dtype: DType,
    pub layers: Vec<TransformersCacheLayer>,
    pub token_ids: Vec<u32>,
    pub position_ids: Vec<u32>,
    pub seq_len: u32,
}

pub struct TransformersCacheLayer {
    pub layer_idx: u32,
    pub key_tensor: Vec<f32>,
    pub value_tensor: Vec<f32>,
}

impl TransformersCacheBundle {
    pub fn into_pool_input(self) -> PoolInput;
    pub fn restore_dynamic_cache(&self) -> Vec<(Vec<f32>, Vec<f32>)>;
    pub fn verify_shape_consistency(&self) -> Result<()>;
}
```

### Task 2.3: Add extraction/restoration tests
**Files:** Create `poly-kv/crates/poly-kv/tests/transformers_adapter.rs`
- `exact_restoration_logit_parity` — restored cache produces identical logits
- `token_output_same_as_oracle` — same tokens from restored cache
- `branch_isolation_no_cross_mutation` — branch A write cannot affect branch B
- `interleaved_branch_stability` — branch outputs stable under other branch activity
- `wrong_model_revision_rejected` — different revision fails
- `wrong_tokenizer_rejected` — different tokenizer fails
- `shape_mismatch_rejected` — wrong shape fails
- `qwen35_hybrid_state_fails_closed` — unsupported cache family rejected

**Gate 2:** All adapter tests pass. Adapter rejects unsupported models cleanly.

---

## Phase 3 — First live shared-prefix lane

### Task 3.1: Add strict local API to PolyKV pool
**Files:** `poly-kv/crates/poly-kv/src/pool.rs`
```rust
impl SharedKvPool {
    pub fn prepare_prefix(bundle: TransformersCacheBundle) -> Result<(Self, PoolBuildReceiptV1)>;
    pub fn fork(&self, agent_id: &str) -> Result<BranchHandle>;
    pub fn release_branch(&self, handle: BranchHandle) -> Result<()>;
    pub fn pool_id(&self) -> &ArtifactDigest;
}
pub struct BranchHandle { /* ... */ }
impl BranchHandle {
    pub fn append(&mut self, tokens: &[u32]) -> Result<()>;
    pub fn generate(&self, model: &dyn ModelRuntime, max_tokens: u32) -> Result<Vec<u32>>;
    pub fn cache_state(&self) -> Result<DynamicCache>;
}
```

### Task 3.2: Integrate as optional local backend in llm-pipeline
**Files:** `llm-pipeline/src/backend/local_kv.rs`
Feature-gated behind `local-kv` feature flag. PolyKV disabled by default.

### Task 3.3: Add live integration tests
**Files:** Create `poly-kv/crates/poly-kv/tests/live_shared_prefix.rs`
- `shared_prefix_across_two_branches` — two agents share prefix
- `branch_isolation_under_mutation` — one branch's writes don't affect others
- `restart_recovery_between_sessions` — pool survives restart
- `ttft_improvement_on_reused_prefix` — second branch faster than cold start
- `memory_efficiency_vs_separate_caches` — shared pool uses less memory
- `disable_feature_restores_normal_behavior` — opt-out works

**Gate 3:** Live integration tests pass. No output-quality drift. PolyKV disabled by default.

---

## Phase 4 — Governed compressed cold tier

### Task 4.1: Add realized-size admission check
**Files:** `poly-kv/crates/poly-kv/src/pool.rs`
```rust
fn should_compress(encoded_bytes: u64, exact_bytes: u64, quality_gate: &QualityGateResultV1) -> bool {
    encoded_bytes < exact_bytes  // compression must actually save space
    && quality_gate.passed       // quality must be in budget
    && exact_fallback_available   // exact recovery must be possible
}
```

### Task 4.2: Port compressed-domain scoring from divergent implementation
**Files:** `poly-kv/crates/poly-kv/src/pool.rs` (extend existing `attention_topk_compressed`)
Port `PreparedCompressedIndex`, `FullyPreparedCompressedIndex`, `PrefetchedGramRows`, and adaptive per-head budgets. Keep FibQuant owner boundary — no duplicate codec math.

### Task 4.3: Add quality-gated admission tests
**Files:** Extend `poly-kv/crates/poly-kv/tests/fibquant_pool.rs`
- `tiny_artifact_stays_exact` — fixture 256B stays exact
- `quality_budget_exceeded_falls_back` — excessive MSE triggers fallback
- `corruption_triggers_exact_fallback` — tampered data triggers fallback
- `compressed_scoring_receipt_proves_no_full_decode` — receipt correct
- `adaptive_budget_respects_per_head_limits` — no budget violation

**Gate 4:** All compression tests pass. Receipts prove selected-value-only decode. Exact fallback is automatic.

---

## Phase 5 — Production-runtime candidate (deferred)

Evaluate vLLM KV connector when GPU probe is complete. Prioritize if GTX 1070 supports current vLLM.

---

## Acceptance Gates (All Phases)

1. `cargo fmt --all -- --check` — pass
2. `cargo check --workspace --all-targets` — pass
3. `cargo test --workspace --all-targets` — pass
4. `cargo clippy --workspace --all-targets -- -D warnings` — pass
5. `git diff --check` — pass
6. All phase-specific gates above pass
7. No public claims of production readiness without reproduced evidence
8. Exact fallback is automatic and receipted for every compressed path

## Rollback

- Remove integration worktree
- Preserve original dirty root untouched
- Disable `local-kv` feature flag (Phase 3)
- PolyKV remains an optional dependency with no runtime effect when disabled

## Files Summary

| Phase | New Files | Modified Files |
|-------|-----------|----------------|
| 0 | `docs/patches/...`, `poly-kv/docs/migration-ledger-...` | `poly-kv/AGENTS.md` |
| 1 | `poly-kv/crates/poly-kv/src/store.rs`, `...tests/pool_persistence.rs` | `poly-kv/crates/poly-kv/src/lib.rs` |
| 2 | `poly-kv/python/poly_kv/adapters/transformers_cache.py`, `...adapters/transformers.rs`, `...tests/transformers_adapter.rs` | `poly-kv/crates/poly-kv/src/lib.rs`, `Cargo.toml` |
| 3 | `llm-pipeline/src/backend/local_kv.rs`, `...tests/live_shared_prefix.rs` | `poly-kv/crates/poly-kv/src/pool.rs`, `llm-pipeline/Cargo.toml` |
| 4 | None (extend existing) | `poly-kv/crates/poly-kv/src/pool.rs`, `...tests/fibquant_pool.rs` |

## Timebox

- Phase 0: 15 minutes
- Phase 1: 30 minutes
- Phase 2: 30 minutes
- Phase 3: 30 minutes
- Phase 4: 30 minutes
- Total: ~2.5 hours wall-clock with parallelism
