# Hostile Audit Remediation Plan — 2026-07-13

## Audit summary
- **77 findings**: 19 Critical, 53 High, 5 Medium
- **Audited commit**: `f071c94f7355b53f2a395ecd4d58bccae72023be`
- **Repository**: `RecursiveIntell/Libraries`
- **Scope**: repository truth, provenance, authority/effects, semantic memory, AiDENs, MCP, graph execution, compression, GPU, supply chain

## Severity/dependency-ordered implementation

### Phase 0 — Critical containment (stop-ship fixes)

#### 0.1 BOUND-001 [Critical] — Boundary schema validation is a no-op
- **Files**: `boundary-compiler/src/schema.rs`, `boundary-compiler/src/error.rs`, `boundary-compiler/tests/jcs_tests.rs`
- **Fix**: Make `SchemaValidator::validate()` return `Err(JcsError::SchemaValidation("schema validation unavailable; refusing schema-required admission"))` instead of `Ok(())`. Add a `ValidationUnavailable` error variant. Add tests proving a payload with missing required fields is rejected.
- **Gate**: `cargo test -p boundary-compiler`

#### 0.2 MCP-003 [Critical] — HTTP search fabricates `verified` provenance status
- **Files**: `semantic-memory-mcp/src/http_server.rs`
- **Fix**: Remove hardcoded `verification_status: "verified"` from HTTP search response. Return `unknown` or derive from actual `search_explained` receipt. Add test asserting no hardcoded "verified" string appears.
- **Gate**: `cargo test -p semantic-memory-mcp --features full`

#### 0.3 MCP-001 [Critical] — Mutation endpoints have no authorization
- **Files**: `semantic-memory-mcp/src/http_server.rs`
- **Fix**: The HTTP server already has bearer token auth + loopback checks. Verify maintenance endpoints (`/maintenance/*`) are gated by both auth AND profile (`allows_http_maintenance()`). Verify `/add` returns 503 (already does). Add test: unauthenticated POST to `/maintenance/vacuum` returns 401.
- **Gate**: `cargo test -p semantic-memory-mcp --features full`

#### 0.4 GRAPH-002 [Critical] — Non-interrupt failures reported as complete
- **Files**: `agent-graph/src/engine.rs`, `agent-graph/tests/execute_with_receipt_tests.rs`
- **Fix**: In `execute_with_interrupt()`, map non-interrupt errors to `ExecutionOutcome::InternalError { message }` instead of `Complete`. Add test: injected node error yields `InternalError`, not `Completed`.
- **Gate**: `cargo test -p agent-graph`

#### 0.5 GRAPH-001 [Critical] — Graph execution receipts contain placeholder evidence
- **Files**: `agent-graph/src/receipt.rs`, `agent-graph/src/engine.rs`
- **Fix**: Replace placeholder `input_digest` (literal "graph-root") and `output_digest` (node-count string) with canonical serialized digests of actual input/state. Make `memory_generations` default to empty vec (already is via `skip_serializing_if`). Add test: mutating input changes `input_digest`.
- **Gate**: `cargo test -p agent-graph`

#### 0.6 SEC-001 [Critical] — agent-guard claims enforcement but initializes only a boolean
- **Files**: `agent-guard/src/lib.rs`
- **Fix**: Rename/reclassify the crate as scaffold. Add `#[deprecated(note = "agent-guard is a scaffold — no OS enforcement is implemented")]` on `AgentGuard::new()` and `initialize()`. Update doc comment to say "scaffold: no OS-level enforcement is implemented yet." Add test asserting the deprecation warning.
- **Gate**: `cargo test -p agent-guard`

#### 0.7 AUTH-001 [Critical] — Production governance falls back to synthetic permissive policy
- **Files**: `forge-pilot/src/loop_runner.rs`, `forge-pilot/src/config.rs`
- **Fix**: Replace `PolicySnapshot::permissive()` fallback with `PolicySnapshot::deny()` or observation-only. Add a build-time feature `dev-permissive-governance` that must be explicitly enabled for permissive behavior. Add test: missing governance state blocks execution.
- **Gate**: `cargo test -p forge-pilot`

#### 0.8 AUTH-003 [Critical] — AdvisoryOnly governance does not prevent the action
- **Files**: `forge-pilot/src/loop_runner.rs`
- **Fix**: When `GovernanceGateResult::AdvisoryOnly`, skip execution and record the halt. Only `Authorized(ExecutionPermit)` should allow effect invocation. Add test: AdvisoryOnly yields zero executor calls.
- **Gate**: `cargo test -p forge-pilot`

#### 0.9 AUTO-001 [Critical] + AUTO-004 [Critical] — Autonomous outputs written to canonical memory before evaluation; quarantine doesn't supersede
- **Files**: `AiDENs/crates/aidens-autonomous/src/capture.rs`, `AiDENs/crates/aidens-autonomous/src/loop_driver.rs`
- **Fix**: Write candidates to a quarantine namespace (`autonomous_candidates`) instead of `autonomous`. Only promote to `autonomous` after all gates pass. If rejected, supersede the candidate fact. Add tests: rejected candidate absent from normal search; promotion is atomic.
- **Gate**: `cargo test -p aidens-autonomous`

#### 0.10 AUTO-003 [Critical] — Auditor and contradiction failures increase permissiveness
- **Files**: `AiDENs/crates/aidens-autonomous/src/loop_driver.rs`
- **Fix**: Make auditor/contradiction failures fail-closed (three-valued: pass/fail/unknown). Unknown may quarantine but never promote or repay debt. Add test: network failure at each gate yields zero promotions.
- **Gate**: `cargo test -p aidens-autonomous`

#### 0.11 MEM-002 [Critical] — Query and document embeddings collide in cache
- **Files**: `semantic-memory/src/search.rs`
- **Fix**: Add an `EmbeddingPurpose` discriminator (Query vs Document) to the cache key. Add test: same text as query and document occupy different keys.
- **Gate**: `cargo test -p semantic-memory --features full`

#### 0.12 MEM-006 [Critical] — USearch mutation uses read lock despite non-thread-safe index
- **Files**: `semantic-memory/src/usearch_backend.rs`
- **Fix**: Switch mutation calls from read lock to write lock. Add a safety comment. Add test: concurrent add/search stress shows no race.
- **Gate**: `cargo test -p semantic-memory --features usearch-backend`

#### 0.13 AUTH-006 [Critical] — Side effect occurs before durable receipt persistence
- **Files**: `llm-tool-runtime/src/semantic_memory.rs`, `llm-tool-runtime/tests/effect_dispatch_receipt.rs`
- **Fix**: Add a durable preflight receipt before tool invocation. If persistence fails, abort before effect. Add test: kill process at persistence point, restart reconciles to one effect.
- **Gate**: `cargo test -p llm-tool-runtime`

#### 0.14 AUTH-002 [Critical] — Active governance observation path is fail-open
- **Files**: `forge-pilot/src/observe.rs` (or equivalent observation path)
- **Fix**: Make strict governance the only path to effectful execution. Advisory observation for read-only only. Add test: fault-injecting governance query proves no tool/effect invoked.
- **Gate**: `cargo test -p forge-pilot`

#### 0.15 AUTH-004 [Critical] — Execution permits replayable, cloneable, insufficiently bound
- **Files**: `llm-tool-runtime/src/` (permit types)
- **Fix**: Make permit non-cloneable. Add expiry, nonce, one-shot consumption. Add test: replay fails, expired permit fails.
- **Gate**: `cargo test -p llm-tool-runtime`

#### 0.16 TRUTH-002 [Critical] — CI regenerates proof artifacts without proving committed artifacts are current
- **Files**: `scripts/run_release_gates.py`, `Makefile`
- **Fix**: Add `git diff --exit-code -- STATUS_EVIDENCE_MANIFEST.json release/closeout_receipt_v1.json` after `make gate`. Split verify from publish.
- **Gate**: `make gate && git diff --exit-code`

### Phase 1 — Integrity hardening

- BOUND-002: Replace raw scanner duplicate-key detection with streaming deserializer
- BOUND-003: Add RFC 8785 Appendix B test vectors
- BOUND-004: Enforce profile fields or remove them
- BOUND-005: Unify canonical JSON digest implementations
- BOUND-006: Make ID newtype fields private with validated parsing
- BOUND-007: Make claim-ledger parsing strict (no silent drops)
- BOUND-008: Bind actual values in bitemporal supersession receipts
- BOUND-010: Define `ArtifactEnvelopeV1` trust anchor
- TRUTH-001: Set default branch to `main` (GitHub settings, not code)
- TRUTH-003-010: Repository contract, CI matrix, schema registry, docs

### Phase 2 — Memory and effects

- MEM-001: Wire USearch backend into MemoryStore default
- MEM-003: Validate batch before cache population
- MEM-004: Include all behavior-changing inputs in cache key
- MEM-005: Fix USearch key collision detection
- MEM-007: Make USearch updates and maps atomic
- MEM-008: Generation-atomic USearch persistence
- MEM-009: Mutually exclusive backend features
- MEM-010: Complete configuration validation
- MEM-011: Reduce crate-level lint suppression
- MEM-012: Fix UTF-8 byte-index slicing
- MEM-013: Persist MCP fact admission metadata
- MEM-014: Enforce append/supersede uniformly
- AUTH-005-010: Tool authorization, receipts, rollback, control data separation

### Phase 3 — Research and verification

- QUANT-001-008: GPU codec validation, benchmark evidence, domain separation
- SEC-002-006: Supply chain, action pinning, SBOM, fuzz/Loom/Kani/Miri, platform coverage
- GRAPH-003-005: Checkpoint errors, graph identity, replay contract
- AUTO-005-008: Durable cycle receipts, source-span binding, shadow mode, status docs
- MCP-002, MCP-004-006: HTTP parser hardening, profile authorization, sensitivity, tool integrity

### Claim boundaries

- No claim of "all 77 findings fixed" unless every gate passes with receipts
- Findings requiring GitHub settings changes (TRUTH-001) are documented as operator actions
- Findings requiring CI infrastructure (TRUTH-006, SEC-003-005) are documented as CI config changes
- Findings requiring external tools (fuzz, Kani, Miri, Loom) are documented as lane setup