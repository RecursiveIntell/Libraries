# RecursiveIntell ~/Coding/Libraries — Remediation Plan

**Date:** 2026-05-27  
**Authority:** `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md`  
**Doctrinal basis:** `01_CANONICAL_DOCTRINE_AND_SOURCE_HIERARCHY.md`, `artifact-runtime.md`, `evidence-first.md`, `governed-compression.md`, `polykv research.md`, `harness research.md`, `agentguard research.md`, `contract/hardening.md`  
**Target:** Close every P0 gap, every safety violation, and every doctrinal falsification in the audit.  
**Total estimated effort:** 40–60 hours of focused implementation + 20 hours of verification/benchmarking.

---

## Plan Philosophy

1. **Current source wins** — If a freshly uploaded file contradicts this plan, the upload wins.
2. **Receipts before release claims** — Every phase ends with a verification step. No phase is "done" without passing its gate.
3. **Append-plus-supersession** — Where possible, add new tables/types/modules rather than mutating old ones in place. Old code gets deprecation notes, not silent deletion.
4. **No shadow truth** — New indexes, caches, or compressed forms must be marked as advisory with explicit degradation receipts.
5. **Anti-false-completion** — Each phase has a falsification condition. If the condition is met, the phase is not complete.

---

## Phase 0 — Emergency Stabilization (0–4 hours)

**Goal:** Stop the bleeding. Fix safety violations, secure uncommitted work, and restore issue-tracking surfaces before any feature work begins.

### 0.1 Commit or quarantine the 178 uncommitted files

**Files:** Workspace root (`git status --short`)  
**Action:**
1. Run `git diff --stat` and read the diff to understand what changed.
2. If the diff contains real work (governance crate test fixtures, `verification-control/src/lib.rs` +266 lines, etc.), **commit it** with a descriptive message: `git add -A && git commit -m "WIP: post-V29 governance fixtures and verification-control expansion (uncommitted since 2026-05-27)"`.
3. If the diff contains noise ( regenerated schemas, auto-formatting), **stash it** or discard it intentionally with a note.
4. **Receipt:** `git log --oneline -1` must show the commit.

**Falsification condition:** Any uncommitted file remains after this step.

### 0.2 Fix `Primitives/check-runner` `unsafe` blocks

**Files:** `Primitives/check-runner/src/lib.rs` (lines ~240, ~807, ~828, ~840)  
**Action:**
1. Read the four `unsafe` blocks to understand what they do (process forking, `libc::kill`, signal handling).
2. **Option A (preferred if feasible):** Replace `libc::kill` with `nix::sys::signal::kill` from the `nix` crate, which provides a safe Rust wrapper. Replace raw `fork()` with `std::process::Command` if possible.
3. **Option B (if raw syscalls are truly required):** Extract the unsafe operations into a dedicated `check-runner-sys` crate with `#![allow(unsafe_code)]` and a big `// SAFETY:` comment explaining why each block is sound. The main `check-runner` crate stays `unsafe`-free and calls into `check-runner-sys`.
4. **Option C (if A and B are both infeasible):** Add an explicit `#[allow(unsafe_code)]` lint override to `Primitives/check-runner/Cargo.toml` with a justification comment, and document the override in the crate README. This is a last resort because it falsifies the `LIB-005` claim.

**Verification:** `cargo check --workspace` passes AND `grep -n "unsafe" Primitives/check-runner/src/lib.rs` returns zero matches (or only in `// SAFETY:` comments if Option B used).

**Falsification condition:** `unsafe` remains in production path without explicit override documented.

### 0.3 Replace `panic!` in `knowledge-runtime` and `kernel-oracles`

**Files:**
- `knowledge-runtime/src/query/classify.rs` (lines ~212, ~221, ~232)
- `kernel-oracles/src/lib.rs` (lines ~949, ~972)

**Action:**
1. Replace each `panic!` with a `thiserror` enum variant. Example:
   ```rust
   // Before:
   other => panic!("expected EntityLookup, got {:?}", other),
   // After:
   other => return Err(ClassifyError::UnexpectedLookupVariant {
       expected: "EntityLookup",
       got: format!("{:?}", other),
   }),
   ```
2. Update all callers to handle the new error variant (usually just bubbling via `?`).

**Verification:** `cargo check --workspace` passes AND `grep -n "panic!" knowledge-runtime/src/query/classify.rs kernel-oracles/src/lib.rs` returns zero matches.

**Falsification condition:** Any `panic!` remains in these files after the fix.

### 0.4 Replace `unwrap()` in `forge-memory-bridge/src/legacy.rs`

**Files:** `forge-memory-bridge/src/legacy.rs` (lines ~239, ~255, ~295)  
**Action:**
1. Replace each `unwrap()` with `thiserror` error propagation.
2. Add a `LegacyMigrationError` variant if one does not exist.

**Verification:** `cargo check -p forge-memory-bridge` passes AND `grep -n "unwrap()" forge-memory-bridge/src/legacy.rs` returns zero matches.

### 0.5 Restore deleted tracking documents

**Files:** `02_MASTER_ISSUE_MATRIX.md`, `06_RISK_REGISTER.md`  
**Action:**
1. If the files were deleted intentionally, restore them from git history: `git checkout HEAD~10 -- 02_MASTER_ISSUE_MATRIX.md 06_RISK_REGISTER.md` (or whatever commit last had them).
2. If they were superseded by newer formats (e.g., JSON tensors), create a migration note in `00_START_HERE.md` explaining the new canonical location.

**Verification:** Both files exist on disk OR `00_START_HERE.md` documents the supersession.

### 0.6 Run full lint suite

**Commands:**
```bash
cd ~/Coding/Libraries
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo doc --workspace --no-deps
```

**Action:** Fix every warning. Do not suppress warnings without a comment explaining why.

**Verification:** All three commands pass with zero warnings.

**Falsification condition:** Any warning remains unaddressed.

---

## Phase 1 — Foundation Crates (4–20 hours)

**Goal:** Build the four missing foundation crates that everything else depends on. These are net-new crates, not modifications.

### 1.1 Create `boundary-compiler` crate

**Location:** `~/Coding/Libraries/boundary-compiler/`  
**Files to create:**
- `boundary-compiler/Cargo.toml`
- `boundary-compiler/src/lib.rs`
- `boundary-compiler/src/jcs.rs`
- `boundary-compiler/src/duplicate_key.rs`
- `boundary-compiler/src/patch.rs`
- `boundary-compiler/src/schema.rs`
- `boundary-compiler/src/error.rs`
- `boundary-compiler/tests/jcs_tests.rs`
- `boundary-compiler/tests/patch_tests.rs`
- `boundary-compiler/tests/degradation_tests.rs`

**Doctrinal basis:** `artifact-runtime.md` Section 3 (boundary compiler as semantic front door); `contract/hardening.md` (type-owned contracts); RFC 8785, RFC 8259, RFC 6902.

**Design (minimum viable):**

```rust
// boundary-compiler/src/lib.rs
//! RFC 8785 JCS canonical JSON, duplicate-key rejection, JSON Patch dialect enforcement,
//! and deterministic serialization for hashing/signing.

pub mod jcs;
pub mod duplicate_key;
pub mod patch;
pub mod schema;
pub mod error;

pub use jcs::{canonicalize_json, canonicalize_value};
pub use duplicate_key::{parse_reject_dup_keys, parse_allow_dup_keys};
pub use patch::{JsonPatchDialect, apply_patch_receipt};
pub use schema::{validate_schema_receipt, SchemaValidationReceiptV1};
pub use error::BoundaryError;
```

**Key types:**
- `CanonicalJsonV1` — wrapper around a `serde_json::Value` that guarantees JCS canonical form.
- `DupKeyPolicy::Reject` / `DupKeyPolicy::Warn` / `DupKeyPolicy::Allow`
- `PatchDialect::Rfc6902` / `PatchDialect::Merge` / `PatchDialect::Custom`
- `PatchApplicationReceiptV1` — receipt for patch apply attempt, including success/failure, partial application, and rollback plan.
- `SchemaValidationReceiptV1` — receipt for schema validation against JSON Schema.

**Implementation notes:**
- JCS: Use `serde_json` + custom serializer that sorts object keys and serializes numbers in shortest decimal form. There is no mature Rust JCS crate; this may require a custom serializer. If time-constrained, start with a "JCS-lite" that only sorts keys and rejects duplicate keys, and document the gap.
- Duplicate keys: Hook into `serde_json::de::Deserializer` or pre-parse with a streaming JSON parser that detects duplicates.
- Patch: Use `json-patch` crate for RFC 6902, but wrap it to emit receipts and enforce atomicity (all-or-nothing per RFC 6902 semantics).
- Tests: Property tests with `proptest` for round-trip canonicalization. Golden tests with known JCS test vectors from the RFC.

**Verification:**
- `cargo test -p boundary-compiler` passes.
- `canonicalize_json(r#"{"b":1,"a":2}"#)` returns deterministic bytes.
- `parse_reject_dup_keys(r#"{"a":1,"a":2}"#)` returns `Err(BoundaryError::DuplicateKey("a"))`.
- Patch receipt includes `rollback_plan_ref` on failure.

**Falsification condition:** Cannot produce deterministic canonical JSON or cannot detect duplicate keys.

### 1.2 Create `bitemporal-runtime` crate

**Location:** `~/Coding/Libraries/bitemporal-runtime/`  
**Files to create:**
- `bitemporal-runtime/Cargo.toml`
- `bitemporal-runtime/src/lib.rs`
- `bitemporal-runtime/src/time.rs`
- `bitemporal-runtime/src/fact.rs`
- `bitemporal-runtime/src/query.rs`
- `bitemporal-runtime/src/supersession.rs`
- `bitemporal-runtime/src/receipt.rs`
- `bitemporal-runtime/src/error.rs`
- `bitemporal-runtime/tests/time_tests.rs`
- `bitemporal-runtime/tests/supersession_tests.rs`

**Doctrinal basis:** `01_CANONICAL_DOCTRINE...md` §3 (bitemporality mandatory); `artifact-runtime.md` (temporal truth); `bitemporal/replay.md`; `bitemporal/storage.md`.

**Design (minimum viable):**

```rust
// bitemporal-runtime/src/time.rs
use chrono::{DateTime, Utc};

/// A bitemporal timestamp pair.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BitemporalTime {
    /// When the fact applies in the modeled world.
    pub valid_time: DateTime<Utc>,
    /// When the system recorded, learned, asserted, or believed it.
    pub recorded_time: DateTime<Utc>,
}

/// A possibly-open interval in valid time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidInterval {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}
```

```rust
// bitemporal-runtime/src/fact.rs
/// A bitemporal fact is a typed claim with temporal scope.
pub struct BitemporalFact<V> {
    pub fact_id: String,         // content-addressed digest of payload + interval
    pub payload: V,
    pub valid_interval: ValidInterval,
    pub recorded_at: DateTime<Utc>,
    pub superseded_by: Option<String>, // fact_id of the superseding fact
    pub contradiction_of: Option<String>, // fact_id of the contradicted fact
}
```

```rust
// bitemporal-runtime/src/supersession.rs
/// Append-plus-supersession: never mutate in place.
pub enum FactEvolution<V> {
    Append(BitemporalFact<V>),           // new fact, no prior
    Supersede { old: String, new: BitemporalFact<V> }, // old closed, new opened
    Contradict { target: String, by: BitemporalFact<V>, resolution: ContradictionResolution },
    Invalidate { target: String, at: DateTime<Utc>, reason: String },
}
```

```rust
// bitemporal-runtime/src/query.rs
/// As-of query receipt.
pub struct AsOfQueryReceiptV1 {
    pub query_id: String,
    pub as_of_valid: DateTime<Utc>,
    pub as_of_recorded: DateTime<Utc>,
    pub facts_returned: Vec<String>, // fact_ids
    pub facts_excluded: Vec<String>, // fact_ids that were valid but recorded after as_of_recorded
    pub degradation: Option<QueryDegradation>,
}
```

**Implementation notes:**
- This crate is **types and logic only**, not a storage backend. Storage is the consumer's job.
- The `fact_id` should be a content-addressed digest (blake3) of `(canonical_json(payload), valid_interval.start, valid_interval.end)`.
- Supersession does not delete old facts. It sets `superseded_by` on the old fact and appends the new fact.
- Query semantics: "Give me all facts where valid_time overlaps the query interval AND recorded_time <= as_of_recorded, ordered by recorded_time DESC."
- Tests: Property tests for supersession chains, as-of query correctness, and interval overlap logic.

**Verification:**
- `cargo test -p bitemporal-runtime` passes.
- Supersession chain of length 10 can be queried at any recorded_time and returns correct fact.
- Contradiction receipt includes both the contradicted and contradicting fact IDs.

**Falsification condition:** Facts can be silently overwritten in place OR as-of query returns wrong fact for a given recorded_time.

### 1.3 Create `quant-governor` crate

**Location:** `~/Coding/Libraries/quant-governor/`  
**Files to create:**
- `quant-governor/Cargo.toml`
- `quant-governor/src/lib.rs`
- `quant-governor/src/policy.rs`
- `quant-governor/src/admissibility.rs`
- `quant-governor/src/receipt.rs`
- `quant-governor/src/eval.rs`
- `quant-governor/src/fallback.rs`
- `quant-governor/src/error.rs`
- `quant-governor/tests/policy_tests.rs`
- `quant-governor/tests/admissibility_tests.rs`

**Doctrinal basis:** `governed-compression.md` (codec family under governance); `polykv research.md` (honest accounting, content-addressed profiles); `evidence-first.md` (receipt semantics).

**Design (minimum viable):**

```rust
// quant-governor/src/policy.rs
/// A codec policy defines which codecs are allowed, in what order, and under what conditions.
pub struct CodecPolicyV1 {
    pub policy_id: String, // digest of the policy document
    pub allowed_families: Vec<CodecFamily>,
    pub default_family: CodecFamily,
    pub fallback_chain: Vec<CodecFamily>, // ordered: try first, then second, etc.
    pub exact_fallback_required: bool,    // must retain raw source?
    pub degradation_disclosure_required: bool,
    pub max_allowed_perplexity_delta: Option<f32>,
    pub max_allowed_recall_drop: Option<f32>,
}

pub enum CodecFamily {
    Raw,       // no compression
    Q8,        // 8-bit symmetric per-tensor
    TurboQuant,
    FibQuant,
    Mixed,     // adaptive: per-layer or per-head selection
}
```

```rust
// quant-governor/src/admissibility.rs
/// Admissibility class for a compressed artifact.
pub enum AdmissibilityClass {
    Strict,      // no loss tolerated; exact fallback mandatory
    Standard,    // small loss tolerated with disclosure
    Degraded,    // known loss, explicit degradation receipt
    Quarantined, // failed evaluation, must not be promoted
}

pub struct AdmissibilityDecisionV1 {
    pub artifact_id: String,
    pub codec_family: CodecFamily,
    pub class: AdmissibilityClass,
    pub eval_receipt_id: String,
    pub fallback_artifact_id: Option<String>, // link to raw/exact artifact
}
```

```rust
// quant-governor/src/receipt.rs
/// The receipt emitted when the governor selects and evaluates a codec.
pub struct CodecGovernanceReceiptV1 {
    pub receipt_id: String,
    pub input_artifact_id: String,
    pub selected_codec: CodecFamily,
    pub policy_id: String,
    pub decision: AdmissibilityDecisionV1,
    pub eval_summary: EvalSummary,
    pub fallback_artifact_id: Option<String>,
    pub degradation_notice: Option<DegradationNotice>,
    pub proof_debt: Vec<ProofDebt>,
}
```

```rust
// quant-governor/src/fallback.rs
/// Exact fallback retention: the raw source must remain accessible.
pub struct ExactFallbackV1 {
    pub raw_artifact_id: String,
    pub raw_digest: String,
    pub compressed_artifact_id: String,
    pub retention_policy: RetentionPolicy,
}

pub enum RetentionPolicy {
    Permanent,      // raw kept forever
    Duration(chrono::Duration),
    UntilSuperseded,
}
```

**Implementation notes:**
- This crate does **not** implement codec math. It depends on `turbo-quant` and `fib-quant` as optional adapters behind trait bounds.
- The codec adapter trait:
  ```rust
  pub trait GovernedCodec {
      fn family(&self) -> CodecFamily;
      fn encode(&self, input: &TensorView, profile: &CodecProfile) -> Result<EncodedArtifact, CodecError>;
      fn decode(&self, artifact: &EncodedArtifact) -> Result<TensorView, CodecError>;
      fn profile_digest(&self, profile: &CodecProfile) -> String; // content-addressed
      fn honest_byte_accounting(&self, artifact: &EncodedArtifact) -> ByteAccounting;
  }
  ```
- `ByteAccounting` must report: pool_bytes, metadata_bytes, decoded_working_bytes, per_reader_bytes.
- Tests: Policy routing tests, fallback chain tests, admissibility downgrade tests.

**Verification:**
- `cargo test -p quant-governor` passes.
- A `Strict` policy with `exact_fallback_required=true` rejects any encode attempt that does not produce a fallback link.
- A `Degraded` class includes a `DegradationNotice` in the receipt.

**Falsification condition:** Governor allows compression without exact fallback when policy requires it.

### 1.4 Create `claim-ledger` crate

**Location:** `~/Coding/Libraries/claim-ledger/`  
**Files to create:**
- `claim-ledger/Cargo.toml`
- `claim-ledger/src/lib.rs`
- `claim-ledger/src/claim.rs`
- `claim-ledger/src/evidence.rs`
- `claim-ledger/src/adjudication.rs`
- `claim-ledger/src/boundary.rs`
- `claim-ledger/src/receipt.rs`
- `claim-ledger/src/error.rs`
- `claim-ledger/tests/claim_tests.rs`
- `claim-ledger/tests/adjudication_tests.rs`
- `claim-ledger/tests/boundary_tests.rs`

**Doctrinal basis:** `evidence-first.md` (claim/evidence adjudication); `artifact-runtime.md` (structured claims); `01_CANONICAL_DOCTRINE...md` §8 (public claim discipline).

**Design (minimum viable):**

```rust
// claim-ledger/src/claim.rs
/// A claim is a typed assertion with an evidence requirement.
pub struct ClaimV1 {
    pub claim_id: String,
    pub claim_text: String,
    pub claim_category: ClaimCategory,
    pub evidence_requirement: EvidenceRequirement,
    pub proof_basis: Vec<String>, // evidence_ids
    pub evidence_class: EvidenceClass, // E0, E1, E2, E3
    pub public_safe_status: PublicSafeStatus,
    pub contradiction_status: ContradictionStatus,
    pub verification_needed: Vec<String>,
}

pub enum ClaimCategory {
    Performance,
    Existence,
    Maturity,
    Security,
    Research,
}

pub enum EvidenceClass {
    E0, // Directly inspected
    E1, // First-party claim
    E2, // Synthesis/inference
    E3, // Speculative
}

pub enum PublicSafeStatus {
    Allowed,
    AllowedWithContext,
    Quarantined,
    Forbidden,
}
```

```rust
// claim-ledger/src/adjudication.rs
/// Adjudication result for a claim against its evidence.
pub struct AdjudicationV1 {
    pub claim_id: String,
    pub adjudicator: String,
    pub verdict: Verdict,
    pub evidence_assessed: Vec<String>,
    pub gaps_found: Vec<String>,
    pub promotion_path: Option<PromotionPath>,
    pub downgrade_path: Option<DowngradePath>,
}

pub enum Verdict {
    Confirmed,      // evidence supports claim at stated class
    Demoted,        // evidence supports lower class only
    Quarantined,    // evidence insufficient for any public claim
    Contradicted,   // evidence contradicts claim
}
```

```rust
// claim-ledger/src/boundary.rs
/// Public-safe language filtering: quarantine unsupported claims before release.
pub struct PublicBoundaryCheckV1 {
    pub claim_id: String,
    pub original_text: String,
    pub filtered_text: Option<String>,
    pub violations: Vec<BoundaryViolation>,
    pub safe_to_publish: bool,
}

pub enum BoundaryViolation {
    UnsupportedPerformanceClaim,
    UnverifiedMaturityClaim,
    ResearchAnalogyAsFact,
    E3ClaimWithoutQuarantine,
    MissingReceiptLink,
}
```

**Implementation notes:**
- This crate is **not** a full NLP pipeline. It is a structured claim/evidence ledger with rules-based boundary checking.
- The boundary checker uses simple pattern matching + the claim's `evidence_class` and `public_safe_status`.
- Integration with `semantic-memory`: claims and adjudications are stored as memory artifacts with receipt linking.
- Integration with `boundary-compiler`: claim JSON must pass canonicalization before hashing.

**Verification:**
- `cargo test -p claim-ledger` passes.
- An E3 claim with `PublicSafeStatus::Quarantined` fails boundary check with `E3ClaimWithoutQuarantine`.
- An E0 claim with receipts passes boundary check.

**Falsification condition:** An E3 claim is marked `safe_to_publish=true`.

---

## Phase 2 — Core Integration (20–35 hours)

**Goal:** Wire the foundation crates into the existing codebase. This is where most of the file-touching happens.

### 2.1 Integrate `bitemporal-runtime` into `semantic-memory`

**Files:**
- `semantic-memory/Cargo.toml` — add `bitemporal-runtime` dep
- `semantic-memory/src/types.rs` — add `BitemporalEpisodeV1`, `BitemporalClaimV1`
- `semantic-memory/src/db.rs` — add bitemporal columns to `episodes`, `claims`, `search_receipts` tables
- `semantic-memory/src/episodes.rs` — replace in-place `update_outcome` with supersession
- `semantic-memory/src/lib.rs` — add `query_as_of()` API

**Action (step by step):**

1. **Add dependency:**
   ```toml
   [dependencies]
   bitemporal-runtime = { path = "../bitemporal-runtime" }
   ```

2. **Extend DB schema (append-only migration):**
   ```sql
   -- Add bitemporal columns to episodes table
   ALTER TABLE episodes ADD COLUMN valid_start TEXT;
   ALTER TABLE episodes ADD COLUMN valid_end TEXT;
   ALTER TABLE episodes ADD COLUMN recorded_time TEXT NOT NULL DEFAULT (datetime('now'));
   ALTER TABLE episodes ADD COLUMN superseded_by TEXT;
   ALTER TABLE episodes ADD COLUMN fact_digest TEXT;
   
   -- Add bitemporal columns to search_receipts
   ALTER TABLE search_receipts ADD COLUMN as_of_valid TEXT;
   ALTER TABLE search_receipts ADD COLUMN as_of_recorded TEXT;
   ```
   Implement this as a V20 migration in `semantic-memory/src/db.rs`.

3. **Replace `update_outcome` with supersession:**
   ```rust
   // semantic-memory/src/episodes.rs
   pub fn supersede_episode_outcome(
       conn: &mut Connection,
       old_episode_id: &str,
       new_outcome: EpisodeOutcome,
   ) -> Result<BitemporalFact<EpisodeOutcome>, MemoryError> {
       // 1. Close the old episode's valid interval
       // 2. Create a new episode fact with the new outcome
       // 3. Link old.superseded_by = new.fact_id
       // 4. Emit SupersessionReceiptV1
   }
   ```
   Deprecate `update_episode_outcome` with `#[deprecated(note = "Use supersede_episode_outcome for bitemporal compliance")]`.

4. **Add `as_of` query:**
   ```rust
   pub fn query_episodes_as_of(
       conn: &Connection,
       as_of_valid: DateTime<Utc>,
       as_of_recorded: DateTime<Utc>,
   ) -> Result<Vec<EpisodeAsOfReceiptV1>, MemoryError> {
       // SELECT * FROM episodes
       // WHERE valid_start <= ?1 AND (valid_end IS NULL OR valid_end > ?1)
       // AND recorded_time <= ?2
       // AND superseded_by IS NULL
       // ORDER BY recorded_time DESC
   }
   ```

**Verification:**
- `cargo test -p semantic-memory` passes.
- A test creates an episode, updates its outcome via supersession, then queries `as_of_recorded` before and after the update. The query returns the old and new facts respectively.
- The old episode row has `superseded_by` set to the new fact's ID.

**Falsification condition:** `update_episode_outcome` still mutates in place OR `as_of` query returns superseded facts.

### 2.2 Mark HNSW as approximate in `semantic-memory` receipts

**Files:**
- `semantic-memory/src/types.rs`
- `semantic-memory/src/db.rs`
- `semantic-memory/src/search.rs`
- `semantic-memory/src/hnsw.rs`

**Action:**

1. Add `approximate: bool` and `backend: SearchBackend` to `VectorSearchReceiptV1`:
   ```rust
   pub enum SearchBackend {
       Exact,       // brute-force or exact index
       Hnsw,        // approximate nearest neighbor
       Compressed,  // searched over quantized vectors
   }
   
   pub struct VectorSearchReceiptV1 {
       // ... existing fields ...
       pub approximate: bool,
       pub backend: SearchBackend,
       pub degradation_notice: Option<DegradationNotice>,
   }
   ```

2. In `search.rs`, when HNSW is used, set `approximate = true` and `backend = SearchBackend::Hnsw`.

3. Add a `DegradationNotice` explaining that HNSW results may miss exact nearest neighbors.

**Verification:**
- A test queries semantic memory via HNSW and verifies the receipt has `approximate: true`.
- A test queries via brute-force and verifies `approximate: false`.

**Falsification condition:** HNSW query receipt lacks `approximate: true`.

### 2.3 Integrate `quant-governor` into `semantic-memory` codec path

**Files:**
- `semantic-memory/Cargo.toml` — add `quant-governor` dep
- `semantic-memory/src/quantize.rs` — replace direct `turbo-quant` calls with governor routing
- `semantic-memory/src/db.rs` — add `codec_governance_receipt_id` to `derived_vector_artifacts`
- `semantic-memory/src/types.rs` — add `CodecGovernanceReceiptV1` link

**Action:**

1. **Replace `quantize.rs` codec selection:**
   ```rust
   // Before: direct turbo-quant call
   // After: governor-mediated call
   pub fn encode_with_governor(
       &self,
       embedding: &[f32],
       policy: &CodecPolicyV1,
   ) -> Result<(DerivedVectorArtifactV1, CodecGovernanceReceiptV1), MemoryError> {
       let governor = QuantGovernor::new(policy);
       let (encoded, receipt) = governor.encode(embedding)
           .map_err(|e| MemoryError::GovernedCompressionFailed(e.to_string()))?;
       
       // Store the receipt in DB
       let receipt_id = receipt.receipt_id.clone();
       self.store_codec_receipt(&receipt)?;
       
       Ok((encoded, receipt))
   }
   ```

2. **Add exact fallback column:**
   ```sql
   ALTER TABLE derived_vector_artifacts ADD COLUMN raw_source_artifact_id TEXT;
   ALTER TABLE derived_vector_artifacts ADD COLUMN codec_governance_receipt_id TEXT;
   ```

3. **Enforce fallback retention:** If `policy.exact_fallback_required`, the encode path must:
   - Store the raw embedding in `vector_embeddings` (or a new `raw_vector_artifacts` table).
   - Set `derived_vector_artifacts.raw_source_artifact_id` to the raw artifact's ID.
   - Include the raw artifact ID in the `CodecGovernanceReceiptV1.fallback_artifact_id`.

**Verification:**
- `cargo test -p semantic-memory --features turbo-quant-codec` passes.
- A test with `CodecPolicyV1 { exact_fallback_required: true, .. }` fails if fallback is missing.
- A test with `CodecPolicyV1 { exact_fallback_required: false, .. }` succeeds without fallback.
- The receipt includes `ByteAccounting` with all four fields (pool, metadata, decoded, per-reader).

**Falsification condition:** Compression proceeds without exact fallback when policy requires it OR byte accounting is missing.

### 2.4 Add execution evidence receipts to `llm-pipeline`

**Files:**
- `llm-pipeline/Cargo.toml` — add `stack-ids`, `bitemporal-runtime` deps
- `llm-pipeline/src/types.rs` — add receipt types
- `llm-pipeline/src/pipeline.rs` — emit `PipelineExecutionReceiptV1`
- `llm-pipeline/src/retry_policy.rs` — emit `RetryDecisionReceiptV1`
- `llm-pipeline/src/backend/mod.rs` — emit `ProviderCallReceiptV1`
- `llm-pipeline/src/trace.rs` — link trace events to receipt IDs

**Action (new types):**

```rust
// llm-pipeline/src/types.rs
use stack_ids::{ContentDigest, DigestBuilder};
use bitemporal_runtime::BitemporalTime;

pub struct PipelineExecutionReceiptV1 {
    pub receipt_id: String,
    pub pipeline_id: String,
    pub plan_digest: String,
    pub trace_receipt_id: String,
    pub provider_calls: Vec<String>, // ProviderCallReceiptV1 IDs
    pub retry_decisions: Vec<String>, // RetryDecisionReceiptV1 IDs
    pub budget_debits: Vec<BudgetDebitV1>,
    pub response_digest: String,
    pub outcome: ExecutionOutcome,
    pub bitemporal_time: BitemporalTime,
}

pub struct ProviderCallReceiptV1 {
    pub receipt_id: String,
    pub provider: String, // "ollama", "openai"
    pub model_route: String,
    pub request_digest: String,
    pub response_digest: String,
    pub latency_ms: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
}

pub struct RetryDecisionReceiptV1 {
    pub receipt_id: String,
    pub attempt_number: u32,
    pub max_attempts: u32,
    pub cause: RetryCause,
    pub decision: RetryDecision,
    pub budget_impact: BudgetDebitV1,
}
```

**Implementation notes:**
- The `pipeline.rs` `execute()` method should collect all sub-receipts and emit a top-level `PipelineExecutionReceiptV1` at the end.
- Each backend call (Ollama, OpenAI) emits a `ProviderCallReceiptV1`.
- Each retry emits a `RetryDecisionReceiptV1`.
- Budget tracking: token counts and time budgets are recorded per call.

**Verification:**
- `cargo test -p llm-pipeline` passes.
- A test runs a pipeline with retries and verifies the receipt chain contains:
  - 1 `PipelineExecutionReceiptV1`
  - N `ProviderCallReceiptV1` (one per attempt)
  - N-1 `RetryDecisionReceiptV1` (one per retry)
  - At least 1 `BudgetDebitV1`

**Falsification condition:** Receipt chain is missing any required artifact type.

### 2.5 Add execution evidence receipts to `agent-graph`

**Files:**
- `agent-graph/Cargo.toml` — add `stack-ids`, `bitemporal-runtime` deps
- `agent-graph/src/lib.rs` — add receipt types to public API
- `agent-graph/src/executor.rs` (or equivalent) — emit `GraphExecutionReceiptV1`
- `agent-graph/src/checkpoint.rs` — emit `CheckpointReceiptV1`

**Action (new types):**

```rust
pub struct GraphExecutionReceiptV1 {
    pub receipt_id: String,
    pub graph_id: String,
    pub step_receipts: Vec<String>, // StepExecutionReceiptV1 IDs
    pub checkpoint_receipts: Vec<String>, // CheckpointReceiptV1 IDs
    pub final_state_digest: String,
    pub bitemporal_time: BitemporalTime,
}

pub struct StepExecutionReceiptV1 {
    pub receipt_id: String,
    pub step_id: String,
    pub node_type: String,
    pub input_digest: String,
    pub output_digest: String,
    pub tool_call_receipt_id: Option<String>, // links to llm-pipeline receipt
    pub latency_ms: u64,
}
```

**Implementation notes:**
- If `agent-graph` uses `llm-pipeline` for LLM nodes, link the `tool_call_receipt_id` to the `PipelineExecutionReceiptV1`.
- Checkpoints should emit `CheckpointReceiptV1` with the full state digest.

**Verification:**
- `cargo test -p agent-graph` passes.
- A test executes a 3-step graph and verifies the receipt chain contains 3 `StepExecutionReceiptV1` and at least 1 `CheckpointReceiptV1`.

**Falsification condition:** Graph execution produces no receipts.

### 2.6 Integrate `boundary-compiler` into `semantic-memory` and `forge-pilot`

**Files:**
- `semantic-memory/src/graph.rs` — replace `canonical_json_string()` with `boundary_compiler::canonicalize_value()`
- `forge-pilot/src/export.rs` — canonicalize exports before digesting
- `forge-pilot/src/import.rs` — validate imports against schema

**Action:**
1. Replace `semantic-memory/src/graph.rs` `canonical_json_string()`:
   ```rust
   // Before:
   fn canonical_json_string(value: &Option<serde_json::Value>) -> Result<String, serde_json::Error> {
       // naive key-sorting
   }
   // After:
   fn canonical_json_string(value: &Option<serde_json::Value>) -> Result<String, BoundaryError> {
       boundary_compiler::canonicalize_value(value.clone())
   }
   ```

2. In `forge-pilot/src/export.rs`, canonicalize `ExportEnvelopeV1` JSON before computing `compute_digest()`.

3. Add duplicate-key detection to JSON import paths.

**Verification:**
- `cargo test -p semantic-memory` passes.
- `cargo test -p forge-pilot` passes.
- A test with duplicate JSON keys is rejected at import boundary.

**Falsification condition:** `canonical_json_string` still uses naive key-sorter OR duplicate keys are accepted.

### 2.7 Integrate `claim-ledger` into `forge-pilot` public claim path

**Files:**
- `forge-pilot/Cargo.toml` — add `claim-ledger` dep
- `forge-pilot/src/export.rs` — run public boundary check before export
- `forge-pilot/src/cli.rs` — add `--public-claim-check` flag

**Action:**
1. Before any `ExportEnvelopeV1` is written to disk or network, pass its claims through `claim_ledger::PublicBoundaryCheckV1`.
2. If the check returns `safe_to_publish: false`, block the export and emit a `BoundaryViolationReceiptV1`.
3. Add a CLI flag `--public-claim-check` that runs the boundary check on a report without exporting.

**Verification:**
- `cargo test -p forge-pilot` passes.
- A test with an E3 claim is blocked from public export.
- A test with an E0 claim + receipts passes public export.

**Falsification condition:** An E3 claim reaches a public file without quarantine.

---

## Phase 3 — Hardening and Coverage (35–50 hours)

**Goal:** Close test gaps, fix dependency health, and add cross-crate integration.

### 3.1 Add tests to all Primitives

**Target:** `Primitives/*` (10 crates, ~4,900 lines, 0 tests)  
**Minimum test requirement per crate:**
- `typed-patch`: 5 tests for patch apply, patch canonicalization, patch receipt emission.
- `check-runner`: 5 tests for process spawn, signal handling, timeout, exit code parsing. (After unsafe removal.)
- `cea-sqlite`: 5 tests for SQLite open, write, read, migration.
- `cea-store`: 5 tests for store open, write, read.
- `stabilizer-core`: 3 tests for stabilization logic.
- `sandbox-workspace`: 3 tests for sandbox creation/destruction.
- `forge-policy`: 3 tests for policy evaluation.
- `mindstate-core`: 3 tests for mindstate transitions.
- `effect-signature`: 3 tests for signature verification.
- `cea-core`: 3 tests for re-export correctness.

**Action:**
1. For each crate, create `tests/basic_tests.rs` with the minimum tests above.
2. Use `tempfile` for temporary directories.
3. Use `assert_matches!` for error variant checking.

**Verification:**
- `cargo test --workspace` passes.
- `cargo test -p typed-patch`, `cargo test -p check-runner`, etc. each show >0 tests passing.

**Falsification condition:** Any Primitive crate shows 0 tests.

### 3.2 Add tests to `llm-output-parser` and `job-queue`

**Target:** `llm-output-parser` (11 files, 0 tests), `job-queue` (8 files, 0 tests)  
**Minimum tests:**
- `llm-output-parser`: 10 tests for JSON extraction, markdown fence parsing, YAML parsing, think-block removal, malformed input handling.
- `job-queue`: 8 tests for enqueue, dequeue, retry, timeout, executor spawn, event emission.

**Verification:** Same as 3.1.

### 3.3 Fix `semantic-memory` turbo-quant version string

**File:** `semantic-memory/Cargo.toml`  
**Action:**
```toml
# Before:
turbo-quant = { version = "0.2.0-alpha.1", path = "../turbo-quant", optional = true }
# After:
turbo-quant = { version = "0.2.0", path = "../turbo-quant", optional = true }
```

**Verification:** `cargo check --workspace` passes.

### 3.4 Integrate `poly-kv` into main workspace

**Files:**
- `poly-kv/Cargo.toml` — change `rust-version = "1.75"` to match workspace
- `~/Coding/Libraries/Cargo.toml` — add `poly-kv` to members and default-members

**Action:**
1. In `poly-kv/Cargo.toml`, remove the standalone workspace declaration and make it a regular crate.
2. In root `Cargo.toml`, add `poly-kv` to `members` and `default-members`.
3. Ensure `poly-kv` depends on `turbo-quant` and `fib-quant` via path deps.
4. Run `cargo check --workspace`.

**Verification:** `cargo check --workspace` passes including `poly-kv`.

**Falsification condition:** `poly-kv` builds outside the main workspace OR `cargo check --workspace` fails.

### 3.5 Implement `tauri-queue` or remove it

**Decision point:**
- **If Gloss needs a Tauri-specific queue:** Implement `tauri-queue` with Tauri command/event integration (2–4 hours).
- **If `job-queue` is sufficient:** Remove `tauri-queue` from workspace and replace all `tauri-queue` references with `job-queue`.

**Action (if implementing):**
1. Create `tauri-queue/src/lib.rs` with Tauri command handlers and event emitters.
2. Add tests.

**Action (if removing):**
1. Remove `tauri-queue` from root `Cargo.toml` members/default-members.
2. Move `tauri-queue/` to `_salvage_from_libraries2/` or delete if empty.
3. Update any docs referencing `tauri-queue`.

**Verification:** `cargo check --workspace` passes. No `tauri-queue` stub remains in the active workspace.

### 3.6 Add receipt emission to governance crates

**Target:** `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `constitutional-memory`, `continuity-runtime`, `effect-runtime`, `mechanism-runtime`  
**Action:**
1. For each crate, add a `receipt.rs` module with a typed receipt for the crate's primary artifact family.
   - `assurance-runtime`: `AssuranceCaseReceiptV1`
   - `attestation-exchange`: `AttestationReceiptV1`
   - `authority-delegation`: `DelegationReceiptV1`
   - etc.
2. Each receipt must include:
   - `receipt_id` (content-addressed digest)
   - `artifact_id` (link to the artifact being certified/attested/delegated)
   - `issuer` (crate name + version)
   - `timestamp` (bitemporal)
   - `valid_until` (optional expiration)
3. Add `stack-ids` as a dependency for `ContentDigest`/`DigestBuilder`.

**Verification:**
- `cargo check --workspace` passes.
- Each governance crate has at least 1 test that round-trips its receipt type through JSON.

**Falsification condition:** Governance crates still have zero receipt types.

### 3.7 Add cross-crate integration tests

**New file:** `integration-tests/` directory at workspace root (or `tests/` in `kernel-conformance`)  
**Tests to add:**
1. **semantic-memory + turbo-quant round-trip:**
   - Encode an embedding with `quant-governor` + `turbo-quant`.
   - Store in `semantic-memory`.
   - Decode and verify exact fallback is accessible.
   - Verify receipt chain is complete.

2. **forge-pilot + semantic-memory observation:**
   - Run `forge-pilot` observe phase on a test workspace.
   - Verify observations are stored in `semantic-memory` with `ObservationReceiptV1`.
   - Query `semantic-memory` for the observation by receipt ID.

3. **llm-pipeline + agent-graph:**
   - Create an agent graph with an LLM node.
   - Execute the graph.
   - Verify the graph receipt links to the pipeline receipt.

4. **boundary-compiler + claim-ledger:**
   - Canonicalize a claim JSON.
   - Run boundary check.
   - Verify the canonical form is stable across repeated canonicalization.

**Verification:**
- `cargo test -p integration-tests` (or equivalent) passes.
- Each integration test emits a receipt that links artifacts across crate boundaries.

---

## Phase 4 — Missing Crates and Benchmarks (50–60 hours)

**Goal:** Create the remaining net-new crates and establish performance baselines.

### 4.1 Create `receipt-bench` / `agent-harness` crate

**Location:** `~/Coding/Libraries/receipt-bench/`  
**Doctrinal basis:** `harness research.md` (ReceiptBench / Agent Harness Lab)

**Design (minimum viable):**
- Fixture runner that loads JSONL fixtures and executes them against the stack.
- Receipt validator that checks every run produced the required artifacts.
- Ablation runner that compares with/without memory, with/without compression, with/without governance.
- Report generator that emits a markdown report with `BenchmarkReceiptV1`.

**Tests:**
- B01 (environment-gotcha recall)
- B03 (false-completion detection)
- B04 (tool/policy compliance)
- B05 (memory usefulness over time)
- B06 (compression-loss impact)
- B11 (receipt completeness)
- B12 (replayability)

**Verification:**
- `cargo test -p receipt-bench` passes.
- At least 5 fixtures run and produce `BenchmarkReceiptV1`.

### 4.2 Create `agent-guard` crate skeleton

**Location:** `~/Coding/Libraries/agent-guard/`  
**Doctrinal basis:** `agentguard research.md`

**Design (minimum viable):**
- `agent-guard/src/launcher.rs` — controlled launcher with systemd scope + cgroup v2.
- `agent-guard/src/broker.rs` — MCP stdio broker with JSON-RPC framing validation.
- `agent-guard/src/receipt.rs` — admission receipts, process events, FD events, MCP session receipts.
- `agent-guard/src/policy.rs` — deny-by-default egress, protected-path policy.

**Note:** This is a skeleton with real types and a working broker. The eBPF/BPF LSM layer is P2 (requires kernel module work).

**Verification:**
- `cargo check -p agent-guard` passes.
- `cargo test -p agent-guard` passes (unit tests for broker framing, policy parsing).

### 4.3 Create `scr-runtime-compression` adapter

**Location:** `~/Coding/Libraries/scr-runtime-compression/`  
**Doctrinal basis:** `governed-compression.md`

**Design:**
- Thin adapter that binds `quant-governor` decisions into `forge-pilot` / `semantic-memory` operator execution.
- Not a codec crate. Does not own codec truth.
- Emits `CompressionOperatorReceiptV1` when compression is applied.

### 4.4 Performance baselines under release profile

**Target:** `turbo-quant`, `fib-quant`, `kernel-conformance`  
**Action:**
1. Add `[[bench]]` entries to `turbo-quant` and `fib-quant` if missing.
2. Run `cargo bench` in release profile.
3. Capture results to `evidence/perf_baseline_2026-05-27.json`.
4. Include: encode throughput, decode throughput, recall@k, exact-rerank recovery, memory breakdown.

**Verification:**
- `cargo bench -p turbo-quant` completes without error.
- `cargo bench -p fib-quant` completes without error.
- Baseline JSON is committed to `evidence/`.

---

## Appendix A — Task Tracker Template

For each task in this plan, use this format in your issue tracker:

```markdown
## TASK-XXX: <short description>

**Phase:** 0/1/2/3/4  
**Priority:** P0/P1/P2  
**Target crate(s):** <list>  
**Doctrinal basis:** <doc reference>  

### Acceptance criteria
- [ ] <criterion 1>
- [ ] <criterion 2>
- [ ] `cargo check --workspace` passes
- [ ] `cargo test -p <crate>` passes
- [ ] `cargo clippy --workspace` passes

### Receipt
- **What changed:**
- **Why:**
- **Verified:**
- **Proof debt:**
- **Falsifies if:**

### Rollback
- <how to undo if needed>
```

---

## Appendix B — Verification Checklist (Final Gate)

Before declaring this plan complete, verify:

- [ ] `cargo check --workspace` passes (all 49+ crates)
- [ ] `cargo test --workspace` passes (all tests green)
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo doc --workspace --no-deps` passes with zero broken links
- [ ] `grep -rn "unwrap()" --include="*.rs" . | grep -v tests/ | grep -v bench | wc -l` == 0 (or only in test modules)
- [ ] `grep -rn "panic!" --include="*.rs" . | grep -v tests/ | grep -v bench | wc -l` == 0 (or only in test modules)
- [ ] `grep -rn "unsafe" --include="*.rs" . | grep -v target/ | grep -v "// SAFETY" | wc -l` == 0 (or only in `check-runner-sys` with documented overrides)
- [ ] `grep -rn "todo!()\|unimplemented!()" --include="*.rs" . | grep -v tests/ | wc -l` == 0
- [ ] All 178 previously uncommitted files are committed or intentionally discarded
- [ ] `02_MASTER_ISSUE_MATRIX.md` and `06_RISK_REGISTER.md` restored or supersession documented
- [ ] `semantic-memory` has bitemporal columns and `as_of` query
- [ ] `quant-governor` exists and has passing tests
- [ ] `boundary-compiler` exists and has JCS tests
- [ ] `claim-ledger` exists and has boundary tests
- [ ] `bitemporal-runtime` exists and has supersession tests
- [ ] `receipt-bench` has at least 5 fixtures passing
- [ ] Performance baseline JSON exists in `evidence/`

---

## Receipt for This Plan

- **What this plan covers:** All 23 prioritized fixes from `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md`, organized into 5 phases with specific file paths, types, and verification gates.
- **What it does NOT cover:** Implementation of the plan itself (this is the plan, not the execution). AiDENs-specific crates, FEUT/SCR scientific claims (quarantined), cosmology analogies (quarantined).
- **Proof debt:** Time estimates are approximate (40–60 hours). Actual time depends on how many `semantic-memory` DB migrations are needed and how complex the JCS serializer is.
- **Falsifies if:** Any phase's falsification condition is met; any crate fails `cargo check` after the prescribed changes; the user uploads a newer spec that supersedes this plan.
- **Supersession policy:** This plan is append-only. If a new audit reveals additional gaps, add them as new phases or append to existing phases. Do not silently delete tasks.
