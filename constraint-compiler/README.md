# constraint-compiler

Deterministic projection-to-inference graph compiler for the canonical lane.

## Usage

```rust
use constraint_compiler::{compile_batch, CompileOutput, CompilerPolicy};
```

## Scope

This crate compiles canonical projection import batches into bounded inference
graph artifacts: nodes, hyperedges, constraints, invalidation cones,
degradations, and oracle candidates.

## Non-goals

- no authority over source truth
- no runtime repair or scheduler policy
- no fabrication of semantics from thin exports

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`ConstraintId`, `ContentDigest`, `OracleSliceId`, `RegionId`, `ScopeKey`)
- `forge-memory-bridge` -- projection import batch types
- `recursive-kernel-core` -- kernel operator metadata and constraint units
- `semantic-memory-forge` -- constraint seed kinds and export record semantics

**Depended on by:**
- `kernel-execution`
- `kernel-oracles`
- `kernel-conformance`
- `knowledge-runtime`
- `forge-pilot`
- `contract-schema-gen`

## stack-ids integration

Uses `ConstraintId`, `ContentDigest`, `OracleSliceId`, `RegionDigestId`,
`RegionId`, and `ScopeKey` from `stack-ids` for all compiler artifact
identifiers and content addressing.
