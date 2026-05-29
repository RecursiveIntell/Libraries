# Libraries Completion Plan — Master Synthesis

**Date:** 2026-05-29  
**Prepared for:** Josh Stevenson / RecursiveIntell  
**Source basis:** Downloads research dossier (2026-05-25 to 2026-05-27), Libraries V30 hardening roadmap, FibQuant/TurboQuant/Poly-KV specs, Gloss P36 release completion truth  

---

## Executive Finding

The research converges on one architecture thesis:

> **RecursiveIntell Libraries should be a proof-governed compressed persistent runtime where semantic memory, reasoning graphs, compression, tool authority, and answer testimony are typed, receipt-bearing projections over canonical source/evidence artifacts.**

The Libraries workspace (676 packages, 52 crates with Cargo.toml) has completed V30 Phase 1-2. Phase 3 remains: turbo-quant/fib-quant wiring to quant-governor, poly-kv workspace merge, semantic-memory HNSW bug fixes (SM-AUD-0010/0011/0026/0027/0042/0058/0059), and llm-pipeline batching receipts.

Gloss audit remediation is complete (146 Rust + 12 TS tests, 0 errors, 0 warnings). Remaining Gloss work: turbo-quant→quant-governor integration, E2E tests, and release-candidate truth/receipt gates.

---

## Decision Summary

| Decision | Recommendation | Why it matters |
|---|---|---|
| D01 | Build receipt schemas before promoting any new runtime path | Compression, graph persistence, and tool execution can silently corrupt truth if receipts are added later |
| D02 | Treat compression as a derived projection | Raw source spans, claims, evidence, and exact baselines remain authoritative |
| D03 | Promote AgentSecurity argument provenance to P0 | PACT validates the core failure: authority-bearing arguments, not whole tool calls, are the security boundary |
| D04 | Make persistent reasoning subgraphs derived and rebuildable | AutoPrunedRetriever is strategically aligned, but graph pruning can erase contradiction history unless governed |
| D05 | Split retrieval, reasoning, and answer testimony | GraphRAG results show the answer can be present in retrieved context while the model still fails to reason correctly |
| D06 | Keep AiDENs as orchestration/adjudication only | AiDENs should route and enforce; canonical truth belongs to semantic-memory, ClaimLedger, AgentSecurity, and quant-governor |

---

## Priority Build Order

1. **P0 Receipt schema pack** — `SemanticResidualReceiptV1`, `CapabilityArgumentContractV1`, `ArgumentLineageReceiptV1`, `PersistentReasoningSubgraphV1`, `CompressionSurvivabilityReportV1`, `EvidenceSufficiencyReceiptV1`
2. **P0 AgentSecurity argument provenance gate** — semantic-role-aware contracts and mixed-trust dry-run enforcement
3. **P0 Compression survivability lab** — exact baseline, turbo-quant adapter, fib-quant prototype slot, DeltaKV residual harness, drift and contradiction metrics
4. **P0 ClaimLedger/semantic-memory graph projection lab** — persistent reasoning subgraphs with pruning history and rebuild receipts
5. **P1 Gloss Answer Testimony layer** — user-visible claims, support, contradictions, retrieval witnesses, reasoning path, compression mode, and degradation status
6. **P1 AiDENs/Recall integration** — route receipts, tool exposure scopes, memory drift, replay, and rollback

---

## Phase Plan Overview

| Phase | Window | Objective | Concrete outputs | Acceptance gate |
|---|---|---|---|---|
| Phase 0 | 0-3 days | Source-basis freeze and no-overclaim ledger | Source ledger, evidence tiers, public-claim boundaries | All external numbers labeled source-reported; no claim promoted without local receipt |
| Phase 1 | 3-10 days | Receipt schema pack | Rust/JSON schema drafts for 10 receipt families; canonical hash rules; fixtures | Schemas round-trip; hash stable; invalid widening rejected |
| Phase 2 | 1-3 weeks | AgentSecurity argument provenance MVP | PACT-like role contracts for shell/path/URL/package/mutation args; dry-run mode; denial receipts | Mixed-trust tests prove unsafe args blocked while benign retrieval-then-act cases route to approval or execution |
| Phase 3 | 2-4 weeks | Compression survivability lab | Exact baseline + turbo-quant adapter + fib-quant slot + DeltaKV residual harness stub | Bench emits metric JSON, peak memory, replay hashes, failure corpus; no product path enabled |
| Phase 4 | 3-6 weeks | Persistent reasoning subgraph lab | ClaimLedger-backed subgraph persistence; pruning receipts; graph rebuild command | Pruning preserves contradiction lineage; graph projections rebuild from source/evidence |
| Phase 5 | 4-8 weeks | Gloss Answer Testimony preview | Answer export showing claims, source spans, retrieval witnesses, graph path, compression mode, unsupported claims | Hostile corpus reproduces support/contradiction decisions and shows degradation disclosures |
| Phase 6 | 6-12 weeks | AiDENs/Recall integration | Inference route receipts, tool exposure scopes, memory drift receipts, scheduler/action receipts | No hidden truth store; all control decisions replayable and receipt-bearing |

---

## Current State Summary

### Libraries Workspace
- **676 packages** in cargo metadata
- **52 crates** with Cargo.toml
- **612 uncommitted files** on `salvage/libraries2-20260525` branch
- **Status:** V30 Phase 1-2 complete, Phase 3 remaining

### Gloss
- **146 Rust + 12 TS tests**, 0 errors, 0 warnings
- **Completed:** pool migration (36 write sites → with_notebook_db_write), system logging, chat/mod.rs decomposed (3340→2529 lines), SM-AUD-0058/0064/0065 synced from Libraries/semantic-memory/, dead code removed, all warnings eliminated, log=0.4 added
- **Remaining:** turbo-quant→quant-governor, E2E tests

### Key Unfinished Work
1. turbo-quant/fib-quant wiring to quant-governor
2. poly-kv workspace merge
3. semantic-memory HNSW SM-AUD-0010/0011/0026/0027/0042/0058/0059 (0058 partially fixed)
4. llm-pipeline batching receipts
5. Receipt schema pack (10 families)
6. AgentSecurity argument provenance gate
7. Compression survivability lab
8. Gloss Answer Testimony layer

---

## Public Positioning Boundary

### Safe claim:
> RecursiveIntell is building proof-governed local AI infrastructure: systems that remember, compress, reason, invoke tools, and emit evidence for what happened.

### Avoid until locally reproduced:
- Any claim that RecursiveIntell achieves DeltaKV or FibQuant published compression numbers
- Any claim that compressed memory is semantically safe by default
- Any claim that GraphRAG proves answers merely because supporting context was retrieved
- Any claim that AgentSecurity prevents prompt injection unless the argument-provenance gate exists and passes local red-team fixtures

---

## Next Actions

1. Commit all 612 uncommitted files with WIP message
2. Create receipt schema pack (Phase 1)
3. Wire turbo-quant/fib-quant to quant-governor (Phase 3)
4. Fix semantic-memory HNSW bugs (Phase 3)
5. Build compression survivability lab (Phase 3)
6. Implement Gloss Answer Testimony (Phase 5)

---

## Evidence Rule

External benchmark numbers are **source-reported research claims** until reproduced locally with workload-specific receipts. No claim is promoted without local receipt.
