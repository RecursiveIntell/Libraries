# recursive-kernel-core

Non-authoritative recursive inference kernel contracts.

## Usage

```rust
use recursive_kernel_core::{OperatorMetadata, ConstraintUnit, ExactnessClass, StopRule};
```

## Scope

This crate owns the shared kernel-facing schemas and operator metadata used by
the compiler, execution, and oracle crates.

## Non-goals

- no truth ownership
- no persistence
- no runtime planning
- no heuristic promotion of artifacts into authority

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`ConstraintId`, `KernelRunId`, `OperatorId`, `OracleSliceId`, `ScopeKey`, `SyndromeId`)

**Depended on by:**
- `constraint-compiler`
- `kernel-execution`
- `kernel-oracles`
- `kernel-conformance`
- `knowledge-runtime`
- `forge-pilot`

## stack-ids integration

Uses `ConstraintId`, `KernelRunId`, `OperatorId`, `OperatorVersionId`,
`OracleSliceId`, `ScopeKey`, and `SyndromeId` from `stack-ids` for operator
identification and kernel run tracking.
