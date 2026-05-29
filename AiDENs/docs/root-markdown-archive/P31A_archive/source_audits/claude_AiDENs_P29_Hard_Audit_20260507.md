# AiDENs Hard Audit — P29 / 2026-05-07

**Auditor:** Claude Sonnet 4.6  
**Source package:** `AiDENs-aidens-next-codex-context-20260507.zip`  
**Package certifier:** v2026.05.07-p29, 0 errors, 0 warnings  
**Audit scope:** Full codebase review — architecture, evidence discipline, code quality, runtime risk, release status, and research corpus

---

## Executive Summary

AiDENs is a Rust-based local agent orchestration runtime now at its 29th codex execution pass. The project has grown into a sophisticated, well-documented system with genuine depth — a real multi-crate Rust workspace, an extensive evidence and audit pipeline, and a serious theoretical research corpus backing the design decisions. The documentation discipline is among the best I've seen in a solo or small-team project: mandatory phase reports, manual gates, forbidden-state checks, and a self-certifying package zipper all combine into something meaningfully rigorous.

That said, P29 ends in a formally incomplete state. The pass completed all 22 phases and passed all pre-package command checks, but stopped at a mandatory operator gate (Injection 6) before final package generation. The `injection6_package_self_replay` validation is `blocked_missing_package`, and `final_labels` remains empty. The current support tier is `candidate-pending-final-package` with `semantic_status: degraded_exact_check`. This is a controlled stop by design, not a crash — but it means the package you are shipping is a codex-context snapshot, not a finalized release artifact.

The open bug backlog remains substantial: 200 confirmed bugs from the prior Claude audit, plus an estimated 100–300 in unaudited components. P29 absorbed the highest-priority items, but the long tail is real.

**Overall verdict:** Strong foundations, honest scoping, unfinished release gate. The right next action is Injection 6 → final package generation → self-replay verification → label grant. P30 should focus on known-limitations fill and quarantine closure.

---

## 1. Release Status

### 1.1 Current Posture

| Field | Value |
|---|---|
| Current run | P29 |
| Support tier | `candidate-pending-final-package` |
| Semantic status | `degraded_exact_check` |
| Final labels | *(none granted)* |
| Validation failures | 1 — `injection6_package_self_replay: blocked_missing_package` |

P29 ran 22 phases. All phase reports are present in `handoffs/p29/`. Manual gates fired at Phases 03, 07, 11, 15, 19, and 21. Phase 21 explicitly stops before final package generation pending Injection 6 — this is correct procedure, not a bug.

### 1.2 What Is Blocked

The following evidence cannot be finalized until Injection 6 is provided and the final package is generated:

- `target/p29/package/AiDENs-p29-codex-context.zip`
- Package sidecars (report, manifest, findings, excluded)
- `python3 scripts/assert_p29_package_self_replay.py` — the mandatory final gate
- `docs/p29/P29_FINAL_AUDIT_REPORT.md`
- `docs/p29/P29_KNOWN_LIMITATIONS_REGISTER.md`
- `handoffs/p29/FINAL_AUDITOR_HANDOFF.md` (marked incomplete)
- `P29_KNOWN_LIMITATIONS_TEMPLATE.md` — all sections empty (template only)

**Finding F-001 [CRITICAL]:** Do not grant any P29 release labels until the package self-replay gate passes and the known-limitations register is populated. The current codex-context zip is a valid snapshot for LLM consumption; it is not the release artifact.

### 1.3 Forbidden Final States — Status

All 12 forbidden states are documented and most are cleanly avoided. The notable risk is item 12: the Phase 21 pre-package manual gate report exists, but the five mandatory post-Injection-6 reports do not. If the final package is generated in a separate session without the operator enforcing these, the forbidden state "evidence manifest references missing files without explicit external/degraded labels" could be violated. The P29 verifier discipline is designed to catch this — ensure it runs after package generation.

---

## 2. Evidence and Documentation Discipline

This is the project's greatest strength. The evidence pipeline is genuinely impressive for a non-enterprise codebase.

**Strengths:**

- **Phase-gated evidence discipline.** Every phase has a required `PHASE_XX_REPORT.md` with: files changed, tests run, issue IDs addressed, evidence artifacts, unresolved risks, and pass/fail gate status. All 22 P29 phase reports are present and structured.
- **Forbidden-state law.** Twelve explicitly enumerated forbidden final states with clear criteria. This is rare and valuable — most projects only define what they want, not what they forbid.
- **Certifier tooling.** `zip.py` runs in strict mode and produces manifest, report, findings, and excluded sidecars. Zero findings on this package.
- **Canonical ownership map.** The `CANONICAL_OWNER_MAP.md` and `CANONICAL_SOURCE_OF_TRUTH.md` documents explicitly assign truth ownership to sibling crates (semantic-memory, knowledge-runtime, etc.) rather than absorbing it into AiDENs. This is architecturally honest.
- **P28 failure postmortem.** The fact that P28's failures are documented, root-caused, and explicitly corrected in P29's design shows mature failure-handling culture.
- **Release law enforcement.** "No package, no claim. No extracted-package verifier, no release." This is stated as law and the gates enforce it.

**Weaknesses:**

**Finding F-002 [MEDIUM]:** `P29_KNOWN_LIMITATIONS_REGISTER.md` is a blank template at package time. This document is supposed to enumerate remaining P0/P1 issues, quarantined surfaces, unaudited surfaces, v11A limitations, v11B seed limitations, and v11C reserved-only limitations. Its absence means the auditor handoff is incomplete even if the package self-replay were to pass. This must be filled before any label grant.

**Finding F-003 [LOW]:** The certifier report flags 128 root Markdown files as "ambiguous" (neither protected nor archive candidates). This creates noise in the archive management pipeline over time. A future pass should resolve ambiguous classifications or explicitly mark them protected/candidate.

**Finding F-004 [LOW]:** 62 stale codex artifacts were excluded with reason `stale-codex-artifact-disabled`. The codex archive shows `active_stale_after: []`, which is correct, but a growing stale archive is overhead. Consider a periodic purge or promotion pass.

---

## 3. Architecture

### 3.1 Crate Structure

The workspace is well-decomposed into 33 Rust crates with a clear layering:

```
aidens-contracts          ← schema/type definitions, no runtime logic
aidens-runner             ← plan-act-verify loop, orchestration
aidens-*-kit              ← domain modules (agency, boundary, budget,
                             capability, daemon, delegation, governance,
                             memory, permit, plan, provider, queue,
                             receipts, repair, schedule, security,
                             tool, wake)
aidens-profile-*          ← deployment-profile configurations
aidens-cli                ← CLI entrypoint
aidens-integration-tests  ← cross-crate test suite
aidens-testkit            ← shared test fixtures
```

The sibling crates (semantic-memory, knowledge-runtime, living-memory, forge-pilot, etc.) are consumed as external Cargo path dependencies and are not owned by AiDENs. This boundary is well-maintained.

**Strength:** The contracts crate is clean — it defines types, not behavior, and is the correct hub for execution envelopes, artifact IDs, provider routes, and boundary policies. The runner's modular split into `execution`, `finalization`, `provider_tool`, `receipts`, and `replay` submodules is clean and testable.

### 3.2 Canonical Ownership Boundary

AiDENs correctly claims ownership of: local operator/orchestration/display/package/runtime surface. It explicitly does NOT claim: canonical memory truth, canonical governance truth, canonical kernel truth, canonical provider/tool contract ownership, or canonical schema-generation ownership.

**Finding F-005 [MEDIUM]:** The boundary is articulated in policy docs but enforcement is discipline-dependent. With 33+ crates and sibling crates touching the same domain, boundary drift is a latent risk. The P20-era `CONTRACT_OWNERSHIP_INVENTORY.md` and scanner (`scripts/p20_scan_aidens.py`) exist to catch this, but there is no evidence these are run in P29 gates. Recommend adding a boundary-scan assertion to `p29_verify.sh` or a P30 acceptance gate.

### 3.3 v11A / v11B / v11C Scope

- **v11A:** Local release-candidate. Evidence scope is limited to the `run-coding-agent` path. Material operations, execution context envelopes, receipts, proof/debt/waiver semantics, boundary compiler profiles, and semantic/view disclosure are all targeted. Evidence is present for supported-local path only — no cloud, no broad autonomy.
- **v11B:** Executable seed. Right-graph misuse tests, region contract seed, convergence/residual/syndrome seed, and lawful subtraction seed exist. This is not a completion claim.
- **v11C:** Reserved-only. Correctly contained.

This scope discipline is exactly right. The explicit forbidden labels (`v11B-complete`, `v11C-complete`, `production-cloud-ready`, `broad-autonomy-ready`) are the appropriate guardrails.

---

## 4. Code Quality

### 4.1 Search Layer (semantic-memory/src/search.rs)

**Quality: Good.** The hybrid search engine (BM25 + vector cosine + RRF) is well-structured.

- `sanitize_fts_query` uses an allowlist strategy — correct approach. It strips FTS5 boolean keywords and quotes remaining tokens. The double-quote escaping (`replace('"', '""')`) is correct for SQLite FTS5.
- Cosine similarity implementation handles the zero-norm edge case cleanly.
- Recency contribution uses exponential decay with configurable half-life and RRF rank integration — mathematically sound.
- `source_dedup_key` discriminates on all five source types with stable integer prefixes — avoids cross-type collisions.

**Finding F-006 [MEDIUM]:** The `VECTOR_SCAN_WARN_THRESHOLD` of 50,000 rows is a warning, not a hard block. At scale, a full vector scan above this threshold will cause significant latency. Consider making this configurable and adding a hard circuit-breaker that refuses the scan (or forces approximate-only mode) above a higher threshold.

**Finding F-007 [LOW]:** `days_since` uses `NaiveDateTime::parse_from_str` with a single format string (`"%Y-%m-%d %H:%M:%S"`). If any timestamp in the database was stored in a different format (RFC3339, or with fractional seconds), this silently returns `None`, causing recency contribution to be dropped. The failure is silent — no log, no metric. Add a format fallback or emit a warning when parsing fails.

### 4.2 SQLite Pool (semantic-memory/src/pool.rs)

**Quality: Solid.** The single-writer + bounded-reader design with WAL mode is the correct approach for rusqlite. The RAII `ReaderGuard` correctly returns slots on drop, including panic paths. The `Condvar`-based blocking acquisition is appropriate.

**Finding F-008 [HIGH]:** The reader acquisition timeout (30 seconds default) returns an error on timeout. If callers do not handle this error path gracefully (e.g., retry with backoff vs. surface to user), a pool exhaustion scenario under load will produce confusing failure modes rather than degraded-mode operation. Audit all call sites that use `acquire_reader` for error handling completeness.

**Finding F-009 [MEDIUM]:** `unwrap_or_else(|e| e.into_inner())` on poisoned Mutexes silently recovers from lock poisoning. This is a pragmatic choice, but means a panic in a writer or reader closure could leave the pool in a state where subsequent operations see inconsistent data. Document the recovery contract explicitly or gate on a health flag after poison recovery.

### 4.3 HNSW Index (semantic-memory/src/hnsw.rs)

**Quality: Moderate — this is the highest-risk module.** The architecture (HNSW as a recoverable acceleration sidecar over SQLite as source of truth) is the right design. The implementation has structural risks consistent with the BUG-001 through BUG-010 family.

**Finding F-010 [HIGH — consistent with BUG family 001-010]:** `HnswIndexInner` holds three separate RwLocks: `key_to_id`, `id_to_key`, and `deleted_ids`. Any operation that needs to update all three (insert, delete, load) must acquire them in a consistent order. A TOCTOU window exists between reading `deleted_ids` to filter search results and reading `id_to_key` to resolve keys — a concurrent delete can produce a key lookup for an ID that is being removed. The absence of a single holding lock over all three maps during mutation is the root cause of the known HNSW deadlock and TOCTOU bug family.

**Finding F-011 [HIGH]:** `keymap_dirty: AtomicBool` and `last_flush_epoch: AtomicU64` are used for flush coordination, but `AtomicBool` alone cannot guarantee that the key maps and the dirty flag are consistent without a sequentially-consistent memory fence at both the write and read sites. If `Ordering::Relaxed` is used anywhere in the flush path, the dirty flag may be visible before the key map writes are.

**Finding F-012 [MEDIUM]:** The `next_id: AtomicUsize` counter is a monotonic allocator. Deleted IDs accumulate in `deleted_ids` but are never recycled. At `max_elements: 100_000` default this is bounded, but a long-running daemon with frequent insert/delete cycles will exhaust the ID space before approaching the HNSW capacity limit. Document the ID exhaustion behavior or add recycling.

### 4.4 Execution Contracts (aidens-contracts/src/execution.rs)

**Quality: Good.** The `ExecutionContextEnvelopeV1` is comprehensive and well-typed. The `local_started` constructor correctly computes `environment_fingerprint` at construction time. The `replay_handle`, `redaction_state`, `degradation_refs`, and `reason_codes` fields show forward-thinking evidence design.

**Finding F-013 [MEDIUM]:** `budget_millis_consumed` is set at construction to 0. If a caller fails to update it before persisting a receipt, all receipts will show zero consumption — making budget analysis useless. There is no enforcement that `budget_millis_consumed` is updated before `completion_state` is set to a terminal state. Add a builder-style `complete()` method that requires consumed budget as a parameter.

### 4.5 Runner (aidens-runner/src/lib.rs)

**Quality: Good structural decomposition.** The `PlanActVerifyLoopV1` separates execution, finalization, provider interaction, receipt management, and replay into distinct modules. The import surface from sibling kits is broad but appropriate for an orchestration layer.

**Finding F-014 [MEDIUM]:** `provider_mock_response: Option<String>` suggests the mock path is a simple string substitution. If mock responses need to simulate tool calls, multi-turn flows, or error conditions, a simple string mock will drive test divergence from real provider behavior over time. The mock should be typed to match the provider response schema.

---

## 5. Bug Backlog Assessment

The P29 manifest records a large open bug list spanning BUG-016 through BUG-200. The P29 Claude Audit Absorption doc acknowledges 200 confirmed bugs and estimates 100–300 more in unaudited components.

### Bug Family Status

| Family | IDs | Status in P29 |
|---|---|---|
| HNSW integrity/concurrency | BUG-001–010 | Targeted Phase 05 — evidence in phase report |
| SQLite/migration/atomicity | BUG-011–020, 076–085 | Targeted Phase 06 |
| Search/ranking/dedup | BUG-021–030, 053–059 | Targeted Phase 07 |
| Quantization/vector disclosure | BUG-031–034 | Targeted Phase 08 |
| Pool/concurrency/reembed/drop | BUG-035–042 | Targeted Phase 09 |
| Graph/chunker/knowledge-runtime | BUG-043–059, 086–100 | Targeted Phase 10 |
| Stack IDs / contracts / living-memory | BUG-060–075, 130–149 | Targeted Phase 11 |
| Unaudited high-risk layers | BUG-190–200 | Quarantine planned Phase 04 |

**Finding F-015 [HIGH]:** The manifest open_bugs list contains a very large number of IDs (BUG-016, 017, 019, 020, 024, 025, 027–030, 057, 076–080, 086–105, 114–123, 131–144, 147, 151–180, 183, 190–200). The presence of these IDs in `open_bugs` after a 22-phase pass targeting them is ambiguous — it may mean "addressed but not fully closed" or "quarantined" rather than "still open." The manifest needs a clearer three-way status: `fixed`, `quarantined`, `deferred` — not a flat list.

**Finding F-016 [CRITICAL]:** The fully unaudited surface includes forge-pilot, effect-runtime, verification pipeline (adjudication, calibration, control, policy), federation, attestation, authority-delegation, and recursive-kernel-core. These are present in the zip (collectively hundreds of files) and are consumed by AiDENs operations. BUG-190 through BUG-200 are placeholders for "unknown unknowns" in these layers. Until they are audited, any claim about system correctness is bounded by the worst-case behavior of these components.

---

## 6. Research Corpus Assessment

The `Full_Provenance__Research_4_26_26.zip` contains 57 research documents spanning:

- Bitemporal replay and append-only history (Temporal Technologies precedent)
- Lawful execution and durable workflow semantics
- W3C PROV model and OpenTelemetry span links for provenance
- OPA decision logs as audit artifacts
- Algebraic semantics, hypergraph decoders, multiscale inference
- Contract hardening, temporal truth, episode identity

This is an unusually strong theoretical foundation. The research correctly identifies the right external precedents: Temporal for replay history, OPA for decision-log audit artifacts, OpenTelemetry for async span correlation (span links > parent-child for fan-in/fan-out scenarios), and Kubernetes admission webhooks for structured allow/deny at control-plane edges.

The `ArbiterDecisionV1` schema proposed in `recall-research.md` is well-designed — it captures route selected, provider directive, signals with evidence refs, candidate ranking, and fallback ladder. This directly informs the v11A turn-arbiter design.

**Finding F-017 [LOW]:** The research documents reference external citations in the format `citeturn10view0` — these appear to be artifacts of a prior deep-research tool session and are not resolved links. The research content itself is valuable but the citation format is not durable for long-term reference. Consider converting key citations to standard bibliography entries.

---

## 7. Operational Risks

**Finding F-018 [HIGH]:** The v11A local release-candidate claim covers the `run-coding-agent` path only. The runner imports from agency-kit, boundary-kit, budget-kit, governance-kit, memory-kit, permit-kit, provider-kit, receipts, and tool-kit. If any of these kits has an unquarantined BUG-family issue, the v11A path is affected even though it is the "supported" path. The v11A evidence should explicitly enumerate which bug IDs were confirmed not to affect the supported execution path.

**Finding F-019 [MEDIUM]:** The `PlanActVerifyLoopV1` holds `provider_mock_response` as a plain string and `Arc<Mutex<...>>` wrappers elsewhere. The mix of ownership strategies (Arc, Mutex, plain clone) across the runner loop suggests the concurrency model was evolved incrementally. A future pass should audit the runner for missed lock acquisition on shared state.

**Finding F-020 [LOW]:** The certifier excludes `target/` output directories, which is correct. However, all audit log evidence is classified as `external:target/p29/audit/...`. This means the only way to verify audit log claims is to trust the agent that generated the phase reports. For a higher assurance bar, the audit logs should be hashed and the hash committed to the phase report at the time of generation.

---

## 8. What Is Genuinely Good

It is important to be clear about what works, not just what needs fixing.

The pass-per-pass evidence model — with its mandatory handoffs, forbidden states, and certifier gates — is genuinely mature. Most projects at this stage have none of this. The fact that P28's failure was fully postmortemed and its specific failure modes (stale classifier trust, missing verifier, evidence manifest pointing to absent files) directly drove P29's design is exactly how engineering discipline should work.

The SQLite-as-source-of-truth / HNSW-as-recoverable-sidecar architecture is the right call. It means HNSW corruption is recoverable, not fatal. The pool design is sound. The search layer is solid.

The contracts crate as a pure type/schema layer (not behavior) is the right dependency inversion. The runner consuming it cleanly keeps the domain logic separated from orchestration.

The explicit v11A/v11B/v11C scope ladder, with forbidden completion claims for v11B and v11C, shows honest release management. The research corpus driving the design is sophisticated and well-grounded in real precedents.

---

## 9. Recommended Next Actions

Listed in priority order.

### Immediate (before any P29 label grant)

1. **Provide Injection 6.** Generate the final package, run `assert_p29_package_self_replay.py`, and confirm it passes.
2. **Populate `P29_KNOWN_LIMITATIONS_REGISTER.md`.** Fill all sections before granting any label. This is the auditor's primary reference for what is and is not supported.
3. **Complete `FINAL_AUDITOR_HANDOFF.md`.** Add package sidecars, self-replay result, known limitations, and unresolved risks.
4. **Grant labels only if all six acceptance gates pass.** Do not grant `v11A-local-release-candidate` without explicit material-operation receipts from the supported-local path.

### P30 Focus Areas

5. **Clarify open_bugs status.** Replace the flat open_bugs list with a three-way `fixed / quarantined / deferred` classification in the status manifest schema.
6. **Fix HNSW RwLock ordering (F-010, F-011).** This is the highest-severity unresolved code issue. Either introduce a single coarse lock over all three maps during mutation, or implement a lock-free alternative.
7. **Audit `acquire_reader` call sites (F-008).** Ensure all callers handle timeout errors with appropriate backoff or degraded-mode behavior.
8. **Add boundary-scan to verify gate (F-005).** Run the ownership scanner as part of `p29_verify.sh` (or its P30 successor) to catch canonical-boundary drift early.
9. **Audit the unaudited layers (F-016).** forge-pilot, effect-runtime, verification pipeline, and federation collectively represent the largest unknown risk surface. Schedule a dedicated audit pass.
10. **Fix `budget_millis_consumed` enforcement (F-013).** Receipts with zero consumed budget make post-hoc analysis misleading.

---

## 10. Audit Findings Summary

| ID | Severity | Area | Summary |
|---|---|---|---|
| F-001 | CRITICAL | Release | No labels before package self-replay passes |
| F-002 | MEDIUM | Evidence | Known limitations register is blank template |
| F-003 | LOW | Evidence | 128 ambiguous root Markdown files unresolved |
| F-004 | LOW | Evidence | 62 stale codex artifacts accumulating |
| F-005 | MEDIUM | Architecture | Canonical boundary not enforced in verify gate |
| F-006 | MEDIUM | Search | Vector scan threshold is warn-only, no hard block |
| F-007 | LOW | Search | `days_since` timestamp parse failure is silent |
| F-008 | HIGH | Pool | Reader timeout error handling not audited at call sites |
| F-009 | MEDIUM | Pool | Silent Mutex poison recovery lacks health-state documentation |
| F-010 | HIGH | HNSW | Separate RwLocks on key maps create TOCTOU window |
| F-011 | HIGH | HNSW | Atomic dirty flag may lack sequentially-consistent ordering |
| F-012 | MEDIUM | HNSW | next_id counter never recycles deleted IDs |
| F-013 | MEDIUM | Contracts | budget_millis_consumed not enforced before terminal state |
| F-014 | MEDIUM | Runner | Mock response is untyped string, not provider-schema-typed |
| F-015 | HIGH | Bugs | open_bugs list status is ambiguous (fixed vs quarantined vs deferred) |
| F-016 | CRITICAL | Bugs | Unaudited high-risk layers (forge-pilot, effect-runtime, verification pipeline, federation) |
| F-017 | LOW | Research | External citations in research docs are unresolved tool artifacts |
| F-018 | HIGH | v11A | v11A path does not enumerate which bug IDs are confirmed not to affect it |
| F-019 | MEDIUM | Runner | Mixed ownership strategies in runner suggest incremental concurrency model |
| F-020 | LOW | Evidence | Audit logs are not hashed at generation time |

**Totals:** 2 CRITICAL · 5 HIGH · 8 MEDIUM · 5 LOW

---

*Audit completed 2026-05-07. Source package SHA-256: `52bf3410b41e22f42073344c5137113d092b437d5cb4eb6a9a0233b57a0d3f46`. This audit covers the codex-context snapshot only — build/test/audit logs referenced as `external:target/...` were not verified.*
