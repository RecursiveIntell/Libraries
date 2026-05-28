# Libraries Fix & Hardening Roadmap — V30 Master Plan

**Date:** 2026-05-27
**From:** `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md` findings
**Doctrinal basis:** `01_CANONICAL_DOCTRINE_AND_SOURCE_HIERARCHY.md`, `02_ARCHITECTURE_SOURCE_OF_TRUTH_AND_ARTIFACT_INDEX.md`, Full Provenance+ Research corpus

---

## Governing principles

1. **Receipts for all material operations.** No silent error swallowing. No dead code. No stubs as implemented.
2. **Bitemporal truth is mandatory**, not optional. `valid_time` / `recorded_time` / `as_of` / supersession or explicit degradation.
3. **RFC 8785 JCS** for every digest boundary. Naive key-sorting is not canonicalization.
4. **Governed compression is a runtime**, not a crate. Policy routing + exact fallback + degradation receipts are not optional.
5. **Commit before moving on.** 178 uncommitted files are a source-of-truth drift risk.
6. **No shadow truth.** HNSW indexes, projection caches, bootstrap deltas — all must mark themselves as advisory, not authoritative.
7. **Unsafe code is deny-by-default.** The workspace says `unsafe_code = deny`. `Primitives/check-runner` violates it — fix it.
8. **Test everything added.** Receipt types without tests are decorations, not evidence.

---

## Phase 0 — Pre-flight & Truth Recovery (1–2 hrs)

**Goal:** Establish a clean working state before touching any crate.

### 0.1 — Commit all 178 uncommitted files

```bash
cd ~/Coding/Libraries
git add -A
git commit -m "WIP: salvage commit — pre-V30 hardening, no feature changes"
```

Rationale: Source-of-truth drift risk. Nothing gets fixed on top of dirty state.

### 0.2 — Restore deleted tracking docs

Recreate from audit or memory:
- `02_MASTER_ISSUE_MATRIX.md` — issue tracker with all P0/P1/P2 from this audit
- `06_RISK_REGISTER.md` — open risks with status

### 0.3 — Fork the canonical audit

Copy the audit doc as a live working artifact:
```bash
cp LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md LIBRARIES_V30_HARDENING_PLAN.md
```

All subsequent work must emit receipts that update this doc.

---

## Phase 1 — Workspace Integrity (P0 — 1 week)

**Goal:** `cargo check --workspace` and `cargo test --workspace` pass cleanly. No violations of declared lint policy. Primitives tested.

### 1.1 — Fix `unsafe` in `Primitives/check-runner` or exclude it

**Problem:** 4 production `unsafe` blocks for process forking / `libc::kill` / signal handling. Workspace declares `unsafe_code = deny`.

**Path A (preferred):** Replace the `unsafe` blocks with safe wrappers. Use `std::process::Command` with owned stdin/stdout, `std::os::unix::ProcessExt` trait bounds instead of raw `libc`. Isolate signal handling behind a `#[ derive(Ab安全工作)]` wrapper that is formally verified.

**Path B:** If the unsafe surface cannot be safely eliminated, move `check-runner` out of the workspace members list and into `exclude` in `Cargo.toml`, document the exclusion rationale in `LIBRARIES_V30_HARDENING_PLAN.md`, and add a local `[lints]` override that is explicit and scoped.

**Verification:** `cargo check --workspace` still passes. `cargo test -p Primitives/check-runner` passes. `grep -r "unsafe" Primitives/check-runner/src/` returns only `unsafe_std` or test-only code.

### 1.2 — Replace `panic!` in `knowledge-runtime` and `kernel-oracles`

**Problem:** `knowledge-runtime/src/query/classify.rs` has 3 `panic!` on unexpected enum variant. `kernel-oracles/src/lib.rs` has 2 `panic!` on unexpected oracle result.

**Fix:** Replace `panic!` with structured error propagation using `thiserror`. Every enum variant that is "unexpected" at runtime must yield a typed error variant with a reason string, not an unchecked panic.

**Verification:** `cargo check --workspace` passes. `grep -rn "panic!" knowledge-runtime/src kernel-oracles/src | grep -v "test\|#\[cfg"` returns nothing.

### 1.3 — Add tests to Primitives

All 10 Primitives have zero tests. Priority order:
1. `typed-patch` (950 lines — untested, highest risk)
2. `check-runner` (846 lines — unsafe surface, critical)
3. `cea-sqlite` (1,218 lines — SQLite operations)
4. `cea-store`, `stabilizer-core`, `sandbox-workspace`, `forge-policy`, `mindstate-core`

**Minimal bar:** Each crate gets one integration test in `tests/` that calls its primary public API and asserts the output is well-typed. Receipt types in `stack-ids` are fair game for reuse.

**Verification:** `cargo test --workspace` covers every crate. Test ratio >= 0.20 for all Primitives by end of phase.

### 1.4 — Confirm workspace lint policy enforcement

Run:
```bash
cargo clippy --workspace -- -D warnings 2>&1 | grep -E "unsafe|unwrap|panic|todo|unimplemented"
```

Fix any lint violations found (other than the known `unsafe` in `check-runner`).

---

## Phase 2 — Missing Core Crates (P0 — 1–2 weeks)

**Goal:** The 8 missing crates mandated by doctrine are created as minimal-viableImplementations.

### 2.1 — `boundary-compiler`: RFC 8785 JCS canonical JSON

**What:** New crate `crates/boundary-compiler/`. Implements:
- RFC 8785 JSON Canonicalization (JCS)
- `Canonicalizer` struct: takes any `serde_json::Value`, returns a `String` in JCS order
- Duplicate-key rejection: parses with a custom JSON parser that errors on duplicate object keys
- `ContentDigest` compatible output: blake3 hash of the JCS string for content-addressing
- `BoundaryProfile` enum: declares dialect, schema ID+version, canonicalization profile, unknown-field policy, resource ceilings
- Schema validation: runtime check against JSON Schema before/after canonicalization

**Integration:** Replaces `semantic-memory/src/graph.rs:canonical_json_string()` with a call to `boundary-compiler`.

**Verification:**
- JCS round-trip: `value → canonicalize → parse → canonicalize → value` is identity
- Duplicate-key input returns `Err`
- Benchmark: canonicalize a 1MB JSON blob in < 50ms

### 2.2 — `bitemporal-runtime`: Bitemporal truth primitives

**What:** New crate `crates/bitemporal-runtime/`. Implements:
- `BitemporalRecord<T>`: `{ valid_time: DateTime<Utc>, recorded_time: DateTime<Utc>, value: T }`
- `append_supersede(db, record)` — appends new recorded-time row; marks prior rows as superseded
- `as_of_query(db, valid_time, recorded_time)` — returns records valid at `valid_time` as of `recorded_time`
- `temporal_snapshot(db, as_of_time)` — full state as of `as_of_time`
- `SupersessionReceipt`: receipt for every supersession event

**Integration:** `semantic-memory` adopts `bitemporal-runtime` primitives. Episodes, claims, vector artifacts all become bitemporal.

**Schema addition to semantic-memory:**
```sql
ALTER TABLE episodes ADD COLUMN valid_time DATETIME;
ALTER TABLE episodes ADD COLUMN superseded_by TEXT;  -- references superseding episode_id
```

**Verification:**
- `cargo test -p bitemporal-runtime`
- Semantic memory `as_of` query returns historically correct results
- Supersession creates a new row, does not mutate the prior row

### 2.3 — `quant-governor`: Policy routing for governed compression

**What:** New crate `crates/quant-governor/`. Implements:
- `GovernancePolicy`: declares codec profiles (`raw`, `q8`, `q4`, `turbo`, `fib`), admissibility classes, degradation thresholds
- `evaluate(request, policy)` → `CodecDecision` with `codec`, `exact_fallback`, `degradation_budget`, `receipt`
- `ExactFallbackReceipt`: `{ raw_digest, compressed_digest, fallback_retention: bool }`
- `DegradationReceipt`: `{ degradation_type, degraded_by, bytes_saved, accuracy_impact }`
- Integration points for `turbo-quant` and `fib-quant` codec profiles
- Policy-driven routing: which codec is selected based on content type, size, accuracy requirements

**Verification:**
- `cargo test -p quant-governor`
- `codec_decision.raw == true` when content is small enough
- `codec_decision.exact_fallback == true` when degradation budget allows compression

### 2.4 — `claim-ledger`: Claim/evidence adjudication

**What:** New crate `crates/claim-ledger/`. Implements:
- `Claim`, `Evidence`, `AdjudicationResult`, `ContradictionRecord`
- `ClaimLedger` struct: stores claims with `claim_id`, `content_digest`, `provenance`, `status`
- `Evidence::bind(claim_id, evidence_id)` — links evidence to claim
- `adjudicate(claim_id)` — runs the claim through the adjudication policy
- Structured claim boundaries and public-safe language filtering

**Verification:** `cargo test -p claim-ledger`. All 5 evidence families have a roundtrip test.

### 2.5 — `receipt-bench`: Replayable benchmark substrate

**What:** New crate `crates/receipt-bench/`. Implements:
- Local-first replayable benchmark harness for memory, provenance, policy, compression
- `BenchmarkSuite`: defines benchmark scenarios (semantic search, compression round-trip, memory lookups)
- `BenchmarkReceipt`: timestamped results keyed to commit hash and machine fingerprint
- Reporting: diff between runs across commits

**Verification:** Runs locally without network. Produces machine-reproducible results on the same commit.

### 2.6 — `agent-guard`: Linux control plane

**What:** New crate `crates/agent-guard/`. Implements (in approximate order):
- `ControlPlane` trait
- Linux-first: BPF LSM, cgroup v2, Landlock, seccomp, eBPF probes
- `AgentGuard` struct: observes agent activity, emits `SecurityReceipt`
- MCP broker compatible: hooks into `llm-tool-runtime` as approval handler

**Verification:** `cargo test -p agent-guard`. Ships with a `linux-only` cfg gate so it compiles cleanly on non-Linux (with zero functionality).

### 2.7 — `scr-runtime-compression`: Adapter for runtime integration

**What:** New crate `crates/scr-runtime-compression/`. Implements:
- Adapter binding `quant-governor` decisions into `semantic-memory` runtime
- `CompressedSearchPath`: wraps the normal search path, applies compression before encoding
- `ExactFallbackAdapter`: retrieves and decompresses exact fallback on decode
- Never owns codec truth — explicitly delegates to `turbo-quant` or `fib-quant`

**Verification:** Semantic-memory search through compressed path returns identical results to uncompressed path, within degradation budget.

### 2.8 — `quant-eval`: Benchmark suite for compression

**What:** New crate `crates/quant-eval/`. Implements:
- `CompressionBenchmark`: target corpora, accuracy metrics (cosine similarity, recall@K, MRR)
- `SemanticMemoryBenchmark`: search quality over compressed vs. raw
- `AdmissibilityTest`: run standard test sets through each codec at each profile
- Results output as `BenchmarkReceipt` compatible with `receipt-bench`

**Verification:** All benchmarks pass locally. Results are reproducible and content-addressed.

---

## Phase 3 — Doctrinal Compliance for Existing Crates (P1 — 1 week)

### 3.1 — HNSW shadow-truth marking

**Fix:** Add `approximate: bool` to `VectorSearchReceiptV1`. Set to `true` when HNSW index was used. Add `ApproximateSearchReceipt` explicit type.

### 3.2 — Episode supersession

**Fix:** `semantic-memory` episode updates create a supersession record instead of mutating in place. `update_outcome` becomes an append-only supersession event.

### 3.3 — Turbo-quant wiring into semantic-memory search path

**Fix:** `semantic-memory` feature flag `turbo-quant-codec` must, when enabled, route embeddings through `scr-runtime-compression`. An `IntegrationReceipt` is emitted for every compressed encode/decode round-trip.

### 3.4 — Fib-quant wiring into semantic-memory (same as 3.3)

### 3.5 — `fibquant` KV receipts wired

**Fix:** `fib-quant/receipt.rs` KV receipts integrated into the codec execution path. Codec execution emits `CodecReceipt` with raw digest, compressed digest, exact fallback flag, degradation disclosure.

### 3.6 — PolyKV integration into workspace

**Fix:** `poly-kv/Cargo.toml` merges `rust-version = 1.78` → `1.75`. Add `poly-kv` to root workspace members. `poly-kv` adopts `boundary-compiler` for canonicalization and `bitemporal-runtime` for temporal semantics.

### 3.7 — LLM pipeline execution receipts

**Fix:** `llm-pipeline` emits `ExecutionTrace`, `RetryDecision`, `ProviderRouteReceipt`, `BudgetDebit`, `ResponseReceipt` on every pipeline run. All receipt types are in `stack-ids` or equivalent shared crate.

### 3.8 — Governance crates emit receipts

**Fix:** All 7 governance crates (assurance-runtime, attestation-exchange, authority-delegation, constitutional-memory, continuity-runtime, effect-runtime, mechanism-runtime) implement at minimum:
- `ArtifactEnvelope` with `content_digest`, `recorded_time`, `artifact_type`
- One test per artifact type that the crate declares
- No crate ships a type that isn't tested in a roundtrip

### 3.9 — Turbo-quant memory accounting

**Fix:** `turbo-quant` and `fib-quant` track pool bytes, metadata bytes, decoded bytes, per-reader bytes per `polykv research.md` D1. `AccountingReceipt` emitted on every encode/decode operation.

### 3.10 — `tauri-queue` resolved or removed

**Fix:** Either implement `tauri-queue` fully (it is a stub) or exclude it from workspace with documented rationale.

---

## Phase 4 — Verification & Publication Readiness (P1 — 2–3 days)

### 4.1 — Full test suite run

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace -- --nocapture  # verbose for failures
```

Any crate with < 0.20 test ratio gets a blocker issue in `02_MASTER_ISSUE_MATRIX.md`.

### 4.2 — Benchmark run under release profile

```bash
cargo bench --workspace --release 2>&1 | tee evidence/release_profile_benchmarks.json
```

Baseline results for `semantic-memory` (search), `turbo-quant` (encode/decode), `fib-quant` (encode/decode), `llm-pipeline` (end-to-end latency).

### 4.3 — Performance baselines and scaling curves

Each codec crate (turbo-quant, fib-quant) produces:
- `benchmark_receipt.json` for raw/q8/turbo/fib at 1K/10K/100K/1M token inputs
- Scaling curves for dominant hot-path operations
- Memory usage curves under compression

### 4.4 — Doctrinal conformance self-attestation

Each crate emits a `ConformanceAttestationV1` with:
- Artifact families it emits (with receipt types)
- Bitemporal compliance status
- Canonicalization method
- Safety invariants (unsafe count = 0, panic count = 0)

### 4.5 — Reproduction bundle for external auditors

Ship a `reproduce.sh` harness that:
- Checks out the current commit
- Runs the full test suite
- Runs benchmarks
- Collects receipts into a `RECEIPT_BUNDLE.json`
- External reviewers can replay without reconstructing the toolchain

---

## Phase 5 — Final Audit & Closeout (P0 — 1 day)

### 5.1 — Run V31 hostile audit

Delegate to a fresh subagent: perform a hostile audit of the V30 canonical state, using the doctrine lens. All P0 findings from V29/V30 must be either fixed or formally downgraded/waived with documented rationale.

### 5.2 — Closeout receipt

Produce a final `CLOSE_V30_RECEIPT.md` containing:
- All P0/P1/P2 issues resolved vs. downgraded
- Test coverage ratios by tier
- Benchmark results
- Receipt bundle (SHA256 of each canonical artifact)
- Reproducibility proof

### 5.3 — Promote to canonical

Tag in git, update `Cargo.toml` version, write release notes in `CHANGELOG.md`, update the doctrine docs if any new semantic law was discovered.

---

## Summary of Required Deliverables

| Deliverable | Phase | Ownership |
|---|---|---|
| Clean git commit | 0.1 | All |
| Restored issue matrix | 0.2 | All |
| `boundary-compiler` crate | 2.1 | New |
| `bitemporal-runtime` crate | 2.2 | New |
| `quant-governor` crate | 2.3 | New |
| `claim-ledger` crate | 2.4 | New |
| `receipt-bench` crate | 2.5 | New |
| `agent-guard` crate | 2.6 | New |
| `scr-runtime-compression` crate | 2.7 | New |
| `quant-eval` crate | 2.8 | New |
| Primitives tests (10 crates) | 1.3 | Existing |
| `knowledge-runtime`, `kernel-oracles` panic removal | 1.2 | Existing |
| `check-runner` unsafe resolved | 1.1 | Existing |
| HNSW approximate marking | 3.1 | `semantic-memory` |
| Episode supersession | 3.2 | `semantic-memory` |
| Codec wiring | 3.3–3.5 | `semantic-memory`, `fib-quant`, `poly-kv` |
| PolyKV workspace merge | 3.6 | `poly-kv` |
| Governance receipts | 3.8 | 7 governance crates |
| Full benchmark suite | 4.2 | All |
| Reproduction bundle | 4.5 | All |
| V31 hostile audit | 5.1 | All |
| Closeout receipt | 5.2 | All |

---

## Hard Rules for All Implementors

1. **No `unwrap()` in production code.** Use `thiserror` or `anyhow` with typed variants.
2. **No `unsafe` in workspace member crates** unless formally reviewed and exempted via a scoped `[lints]` override with documentation.
3. **Every new receipt type requires a roundtrip test** — serialize → deserialize → assert fields match.
4. **Every new crate requires** `cargo test -p <crate>`, `cargo doc --no-deps`, and a README explaining the doctrinal basis.
5. **Do not fork canonical semantics** into application crates. Gloss/AiDENs must consume canonical crates, not reimplement them.
6. **Digest linking is mandatory** for all artifact families. No artifact enters the system without a `content_digest`.
7. **Bitemporal mutations use append-plus-supersession.** No `UPDATE` in place for truth-bearing state.
8. **All CLI tooling must belong to a CLI crate**, not live as loose scripts in `scripts/`.

---

## Timeline Reference

| Phase | Duration | Focus |
|---|---|---|
| Phase 0 | 1–2 hrs | Git hygiene, recovery |
| Phase 1 | 1 week | Workspace integrity, tests, panic fixes |
| Phase 2 | 1–2 weeks | 8 new canonical crates |
| Phase 3 | 1 week | Doctrinal compliance for existing crates |
| Phase 4 | 2–3 days | Verification and benchmarks |
| Phase 5 | 1 day | Hostile audit + closeout |

**Total estimated time:** 5–6 weeks with AI assist. Phase 2 (new crates) is the longest singly. Phase 0 (git hygiene) is the most urgent.
