# Hostile Audit Full Remediation Plan — 2026-07-13 (Phase 2)

## Already fixed (15/77) — commits 861fbad through fcc6bf2
See HOSTILE_AUDIT_REMEDIATION_HANDOFF_2026-07-13.md for details.

## Remaining 62 findings — organized into Codex dispatchable workstreams

### Batch A: Boundary & Identity (BOUND-002 through BOUND-010)
**Crate**: boundary-compiler, stack-ids, bitemporal-runtime, claim-ledger
**Files**: boundary-compiler/src/{canonicalizer.rs, schema.rs, profile.rs, lib.rs, error.rs}, stack-ids/src/{ids.rs, digest.rs}, bitemporal-runtime/src/{types.rs, sqlite.rs}, claim-ledger/src/{ledger.rs, types.rs}

- BOUND-002 [Critical]: Replace raw scanner duplicate-key detection with serde_json streaming visitor that tracks decoded property names per object before map construction. Remove post-parse duplicate checks.
- BOUND-003 [Critical]: Add RFC 8785 Appendix B test vectors (numbers and property-order examples) to jcs_tests.rs. Fix number formatting to use ryu/lexical instead of serde_json::Number::to_string(). Fix string escaping to only escape U+0000-U+001F (not all control chars). Fix key sorting to use UTF-16 code units.
- BOUND-004 [High]: Compile BoundaryProfile fields into an immutable executable plan. For each profile field, either enforce it during parsing or remove it. Add aggregate byte/node/depth budgets.
- BOUND-005 [High]: Create cross-crate golden tests proving stack-ids::ContentDigest::compute_json() and boundary-compiler::ContentDigest::compute() produce identical bytes for the same input. Add domain separator and algorithm/profile/version to both digest types.
- BOUND-006 [High]: Make ID newtype fields private in stack-ids. Add validated parsing (non-empty, format checks). Separate IssuedId from ExternalId. Add #[derive(not(Clone))] or remove Clone where mint-only is required.
- BOUND-008 [High]: Require T: Serialize in append_supersede. Canonicalize the complete BitemporalRecord before receipt creation. Preserve nanoseconds. Make receipt creation fallible (Result<>).
- BOUND-009 [Medium]: Document the current model as "dual-time append history" (not full interval bitemporality). Add property tests for retroactive corrections, future-effective facts, retractions, same-time ties.
- BOUND-010 [High]: Define ArtifactEnvelopeV1 struct mapped to in-toto/SLSA concepts. Add signature, signer identity, trusted time fields. Distinguish digest-valid, signature-valid, signer-authorized, time-valid, policy-admitted states.

### Batch B: Memory & Retrieval (MEM-001, MEM-003 through MEM-010, MEM-012 through MEM-014)
**Crate**: semantic-memory, semantic-memory-mcp
**Files**: semantic-memory/src/{lib.rs, search.rs, usearch_backend.rs, config.rs, vector_backend.rs}, semantic-memory-mcp/src/server.rs

- MEM-001 [High]: Add a default-feature integration test that inserts data, proves USearch files exist, observes ANN candidate retrieval, restarts, and rebuilds.
- MEM-003 [High]: Validate the entire batch first (check finite, correct dimensions), then atomically populate the cache. A batch with one bad vector must leave the cache unchanged.
- MEM-004 [High]: Create SearchRequestV2 with canonicalized query, k, namespaces, context, exactness profile, backend generation, and corpus revision in the cache key. Changing any field must cause a miss.
- MEM-005 [High]: Fix USearch key collision detection to check both directions (key→id and id→key) before insertion. Add property tests with forced hash collisions.
- MEM-007 [High]: Stage USearch changes — apply to native index first, then commit map state only after success. Fault injection at every native call must leave old or new complete state.
- MEM-008 [High]: Write immutable generation directories with manifest/digests, fsync, then atomically swap a CURRENT pointer. Reject any mismatch on load and rebuild from SQLite.
- MEM-009 [High]: Add compile_error! when both hnsw and usearch-backend features are enabled simultaneously. Report selected backend in stats and receipts.
- MEM-010 [High]: Validate every numeric field in MemoryConfig. Preserve valid fields when one is invalid. Return a structured correction report. Add property tests over arbitrary configs.
- MEM-012 [High]: Find all text compaction paths that slice by byte index. Replace with char_indices or grapheme-aware truncation. Add fuzz tests with CJK, emoji, combining characters.
- MEM-013 [High]: Wire sm_add_fact metadata (memory_kind, sensitivity, evidence_refs) into the actual add_fact call. Read it back in the integration test and verify it's persisted.
- MEM-014 [High]: Gate direct fact update/delete behind admin-ops feature. Normal builds must have no public hard mutation path. Break-glass actions must produce signed incident receipts.

### Batch C: Authority & Effects (AUTH-004, AUTH-005, AUTH-007 through AUTH-010)
**Crate**: llm-tool-runtime, forge-pilot
**Files**: llm-tool-runtime/src/{contracts.rs, runtime.rs, semantic_memory.rs}, forge-pilot/src/{loop_runner.rs, act.rs, bundle_builder.rs}

- AUTH-004 [Critical]: Make ToolExecutionPermit non-cloneable (remove Clone derive). Add expiry timestamp, nonce, one-shot consumption flag. Add method digest and effect digest binding. Test: replay fails, expired fails, double-spend fails.
- AUTH-005 [High]: Add describe_effect(args) -> EffectIntent method to ToolDescriptor. Canonicalize and digest the intent. Add negative tests for alternate/nested/compound targets.
- AUTH-006 [Critical]: Add durable preflight receipt before tool invocation. If persistence fails, abort before effect. Persist recovery record. Test: kill at persistence point, restart reconciles.
- AUTH-007 [High]: Add authority lineage (origin label, permit chain) to tool receipts. Given a receipt, an offline verifier must resolve the full chain.
- AUTH-008 [High]: Change default receipt persistence from Ephemeral to Durable. Release config must refuse to start effectful routes without a health-checked durable receipt store.
- AUTH-009 [High]: Define CompensatingActionV1 trait and rollback contract. Policy must reject irreversible effects without explicit approval. Failure injection must execute compensation or produce durable incident.
- AUTH-010 [High]: Separate control data (permits, policy, receipts) from untrusted data (model output, tool args) in typed wrappers. Add AgentDojo-style injection tests for forbidden data flows.

### Batch D: MCP Security (MCP-002, MCP-004, MCP-005, MCP-006)
**Crate**: semantic-memory-mcp
**Files**: semantic-memory-mcp/src/{http_server.rs, server.rs, profile.rs}

- MCP-002 [High]: Add body size cap (already 10MB at line 271), connection timeout (already 2s at line 194), and concurrency limit. Add Slowloris and oversized body tests.
- MCP-004 [High]: Apply per-handler authorization with authenticated subject, capability, and scope. Direct invocation of a hidden/admin tool must fail without capability.
- MCP-005 [High]: Treat caller-supplied sensitivity labels as claims. Add deterministic DLP check (regex for secrets, API keys, PII patterns). Block regardless of caller label. Produce redacted incident receipt.
- MCP-006 [High]: Pin tool descriptor/schema digests into permits and receipts. Descriptor mutation after approval must invalidate permits. Add a local tool registry with signed manifests.

### Batch E: Graph Orchestration (GRAPH-003, GRAPH-004, GRAPH-005)
**Crate**: agent-graph
**Files**: agent-graph/src/{engine.rs, checkpoint.rs, checkpoint_store.rs, graph.rs}

- GRAPH-003 [High]: Make checkpoint policy explicit (Required, BestEffort, Disabled). Required mode must atomically persist state and receipt before advancing. Fault-injected checkpoint failure must stop before next node.
- GRAPH-004 [High]: Replace DefaultHasher with a canonical GraphSpecV1 digest (blake3 of serialized graph topology, node schemas, edge conditions, retry/checkpoint policy). Cross-process golden tests must be stable.
- GRAPH-005 [High]: Add RunBundleV1 (graph spec, input, model/tool/memory envelopes, step state deltas, terminal receipt). Support verify-only replay without network. Divergences must localize to a step.

### Batch F: Autonomous Learning (AUTO-002, AUTO-005, AUTO-006, AUTO-007, AUTO-008)
**Crate**: AiDENs/crates/aidens-autonomous
**Files**: AiDENs/crates/aidens-autonomous/src/{evaluation.rs, loop_driver.rs, capture.rs, receipt.rs}

- AUTO-002 [High]: Replace lexical scoring with evidence-based scoring. Add source spans, retrieval evidence, contradiction witnesses. Add a labeled benchmark measuring precision, recall, false promotion, calibration error.
- AUTO-005 [High]: Persist each cycle receipt before state transition. Hash-chain to a durable root. Add offline verification/replay from genesis. Restart mid-cycle must recover last committed state.
- AUTO-006 [High]: Route all candidates through claim-ledger types (source artifact, span, normalized claim, evidence bundle, support judgment, admission receipt). Every promoted fact must resolve to source span + evidence.
- AUTO-007 [High]: Generate AiDENs maturity matrix from workspace inventory. Update status docs to match current tree. Status timestamp/commit must match.
- AUTO-008 [High]: Add shadow/reviewed/autonomous modes. Ship shadow by default. Shadow mode must be provably write-isolated. Produce candidate bundles and evaluation reports without canonical writes.

### Batch G: Repository Truth & CI (TRUTH-001, TRUTH-003 through TRUTH-010)
**Files**: Cargo.toml, Makefile, scripts/, .github/workflows/, README.md, AGENTS.md, deny.toml

- TRUTH-001 [High]: Document that default branch must be `main` (GitHub settings change — operator action). Add a CI check that fails if default branch is not main.
- TRUTH-003 [High]: Publish evidence under evidence/<commit_sha>/<gate_digest>/. Add CURRENT_EVIDENCE.json pointer binding commit, workflow run, gate digest, and artifact digests.
- TRUTH-004 [High]: Create repo_contract.toml with every package, workspace, maturity, support tier, owners, features, and required gates. Generate support docs, CI matrices, receipts from it.
- TRUTH-005 [High]: Replace substring gates with generated data and semantic comparisons. Mutating a crate count, branch, support tier, or limitation must regenerate views or fail with drift error.
- TRUTH-006 [High]: Add generated CI matrix for every active Cargo workspace/package root (fmt, check, test, clippy, doc, feature combinations).
- TRUTH-007 [High]: Replace root README with ecosystem map, support/maturity matrix, workspace map, validation commands, stable operating doctrine. Move pass bundles under docs/archive/runs/.
- TRUTH-008 [High]: Declare one authoritative schema registry with stable IDs, versions, owners, compatibility rules. Make every consumer resolve by schema ID, not path.
- TRUTH-009 [Medium]: Generate workspace members from checked package registry or add strict parser rejecting duplicates, nonexistent members, and name collisions.
- TRUTH-010 [Medium]: Require machine-readable test/benchmark attestations with command, environment, toolchain, input digests, exit status, stdout/stderr digest, and commit/tree digest.

### Batch H: Compression, GPU & Supply Chain (QUANT-001 through QUANT-008, SEC-002 through SEC-006)
**Files**: gpu-backend/src/, turbo-quant/src/, quant-eval/src/, poly-kv/Cargo.toml, deny.toml, .github/workflows/ci.yml

- QUANT-001 [High]: Add validated TensorShape/QuantProfile newtypes with checked arithmetic. Reject k==0, n_levels>256, overflow. Add Kani/proptest harnesses.
- QUANT-002 [High]: Add scheduled CUDA matrix with CPU/GPU byte parity, determinism, corruption tests. Record artifacts.
- QUANT-003 [High]: Label FibQuant claims as fixture-local. Add dataset digest, query count, model, seed, exact ground truth, realized bytes to all claims.
- QUANT-004 [High]: Report payload, envelope, index, codebook, alignment, allocator, transfer, and peak working-set bytes separately from actual artifacts.
- QUANT-005 [High]: Bind full canonical benchmark envelope (dataset/config/build/toolchain digests, distributions, errors, warmup) in receipt hash. Missing/extra benchmarks must fail.
- QUANT-006 [High]: Add dedicated matrix for poly-kv (no-default, each codec, full-stack, concurrency, corruption). Consider defaulting to exact/no-codec.
- QUANT-007 [High]: Split admissibility profiles: ann_candidate, embedding_storage, kv_random_access, kv_eviction, model_weight. Policy must reject wrong-domain benchmarks.
- QUANT-008 [Medium]: Document that D4/E8 lattice expansion is deferred pending comparative harness evidence. No new lattice code.
- SEC-002 [High]: Make deny.toml yanked = deny, sources = deny for release. Evaluate all feature sets. Add exception ledger with owner/expiry.
- SEC-003 [High]: Pin every GitHub Action to a reviewed commit SHA. Add policy check rejecting non-SHA uses: entries.
- SEC-004 [High]: Generate CycloneDX/SPDX SBOMs per artifact. Add SLSA provenance statements, signatures, checksums.
- SEC-005 [High]: Add cargo-fuzz corpora, Loom models, Kani harnesses, Miri for unsafe/native wrappers. Each critical boundary needs a named harness.
- SEC-006 [Medium]: Generate per-package platform/MSRV policy. Test Linux, macOS, Windows, current stable, declared MSRV.

## Codex dispatch strategy

### Phase 1: Boundary & Identity (Batch A) — sequential, single agent
boundary-compiler, stack-ids, bitemporal-runtime, claim-ledger are interdependent.
Agent: gpt-5.6-sol, workdir: /home/sikmindz/Coding/Libraries

### Phase 2: Memory & Retrieval (Batch B) — parallel with Phase 3
semantic-memory is independent of llm-tool-runtime.

### Phase 3: Authority & Effects (Batch C) — parallel with Phase 2
llm-tool-runtime is independent of semantic-memory.

### Phase 4: MCP Security (Batch D) — after Phase 2 (depends on semantic-memory)
### Phase 5: Graph (Batch E) — parallel with Phases 2-4
### Phase 6: Autonomous (Batch F) — after Phase 3 (depends on llm-tool-runtime patterns)
### Phase 7: Repo Truth & CI (Batch G) — parallel, independent
### Phase 8: Compression & Supply Chain (Batch H) — parallel, independent