# Libraries hard audit — Claude Opus 4.6 — 2026-03-30

## Source basis

- **Grounding:** direct code inspection of `libraries-source-clean-20260330.zip` within the Recall workspace
- **Scope:** `deps/llm-pipeline` (9,084 src LOC), `deps/llm-output-parser` (3,425 src LOC), plus all workspace dependency declarations and their interaction surfaces
- **Compile-confirmed:** No (no Rust toolchain in sandbox)
- **Note:** The 30-crate parent library workspace was NOT in this upload. This audit covers the vendored library surface visible inside the Recall workspace — specifically `llm-pipeline`, `llm-output-parser`, and all 15 path-dependency declarations in `Cargo.toml`. Findings about the broader library stack are inferred from type signatures, import surfaces, and API contracts visible at consumption boundaries.

## Overall score: 8.0 / 10

| Dimension | Score |
|---|---|
| Architecture | 8.8 |
| Type discipline / contracts | 9.0 |
| Runtime closure | 7.2 |
| Governance integration | 6.8 |
| Tool runtime completeness | 8.5 |
| Operational readiness | 7.0 |
| Error handling | 8.2 |
| Performance under constraint | 7.5 |
| Composability / reuse | 9.0 |
| Documentation fidelity | 7.0 |

---

## 10-perspective audit

### Perspective 1: Type system lawyer

**Verdict:** Strong. The stack's type discipline is its primary moat.

**Findings:**

**CLIB-001 — GovernanceReceipt uses stringly-typed disposition instead of enum (P2)**
`recall-session/src/governance.rs` defines `GovernanceReceipt` with `disposition: String` and `scope_decision: String`. But `ScopeDecision` is already a proper enum in `scope_governance.rs`. The receipt flattens typed decisions into strings, breaking exhaustive-match guarantees at consumption boundaries. Every downstream consumer must match on `"allowed"`, `"blocked"`, `"allowed-via-exception"`, `"unenforced"` as raw strings.

Evidence: `governance.rs:22-36`, `scope_governance.rs:10-17`

Fix: Make `disposition` and `scope_decision` enum fields on `GovernanceReceipt`, or newtype wrappers. Serialize as strings for JSON, but keep Rust-side exhaustive matching.

**CLIB-002 — ToolCtx construction via serde_json round-trip is fragile (P2)**
`session.rs:1012-1022` constructs `ToolCtx` by serializing a JSON object and deserializing it. If `llm-tool-runtime` adds a required field to `ToolCtx`, this silently fails at runtime rather than at compile time. A builder or constructor would catch this statically.

Evidence: `session.rs:1012-1022`

Fix: Use a `ToolCtx::builder()` or direct struct construction.

### Perspective 2: Failure semantics adversary

**Verdict:** Good defensive posture, one structural gap.

**CLIB-003 — Constraint compilation output is computed and discarded during ingest (P1)**
`pipeline.rs:180-199` computes `CompileOutput` with `graph_hash`, `degradation_markers`, and `invalidation_cones` on every ingestion. It logs warnings for degradations and invalidation cones, then drops the result. The session holds `compiled: Arc<Mutex<Option<CompileOutput>>>` but it's only populated through the counterfactual tool, not through the ingest pipeline. The constraint graph doesn't accumulate.

Evidence: `pipeline.rs:180-199`, `session.rs:427-429`

Fix: Accumulate `CompileOutput` into the session's `compiled` mutex on every ingest, or maintain a persistent constraint graph in `ForgeStore` that the Orient phase can query.

**CLIB-004 — Reranker failure is silent degradation with no receipt signal (P2)**
`session.rs:581-585` applies the reranker if present, otherwise falls through. But the receipt doesn't record whether reranking was applied, skipped, or failed. A query that returns degraded results (BM25-only when reranker was expected) looks identical in the receipt to a properly-reranked query.

Evidence: `session.rs:581-585`, `QueryReceipt` struct

Fix: Add `reranker_status: Option<String>` to `QueryReceipt` — "applied", "not_configured", or "failed:{reason}".

### Perspective 3: Governance skeptic

**Verdict:** Real governance exists but doesn't yet command the OODA loop.

**CLIB-005 — Governance receipt is append-only metadata, not a gate (P1)**
`session.rs:1609-1614` evaluates governance and attaches the receipt, but governance disposition never gates execution. A "blocked" governance receipt still returns results. The query proceeds regardless of what governance says. Governance is observational, not authoritative.

Evidence: `session.rs:1609-1614`, `session.rs:1639-1653`

Fix: If `governance_receipt.disposition == "blocked"` and `strict_scope == true`, the query should return a structured error or empty result set with the governance receipt explaining why, not silently proceed.

**CLIB-006 — Governance exceptions have no temporal validation (P2)**
`scope_governance.rs:54-81` checks `exception_covers()` by matching namespace and domain refs, but doesn't validate `starts_at` / `expires_at` timestamps on the `ProfileExceptionBundleV1`. An expired exception still grants access.

Evidence: `scope_governance.rs:54-81`, `ProfileExceptionBundleV1` fields

Fix: Add temporal validation to `exception_covers()` checking `starts_at <= now <= expires_at`.

### Perspective 4: Tool runtime adversary

**Verdict:** Strong tool surface, one exposure gap.

**CLIB-007 — Tool prompt shows write tools to the model even when no approval handler is set (P1)**
`session.rs:815-876` builds the tool prompt including all 9 tools with their schemas. Write tools are listed as "write (requires approval)". But if `approval_handler` is `None`, write tool invocations will be hard-rejected at dispatch. The model is shown a tool surface it cannot use, wasting context budget on unreachable tools and causing false tool-call attempts.

Evidence: `session.rs:815-876`, `session.rs:1091-1099`

Fix: Filter write tools from the tool prompt when `approval_handler.is_none()`, or add a `(currently unavailable — no approval handler)` suffix.

### Perspective 5: Memory integrity auditor

**Verdict:** Memory path is well-designed, one dedup weakness.

**CLIB-008 — Memory dedup uses re-embedding search as proxy instead of direct hash lookup (P2)**
`session.rs:1168-1175` calls `self.observe()` (full semantic search) to find potential duplicates, then hashes and compares. This means dedup depends on embedding similarity, not content identity. If embeddings change (different provider, dimension change), the same content could fail to deduplicate. A direct content-hash index would be deterministic.

Evidence: `session.rs:1168-1175`, `memory_policy.rs:278-296`

Fix: Add a `content_hash` column to the memory store's SQLite schema and do a direct hash lookup for dedup before falling back to semantic similarity.

### Perspective 6: Performance pessimist

**Verdict:** Reasonable for local-first, one scaling concern.

**CLIB-009 — orient() has no hard character cap independent of budget (P2)**
`session.rs:696-735` builds the orient prompt by iterating context chunks with a character budget derived from `budget.context_max * 4`. But `orient()` also prepends a system identity prompt whose size is uncapped. For large identity prompts or many high-scoring chunks, the combined orient output can exceed what `act()` expects, causing the history budget to be squeezed to zero.

Evidence: `session.rs:696-735`, `session.rs:747-757`

Fix: Have `orient()` accept the total budget and return both the prompt and the remaining budget for history.

### Perspective 7: Provider integration tester

**Verdict:** Multi-provider support is real, one identity gap.

**CLIB-010 — FailoverProvider doesn't record which provider actually served the request (P2)**
`provider.rs:510-547` chains providers and returns the first successful response. The response's `model` field records the model name, but not the provider label. If Ollama and OpenRouter both serve `llama3.1:8b`, the receipt can't distinguish which backend ran.

Evidence: `provider.rs:510-547`, `CompletionResponse` struct

Fix: Add `provider_label: String` to `CompletionResponse` and have `FailoverProvider` set it on successful completion.

### Perspective 8: Concurrency adversary

**Verdict:** Sound use of Mutex/RwLock, one theoretical concern.

**CLIB-011 — Mutex poisoning recovery may silently use stale turn data (P3)**
`lock_or_recover()` in `session.rs:130-135` recovers from poisoned mutexes by calling `into_inner()`. The design rationale (RV3-013) is documented and defensible for a personal journal app. However, if a panic occurs mid-turn-append, the recovered state could have a partially-written turn. The auto-persist would then write this partial state to disk.

Evidence: `session.rs:130-135`

Fix: Consider validating turn integrity after recovery (e.g., checking the last turn has both role and content populated) before persisting.

### Perspective 9: Build system / dependency auditor

**Verdict:** Clean workspace structure, one dependency concern.

**CLIB-012 — 15 sibling path dependencies create a fragile build topology (P1)**
`Cargo.toml:24-39` declared sibling path dependencies outside the uploaded archive. Any CI, external audit, or fresh clone requires the full canonical library structure. This is the dominant reproducibility barrier.

Evidence: `Cargo.toml:24-39`

Fix: Publish workspace crates to a private registry or vendor them as git submodules with pinned revisions. The `deps/` pattern used for `llm-pipeline` and `llm-output-parser` is the right model — extend it to the remaining 15.

### Perspective 10: DARPA/CLARA reviewer

**Verdict:** The technical artifact is strong. The narrative needs tightening.

**CLIB-013 — The ExportEnvelopeV3 carries 35+ empty vector fields (P3)**
`pipeline.rs:48-87` constructs `ExportEnvelopeV3` with 35+ fields set to `vec![]`. These are v14/v15 schema extension points from the broader library stack. For Recall's use case, they're structural dead weight that inflates serialized envelopes and confuses reviewers who see `refuter_suites_v14`, `rollback_decisions_v14`, etc.

Evidence: `pipeline.rs:48-87`

Fix: For CLARA submission, either document which fields are active and which are extension points, or provide a `RecallEnvelope` type alias that hides the unused fields.

**CLIB-014 — Capability spec claims are accurate but not machine-verifiable (P2)**
`recall_full_capability_spec.md` describes the full product thesis accurately. But the claims are prose, not executable assertions. A reviewer can't run a test suite that proves "every important state transition emits a receipt" or "the model never self-authorizes writes."

Evidence: `recall_full_capability_spec.md`, test suite coverage

Fix: Add a `conformance_tests/` directory with one test per spec invariant (I1 through I10), each named after the invariant it proves.

---

## Summary by priority

| Priority | Count | IDs |
|---|---|---|
| P0 | 0 | — |
| P1 | 4 | CLIB-003, CLIB-005, CLIB-007, CLIB-012 |
| P2 | 7 | CLIB-001, CLIB-002, CLIB-004, CLIB-006, CLIB-008, CLIB-009, CLIB-010, CLIB-014 |
| P3 | 2 | CLIB-011, CLIB-013 |

## Bottom line

The library stack visible through the Recall workspace is architecturally mature and type-disciplined. The dominant theme across all 10 perspectives is **observation without authority** — governance, constraints, and reranking all produce typed artifacts that get logged or attached to receipts, but none of them gate execution. The system knows when something is wrong and records it honestly, but doesn't stop. For a personal journal app this is reasonable. For a DARPA submission claiming governance as a differentiator, the gap between "observed" and "commanding" needs to close.
