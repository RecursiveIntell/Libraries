# knowledge-runtime

Bounded orchestration scaffold for semantic-memory: classification, routing, scoped entity resolution, provenance-preserving merge, and projection status tracking.

## Usage

```rust
use knowledge_runtime::{KnowledgeRuntime, QueryResult, QueryTrace};
```

## What this crate is for

This crate provides intent classification, route planning, entity resolution,
result merging, and projection lifecycle tracking on top of `semantic-memory`.
It never owns source truth; all facts, episodes, documents, and messages live
in `semantic-memory`.

## Non-goals

- do not let local convenience APIs become de facto truth surfaces
- prefer explicit versioned contracts over drift-by-example

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`EntityId`, `ScopeKey`, `TraceCtx`, `ClaimId`, `ClaimVersionId`)
- `semantic-memory` -- durable projected/queryable truth store
- `constraint-compiler`, `forge-memory-bridge`, `kernel-execution`, `kernel-oracles`
- `recursive-kernel-core`

**Depended on by:**
- `forge-pilot`
- `kernel-conformance`
- `contract-schema-gen`

## stack-ids integration

Uses `EntityId`, `ScopeKey`, `TraceCtx`, `ClaimId`, and `ClaimVersionId` from
`stack-ids` for entity resolution, scope partitioning, and query tracing.
