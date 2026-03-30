# 05. Crate Authority Map

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        ORCHESTRATION LAYER                              │
│                                                                         │
│  forge-pilot (13K lines)                                                │
│    OODA loop, observation, orient, decide, act, governance gate,        │
│    receipts, bootstrap, repo-chat, export/import                        │
│    Depends on: everything below                                         │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        EXECUTION ENGINE                                  │
│                                                                         │
│  living-memory/forge-engine (16K lines)                                 │
│    Patches, checks, scoring, CEA, experiments, tool receipts            │
│    Depends on: semantic-memory, forge-memory-bridge, llm-tool-runtime,  │
│                Primitives/*                                             │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        KNOWLEDGE LAYER                                   │
│                                                                         │
│  knowledge-runtime (10.8K lines)                                        │
│    Query pipeline, entity resolution, projection lifecycle,             │
│    temporal claims, inference advisory, runtime views                   │
│    Depends on: semantic-memory, forge-memory-bridge, constraint-compiler,│
│                kernel-execution, kernel-oracles, recursive-kernel-core   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        VERIFICATION LAYER                                │
│                                                                         │
│  verification-control (3.1K)  verification-policy (2.6K)                │
│  verification-adjudication (1.4K)  verification-calibration (352)       │
│    Cases, check plans, ledger, adjudication, calibration                │
│    Depend on: llm-tool-runtime, semantic-memory-forge, stack-ids        │
│                                                                         │
│  constraint-compiler (1.4K)                                             │
│    Inference graphs, constraint compilation                             │
│    Depends on: forge-memory-bridge, recursive-kernel-core,              │
│                semantic-memory-forge, stack-ids                          │
│                                                                         │
│  kernel-execution (1.2K)  kernel-oracles (1K)                           │
│  kernel-conformance (2.8K)  recursive-kernel-core (583)                 │
│    Execution scheduling, oracle evaluation, conformance testing         │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        GOVERNANCE SCHEMA LAYER                           │
│                                                                         │
│  effect-runtime (2.3K)     - Effect lifecycle artifacts + validators    │
│  assurance-runtime (1.1K)  - Assurance case, certification, deployment  │
│  continuity-runtime (1.2K) - Incident, recovery, SLO, error budget     │
│  mechanism-runtime (601)   - Mechanism bundles, theory, fit runs        │
│  authority-delegation (901) - Delegation chains, approval, leases       │
│  attestation-exchange (804) - Vendor trust, certification adapters      │
│  constitutional-memory (656) - Amendments, effective constitutions      │
│  federated-settlement (617) - Settlement protocols                      │
│                                                                         │
│  profile-runtime (4K)      - Constitutional composition engine          │
│    REAL LOGIC: adapters.rs projects all governance profiles into         │
│    ObligationContributionV1 stream. The composition pipeline.           │
│    Depends on: all governance schema crates above + verification-policy │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        STORAGE + BRIDGE LAYER                            │
│                                                                         │
│  semantic-memory (29K lines) — THE GRAVITY WELL                         │
│    SQLite WAL, connection pool, BM25+FTS5, HNSW sidecar, embedder,     │
│    graph view, chunker, quantization, compaction, migrations            │
│    Depends on: stack-ids, forge-memory-bridge, semantic-memory-forge    │
│                                                                         │
│  semantic-memory-forge (export wire truth)                              │
│  forge-memory-bridge (3.4K) (transformation only)                      │
│  llm-tool-runtime (4K) (tool dispatch contracts)                        │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                        FOUNDATION LAYER                                  │
│                                                                         │
│  stack-ids (218 typed IDs, SurfaceStatus, ScopeKey, TraceCtx, Digest)  │
│    ZERO dependencies on other workspace crates                          │
│                                                                         │
│  Primitives/ (excluded from workspace, compiled standalone)             │
│    cea-core, cea-sqlite, cea-store, check-runner, effect-signature,     │
│    forge-policy, mindstate-core, sandbox-workspace, stabilizer-core,    │
│    typed-patch                                                          │
│                                                                         │
│  discovery-portfolio, remote-oracle-admission, spec-execution,          │
│  contract-schema-gen                                                    │
│    Satellite crates for schema generation and portfolio management      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Authority Rules

| Domain | Owner Crate | Rule |
|--------|-------------|------|
| Typed IDs | `stack-ids` | All opaque ID newtypes defined here. No crate may define its own ID type. |
| Surface status | `stack-ids` | Single `SurfaceStatus` enum. No crate may redefine locally. |
| Durable storage | `semantic-memory` | All persistent records live in SQLite. No other crate writes to disk. |
| Export wire truth | `semantic-memory-forge` | Canonical export/import envelope schemas defined here. |
| Transformation | `forge-memory-bridge` | Bridge transform logic only. No source truth, no storage. |
| Query pipeline | `knowledge-runtime` | Classification, planning, execution, merge. No source truth. |
| OODA orchestration | `forge-pilot` | Loop control, observation, decision. Reads from everything, writes through store. |
| Operational execution | `living-memory/forge-engine` | Patches, checks, scoring, CEA. Writes evidence through store. |
| Governance schemas | Respective `*-runtime` crates | Own their artifact families. Do NOT own runtime behavior. |
| Constitutional composition | `profile-runtime` | Owns the obligation composition pipeline via `adapters.rs`. |
| Verification cases | `verification-control` | Cases, check plans, ledger. |
| Verification policy | `verification-policy` | Policy evaluation, approval matching. |

## Governance Integration Seam (V28 Focus)

The critical path for GOV-001/GOV-002 is:

```
governance schema crates
    ↓ (define artifact types)
profile-runtime/adapters.rs
    ↓ (project into ObligationContributionV1)
knowledge-runtime/views.rs
    ↓ (runtime views — currently dead)
forge-pilot/governance_gate.rs
    ↓ (observe → gate → receipt)
forge-pilot/loop_runner.rs
    ↓ (populate governance_receipt, honor gate result)
```

The architecture exists. The wiring does not. V28 connects these layers.
