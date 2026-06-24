# AiDENs Full Integration Plan — Wire Every Library Capability

**Date:** 2026-06-19
**Goal:** Make AiDENs the single framework that makes all Libraries capabilities accessible for agent creation.
**Success metric:** An agent built with AiDENs can search memory, reason over knowledge graphs, run governed tool calls, detect contradictions, verify integrity, use compressed search, emit receipt chains, and pass a full E2E vertical slice test.

---

## What Already Works (verified by execution)

- cargo check --workspace: 0 errors
- cargo test --workspace: 433 tests, 0 failures
- 27 of 34 crates have real implementations
- Shadow semantics resolved: no local type duplication, all types from canonical owners
- The runner loop (PlanActVerifyLoopV1) executes: provider call, tool dispatch, boundary parse, receipt emission
- The app builder (AiDENsApp::builder().name().profile().config_file().tools().build()) works
- 5 profiles exist: ChatOnly, CodingAgent, MemoryAgent, AutonomousDaemon, ResearchWorkbench
- Boundary kit parses JSON, validates schemas, detects duplicate keys
- Provider kit routes to Mock/Ollama/Unavailable/Disabled with capability detection
- Tool kit registers tools, plans exposure, gates with permits, validates arguments
- Governance kit creates verification cases, check plans, control receipts, policy decisions, adjudication
- Memory kit opens MemoryStore, imports Forge exports, queries via KnowledgeRuntime, does temporal queries
- Kernel kit compiles projection batches, executes acyclic/message-passing, evaluates oracles
- 22 integration test files cover cross-crate behavior

## What's Missing (the gap between "framework" and "unified framework")

The kits are facades over types and some function calls, but they don't surface the full capability set. An agent today can: start, call a provider, parse tool output, emit receipts. An agent today cannot: search a knowledge base with hybrid search, detect contradictions, verify memory integrity, replay search receipts, use quantized/compressed search, run counterfactual analysis, create governance cases during a run, get release readiness decisions, or use the boundary compiler for canonical JSON digests.

---

## Architecture Constraint (must not violate)

AiDENs kits are adapters/facades. They never own canonical truth. Dependencies flow strictly upward through 5 strata. Every boundary crossing produces a receipt. The runner coordinates but does not own memory/tool/provider/kernel truth. These laws are already enforced and tested — the plan works within them.

---

## Phase 0: Gate Fixes (1-2 hours, unblocks certification)

These are trivial file operations from the existing COMPLETION_BLUEPRINT_P32.md. Listed here for completeness but not the focus of this plan.

0.1 Copy P29 matrices from archive to matrices/
0.2 Add P32 entries to CODEX_ARTIFACT_CLASSIFICATION.json
0.3 Create PHASE_15_AUDIT_LOG_HASHES.json
0.4 Create unified verify_release.sh wrapping all 29 assertion scripts
0.5 Fix duplicate receipt ID in agency-kit (add sequence counter)
0.6 Run z.py to generate P32 package

---

## Phase 1: Memory Kit — Full Search & Integrity Surface (3-4 days)

The memory kit currently exposes: open, import_forge_export, query, query_temporal. It does NOT expose: hybrid search, graph traversal, contradiction detection, integrity verification, search receipt replay, proveKV pool status, embedding operations, projection queries, stats.

### 1.1 Add MemorySearch facade

**File:** `crates/aidens-memory-kit/src/lib.rs`

Add methods to `CanonicalMemoryAdapter`:

```rust
/// Hybrid BM25 + vector search with RRF fusion.
pub async fn search(
    &self,
    query: &str,
    namespaces: Option<&[String]>,
    top_k: Option<usize>,
) -> Result<Vec<canonical_stack::SearchResult>, CanonicalMemoryAdapterError>

/// Search with deterministic context + receipt.
pub async fn search_with_receipt(
    &self,
    query: &str,
    namespaces: Option<&[String]>,
    top_k: Option<usize>,
) -> Result<(Vec<canonical_stack::SearchResult>, semantic_memory::VectorSearchReceiptV1), CanonicalMemoryAdapterError>

/// Full-text only search.
pub async fn search_fts_only(
    &self,
    query: &str,
    top_k: Option<usize>,
) -> Result<Vec<canonical_stack::SearchResult>, CanonicalMemoryAdapterError>

/// Vector-only search.
pub async fn search_vector_only(
    &self,
    query: &str,
    top_k: Option<usize>,
) -> Result<Vec<canonical_stack::SearchResult>, CanonicalMemoryAdapterError>

/// Explained search with full score breakdown.
pub async fn search_explained(
    &self,
    query: &str,
    top_k: Option<usize>,
) -> Result<semantic_memory::ExplainedSearchResponse, CanonicalMemoryAdapterError>
```

**Implementation:** Delegate to `self.store.search(...)`, `self.store.search_with_context(...)`, etc. The MemoryStore already has all these methods. This is pure delegation — no new logic.

**Test:** Add test that opens a store with MockEmbedder, adds a fact, searches, and verifies results + receipt.

### 1.2 Add MemoryReceipt facade

```rust
/// Load a durable search receipt by ID.
pub async fn get_search_receipt(
    &self,
    receipt_id: &str,
) -> Result<Option<semantic_memory::VectorSearchReceiptV1>, CanonicalMemoryAdapterError>

/// Deterministically replay a search receipt.
pub async fn replay_search_receipt(
    &self,
    receipt: &semantic_memory::VectorSearchReceiptV1,
) -> Result<Vec<canonical_stack::SearchResult>, CanonicalMemoryAdapterError>
```

**Implementation:** Delegate to `self.store.get_search_receipt(...)` and `self.store.replay_search_receipt(...)`.

### 1.3 Add MemoryGraph facade

```rust
/// Add a typed graph edge between two items.
pub async fn add_graph_edge(
    &self,
    params: semantic_memory::AddGraphEdgeParams,
) -> Result<semantic_memory::StoredGraphEdge, CanonicalMemoryAdapterError>

/// List graph edges for a node.
pub async fn list_graph_edges(
    &self,
    node_id: &str,
) -> Result<Vec<semantic_memory::StoredGraphEdge>, CanonicalMemoryAdapterError>

/// Count total graph edges.
pub async fn count_graph_edges(&self) -> Result<usize, CanonicalMemoryAdapterError>
```

### 1.4 Add MemoryIntegrity facade

```rust
/// Verify memory integrity (FTS/embedding/vector consistency).
pub async fn verify_integrity(
    &self,
    mode: semantic_memory::VerifyMode,
) -> Result<semantic_memory::IntegrityReport, CanonicalMemoryAdapterError>

/// Reconcile memory (rebuild FTS, re-embed stale vectors).
pub async fn reconcile(
    &self,
    mode: semantic_memory::VerifyMode,
) -> Result<Vec<semantic_memory::ReconcileAction>, CanonicalMemoryAdapterError>

/// Get memory statistics.
pub async fn stats(&self) -> Result<semantic_memory::MemoryStats, CanonicalMemoryAdapterError>
```

### 1.5 Add MemoryWrite facade

```rust
/// Store a fact in the knowledge base.
pub async fn add_fact(
    &self,
    namespace: &str,
    content: &str,
    source: Option<&str>,
    confidence: Option<f64>,
) -> Result<String, CanonicalMemoryAdapterError>
```

### 1.6 Add ProveKV status facade (feature-gated)

```rust
/// Check proveKV pool artifact status (feature: poly-kv-pool).
#[cfg(feature = "poly-kv-pool")]
pub async fn provekv_pool_status(
    &self,
) -> Result<semantic_memory::ProveKvPoolArtifactStatusV1, CanonicalMemoryAdapterError>
```

### 1.7 Add MemoryConfig presets

```rust
/// Preset config for a local-first agent with SQLite + usearch.
pub fn local_first_config(db_path: impl Into<PathBuf>) -> canonical_stack::CanonicalMemoryConfig

/// Preset config for testing with MockEmbedder.
pub fn mock_config(db_path: impl Into<PathBuf>) -> canonical_stack::CanonicalMemoryConfig
```

---

## Phase 2: Kernel Kit — Full Reasoning Surface (2-3 days)

The kernel kit currently exposes: compile_projection_batch, execute_acyclic, execute_message_passing, evaluate_exact_bounded, evaluate_conservative, schedule_execution. It does NOT expose: conformance gate execution, belief propagation with stop reasons, oracle mode selection, or make it easy to go from "search results" to "reasoning input."

### 2.1 Add reasoning-from-search bridge

```rust
/// Compile search results into a constraint graph for reasoning.
pub fn compile_search_results(
    &self,
    results: &[canonical_stack::SearchResult],
    policy: &canonical_stack::CompilerPolicy,
) -> canonical_stack::CompileOutput
```

This wraps `constraint_compiler::compile_batch` but takes search results directly, converting them to `ProjectionImportBatchV3` internally. Agents shouldn't have to manually construct projection batches.

### 2.2 Add high-level reasoning method

```rust
/// Run full reasoning pipeline: compile -> execute -> evaluate oracle.
pub fn reason(
    &self,
    results: &[canonical_stack::SearchResult],
    max_iterations: u32,
) -> ReasoningOutput
```

where `ReasoningOutput` bundles `ExecutionReport`, `OracleAssessment`, and `ExecutionStopReason` into one struct. This is the "one call to reason" method.

### 2.3 Add conformance check method

```rust
/// Run conformance gates against a compiled graph.
pub fn check_conformance(
    &self,
    compiled: &canonical_stack::CompileOutput,
) -> Vec<ConformanceGateResult>
```

where `ConformanceGateResult` wraps the gate ID, pass/fail, and details.

---

## Phase 3: Governance Kit — Full Verification & Assurance Surface (2-3 days)

The governance kit currently creates verification cases, check plans, control receipts, policy decisions, and adjudication results. It does NOT expose: assurance-runtime release readiness, authority-delegation chains, or make it easy to create a governance case during a run.

### 3.1 Add ReleaseReadiness facade

```rust
/// Create a release readiness decision for a deployment.
pub fn release_readiness_decision(
    &self,
    deployment_profile: &assurance_runtime::DeploymentProfileV1,
    blocking_gaps: Vec<String>,
    required_monitors: Vec<String>,
    advisory_only: bool,
) -> assurance_runtime::ReleaseReadinessDecisionV1
```

### 3.2 Add AuthorityDelegation facade

```rust
/// Create an authority chain for delegated actions.
pub fn authority_chain(
    &self,
    lease: &authority_delegation::AuthorityLeaseV1,
    acting_on_behalf: &authority_delegation::ActingOnBehalfReceiptV1,
) -> authority_delegation::DelegationBundleV1
```

### 3.3 Add GovernanceContext for runner integration

```rust
/// A governance context that the runner can use to create cases
/// and check permits during a run.
pub struct GovernanceContext {
    adapter: CanonicalGovernanceAdapter,
    policy_snapshot: canonical_stack::PolicySnapshot,
}

impl GovernanceContext {
    pub fn new(policy: canonical_stack::PolicySnapshot) -> Self
    pub fn open_case(&self, class: canonical_stack::VerificationCaseClass, ...) -> canonical_stack::VerificationCase
    pub fn check_permit(&self, case: &canonical_stack::VerificationCase, plan: &canonical_stack::CheckPlan) -> Result<canonical_stack::ExecutionPermit, ...>
    pub fn adjudicate(&self, ...) -> canonical_stack::AdjudicationResult
}
```

This makes governance usable from the runner without the caller constructing 5 intermediate objects.

---

## Phase 4: Security Kit — MCP Trust & Attestation (1-2 days)

The security kit is 144 lines — just path safety and side-effect classification. It does NOT expose: MCP trust gate, attestation exchange, or remote oracle admission.

### 4.1 Add McpTrustGate

```rust
/// Evaluate whether an MCP tool descriptor is safe to expose.
pub fn evaluate_mcp_tool_safety(
    descriptor: &llm_tool_runtime::ToolDescriptor,
) -> McpTrustReportV1
```

where `McpTrustReportV1` wraps: permission scopes, policy IDs, risk classification, allow/deny decision.

### 4.2 Add attestation facade

```rust
/// Create an attestation envelope for a tool invocation.
pub fn attest_tool_invocation(
    receipt: &llm_tool_runtime::ToolReceipt,
) -> attestation_exchange::AttestationEnvelopeV1
```

---

## Phase 5: Boundary Kit — Canonical Digest Integration (1 day)

The boundary kit parses JSON and validates schemas. It does NOT use the boundary-compiler crate from the parent workspace for canonical JSON digest computation.

### 5.1 Wire boundary-compiler for canonical digests

```rust
/// Compute a canonical RFC 8785 JSON digest for any value.
pub fn canonical_digest(value: &serde_json::Value) -> Result<boundary_compiler::ContentDigest, boundary_compiler::JcsError>

/// Canonicalize a JSON value per RFC 8785.
pub fn canonicalize_json(value: &serde_json::Value) -> Result<String, boundary_compiler::JcsError>
```

Add `boundary-compiler` as a dependency of `aidens-boundary-kit` in Cargo.toml.

### 5.2 Add boundary-compiler-core wiring

The `canonical_boundary` submodule already exists but delegates to `boundary-compiler-core` (the AiDENs-internal version). Wire it to the parent `boundary-compiler` crate instead, or keep both and document which is canonical. The parent `boundary-compiler` is published on crates.io (0.1.0, 23 downloads) — it's the canonical one.

---

## Phase 6: Runner Integration — Make Capabilities Accessible During Runs (3-4 days)

The runner currently calls: provider, tool dispatcher, boundary parser, receipt log. It does NOT call: memory search, kernel reasoning, governance case creation, or integrity verification during a run.

This is the core phase — it's what makes AiDENs a unified framework instead of a collection of adapters.

### 6.1 Add memory to the runner

**File:** `crates/aidens-runner/src/lib.rs`

Add `memory: Option<CanonicalMemoryAdapter>` to `PlanActVerifyLoopV1`.

Add builder method:
```rust
pub fn with_memory(mut self, memory: CanonicalMemoryAdapter) -> Self
```

In the `execute` method, after receiving the provider response, if memory is configured:
1. Search memory for context relevant to the prompt
2. Inject search results into the prompt context for the next provider call
3. Emit a `MemoryGroundingEvidenceV1` receipt for the search

This is the single highest-value change in the entire plan. It makes every agent automatically grounded in memory.

### 6.2 Add governance to the runner

Add `governance: Option<GovernanceContext>` to `PlanActVerifyLoopV1`.

Add builder method:
```rust
pub fn with_governance(mut self, governance: GovernanceContext) -> Self
```

In the `execute` method, before tool dispatch:
1. Open a verification case for the tool call
2. Create a check plan
3. Evaluate policy
4. If permit denied, block the tool call and emit a receipt
5. After tool execution, adjudicate and emit control receipt

### 6.3 Add kernel reasoning to the runner

Add `kernel: Option<CanonicalKernelAdapter>` to `PlanActVerifyLoopV1`.

Add builder method:
```rust
pub fn with_kernel_reasoning(mut self, kernel: CanonicalKernelAdapter) -> Self
```

In the `execute` method, after memory search, if kernel is configured:
1. Compile search results into a constraint graph
2. Run belief propagation
3. Evaluate oracle
4. If oracle says "insufficient evidence," emit an abstention receipt
5. If oracle says "supported," inject the reasoning result into the prompt context

### 6.4 Add integrity check to the runner

Add a `verify_memory_integrity` method that runs after a run completes:
```rust
pub async fn verify_run_integrity(&self) -> Result<IntegrityReport, ...>
```

This proves the memory was not corrupted during the run.

---

## Phase 7: App Kit — Profile Integration (2-3 days)

The app kit builds apps from profiles. Profiles expand to plans. But the plans don't currently configure memory, kernel, governance, or security — only provider, tools, and permits.

### 7.1 Add memory config to profiles

In `AiDENsProfile::runtime_defaults` and `AiDENsProfile::expand`, add:
- `memory_config: Option<MemoryConfigV1>` — database path, embedder config, namespaces
- `memory_mode: MemoryModeV1` — None, ReadOnly, ReadWrite, GroundedChat

Profiles:
- ChatOnly: memory = None
- CodingAgent: memory = None (code is on disk)
- MemoryAgent: memory = ReadWrite, grounded chat enabled
- AutonomousDaemon: memory = ReadWrite + governance enabled
- ResearchWorkbench: memory = ReadWrite + kernel reasoning enabled

### 7.2 Add governance config to profiles

Add `governance_config: Option<GovernanceConfigV1>` to the plan. Profiles:
- ChatOnly: governance = None
- CodingAgent: governance = None
- MemoryAgent: governance = None
- AutonomousDaemon: governance = Strict
- ResearchWorkbench: governance = AdvisoryOnly

### 7.3 Wire app builder to construct all adapters

In `AiDENsAppBuilder::build`, if the plan includes memory config, construct a `CanonicalMemoryAdapter` and pass it to the runner. If governance config, construct `GovernanceContext`. If kernel, construct `CanonicalKernelAdapter`.

This is where it all comes together. The user says `AiDENsApp::builder().profile(AiDENsProfile::MemoryAgent).build()` and gets an agent that searches memory, reasons over knowledge, creates governance cases, and emits receipt chains — all automatically.

---

## Phase 8: ClaimLedger Integration (1-2 days)

ClaimLedger is a Python crate (pyproject.toml), not Rust. It's consumed by Gloss and AiDENs at the semantic level. The Rust `claim-ledger` crate in Libraries (1.9K lines, v0.1.0, MIT, published on crates.io) IS Rust and should be wired.

### 8.1 Add claim-ledger as a dependency

Add `claim-ledger` to `aidens-memory-kit` or create a new `aidens-claim-kit`.

### 8.2 Surface claim/evidence operations

```rust
/// Record a claim from source material.
pub fn record_claim(&self, source_id: &str, span_id: &str, claim_text: &str, claim_type: &str) -> claim_ledger::Claim

/// Create an evidence bundle for a claim.
pub fn create_evidence_bundle(&self, claim_id: &str) -> claim_ledger::EvidenceBundle

/// Record a support judgment.
pub fn record_support_judgment(&self, claim_id: &str, state: claim_ledger::SupportState, rationale: &str) -> claim_ledger::SupportJudgment
```

---

## Phase 9: Compression Wiring (2-3 days, feature-gated)

turbo-quant, poly-kv, quant-governor, gpu-backend, and scr-runtime-compression are all in the Libraries workspace but not wired into AiDENs.

### 9.1 Add compression config to memory kit

```rust
/// Enable turbo-quant compressed search (feature: turbo-quant-codec).
#[cfg(feature = "turbo-quant-codec")]
pub fn with_turbo_quant(mut config: CanonicalMemoryConfig) -> CanonicalMemoryConfig

/// Enable proveKV pool candidate search (feature: poly-kv-pool).
#[cfg(feature = "poly-kv-pool")]
pub fn with_provekv_pool(mut config: CanonicalMemoryConfig) -> CanonicalMemoryConfig
```

### 9.2 Surface compression receipts

```rust
/// Get the derived candidate receipt for a search (if compressed backend was used).
pub async fn derived_candidate_receipt(&self) -> Option<semantic_memory::DerivedCandidateReceiptV1>
```

### 9.3 Add quant-governor routing (optional, advanced)

```rust
/// Route a compression decision through quant-governor.
pub fn route_compression(
    content_type: quant_governor::ContentType,
    size: usize,
    policy: &quant_governor::GovernancePolicy,
) -> quant_governor::CodecDecision
```

---

## Phase 10: E2E Vertical Slice Proof (2-3 days)

This is the test that proves AiDENs works as a unified framework.

### 10.1 Write the vertical slice test

**File:** `crates/aidens-integration-tests/tests/e2e_unified_agent.rs`

```rust
#[tokio::test]
async fn e2e_unified_agent_with_memory_reasoning_governance() {
    // 1. Create a memory store with MockEmbedder
    let memory = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(tempdir()),
        runtime_config_for_namespace("test"),
    )?;

    // 2. Add facts to the knowledge base
    memory.add_fact("test", "Rust is memory-safe without garbage collection", None, None).await?;
    memory.add_fact("test", "Python uses reference counting + garbage collection", None, None).await?;

    // 3. Create governance context with permissive policy
    let governance = GovernanceContext::new(PolicySnapshot::permissive("v1", "2026-01-01"));

    // 4. Create kernel adapter
    let kernel = CanonicalKernelAdapter::default();

    // 5. Build an agent
    let app = AiDENsApp::builder()
        .name("test-agent")
        .profile(AiDENsProfile::MemoryAgent)
        .mock_provider("Rust is memory-safe without GC")
        .build().await?;

    // 6. Run a prompt
    let output = app.run_once("Is Rust memory safe?").await?;

    // 7. Verify the run produced:
    //    - A provider response
    //    - A memory search receipt (grounded in facts)
    //    - A governance case + control receipt
    //    - A kernel reasoning result (if kernel configured)
    //    - A run report with the full receipt chain
    assert!(output.run_report.is_some());
    assert!(output.memory_grounding_evidence.is_some());
    assert!(output.governance_receipts.len() > 0);

    // 8. Verify memory integrity after the run
    let integrity = memory.verify_integrity(VerifyMode::Full).await?;
    assert!(integrity.errors.is_empty());
}
```

### 10.2 Write the receipt chain verification test

Verify that the full receipt chain is replayable:
- Run receipt -> turn receipt -> provider route receipt -> tool attempt receipt -> memory grounding receipt -> governance control receipt -> boundary repair receipt (if any)

### 10.3 Write the compression integration test (feature-gated)

```rust
#[cfg(feature = "turbo-quant-codec")]
#[tokio::test]
async fn e2e_compressed_search() {
    // Build memory with turbo-quant codec
    // Add facts
    // Search
    // Verify derived candidate receipt exists
    // Verify exact rerank happened
    // Verify results match uncompressed search
}
```

---

## Phase 11: Documentation & Visibility (1 week, after wiring)

### 11.1 Write README.md for every crate

Each crate gets a README with:
- What it does (1 paragraph)
- What sibling crates it wraps
- Key API surface (3-5 most important types/methods)
- Example code snippet
- Feature flags (if any)

### 11.2 Add Cargo.toml descriptions to all 33 crates missing them

### 11.3 Write the top-level AiDENs README

The one that shows:
```
AiDENsApp::builder()
    .name("my-agent")
    .profile(AiDENsProfile::MemoryAgent)
    .build().await?
    .run().await?
```
and explains what happens internally (memory search, reasoning, governance, receipts).

### 11.4 Write the architecture diagram

5 strata, showing which sibling crates each kit wraps.

### 11.5 Publish to crates.io

After all wiring is done and tests pass, publish the crates that are stable enough. Start with: aidens-contracts, aidens-boundary-kit, aidens-receipts, aidens-config, aidens-capability-kit, aidens-permit-kit, aidens-security-kit, aidens-budget-kit.

---

## Execution Order & Dependencies

```
Phase 0 (gate fixes)     — independent, do first
Phase 1 (memory kit)     — independent, highest ROI
Phase 2 (kernel kit)     — independent
Phase 3 (governance kit) — independent
Phase 4 (security kit)   — independent
Phase 5 (boundary kit)   — independent
Phase 6 (runner)         — depends on Phases 1-5
Phase 7 (app kit)        — depends on Phase 6
Phase 8 (claim ledger)   — depends on Phase 1
Phase 9 (compression)    — depends on Phase 1, feature-gated
Phase 10 (E2E test)      — depends on Phases 6-7
Phase 11 (docs)          — depends on Phase 10
```

Phases 1-5 can run in parallel (5 independent workstreams).
Phase 6 is the critical path (runner integration).
Phase 7 is the user-facing surface (profiles configure everything).
Phase 10 is the proof.
Phase 11 is the visibility.

Total estimated time: 3-4 weeks for one person working sequentially. 2 weeks if Phases 1-5 are parallelized (e.g., with Codex/Claude agents).

---

## What Makes This "Easy to Implement"

1. **Pure delegation.** Every new method in Phases 1-5 is a thin wrapper calling an existing sibling crate method. No new logic, no new algorithms. The sibling crates already work — AiDENs just needs to call them.

2. **Consistent facade pattern.** Every kit already follows the same pattern: `canonical_stack` module re-exports types, an `Adapter` struct wraps function calls. New methods follow the same pattern.

3. **Feature-gated compression.** Phase 9 (compression wiring) is behind feature flags so it doesn't break the default build. Agents that don't need compression pay zero cost.

4. **The runner is already extensible.** `PlanActVerifyLoopV1` already has builder methods for tools, permits, providers, receipts, agency policy. Adding `with_memory`, `with_governance`, `with_kernel_reasoning` follows the exact same builder pattern.

5. **Profiles already expand to plans.** Adding memory/governance/kernel config to profiles is adding fields to an existing expansion — not a new architecture.

6. **The E2E test writes itself.** Once the runner has memory + governance + kernel, the test is just "build app, run prompt, check receipts." The receipt chain verification is the proof.

7. **No new crates needed.** Every capability is surfaced through existing kits. No new workspace members, no new Cargo.toml files, no new dependency edges in the workspace graph.

8. **The specs already exist.** The prior-design-packet (18 files) defines the intended architecture. The API sketches (doc 15) show the intended user surface. The design laws (doc 14) constrain the implementation. This plan just executes what was already designed.

---

## Risk Mitigation

- **Don't break existing tests.** Every new method is additive. Existing 433 tests must still pass.
- **Feature-gate compression.** turbo-quant-codec and poly-kv-pool features are opt-in. Default build stays clean.
- **Keep memory optional.** ChatOnly profile has no memory. Don't force memory on agents that don't need it.
- **Keep governance optional.** ChatOnly and CodingAgent profiles have no governance. Don't force governance on simple agents.
- **Keep kernel optional.** Kernel reasoning is for ResearchWorkbench and advanced agents. Don't force it on basic agents.
- **Receipts for everything.** Every new capability call emits a receipt. If a memory search happens, there's a receipt. If a governance case opens, there's a receipt. If a kernel reasoning run happens, there's a receipt.
- **No shadow semantics.** All types come from canonical owners. No local redefinitions. The shadow semantics audit must stay clean.

---

## Verification Checklist

- [ ] Phase 0: All 29+ gates pass via verify_release.sh
- [ ] Phase 1: Memory kit exposes search, graph, integrity, receipts, replay, add_fact, provekv status
- [ ] Phase 2: Kernel kit exposes compile-from-search, reason(), check_conformance()
- [ ] Phase 3: Governance kit exposes release_readiness, authority_chain, GovernanceContext
- [ ] Phase 4: Security kit exposes MCP trust gate, attestation
- [ ] Phase 5: Boundary kit uses parent boundary-compiler for canonical digests
- [ ] Phase 6: Runner calls memory search, governance cases, kernel reasoning during execute()
- [ ] Phase 7: Profiles configure memory, governance, kernel; app builder constructs all adapters
- [ ] Phase 8: Claim ledger operations accessible from AiDENs
- [ ] Phase 9: Compressed search works behind feature flags
- [ ] Phase 10: E2E vertical slice test passes (memory + reasoning + governance + receipts + integrity)
- [ ] Phase 11: All 34 crates have README.md and Cargo.toml descriptions
- [ ] cargo test --workspace: all existing 433 tests + all new tests pass
- [ ] cargo check --workspace: 0 errors
- [ ] cargo clippy --workspace: 0 warnings (excluding sibling semantic-memory)