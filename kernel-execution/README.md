# kernel-execution

Deterministic bounded execution baseline for recursive inference graphs.

## Usage

```rust
use kernel_execution::{
    execute_message_passing_baseline, execute_delta_propagation,
    ExecutionReport, ExecutionMode, ExecutionStopReason,
};
```

## Scope

This crate owns the acyclic baseline, message-passing baseline, delta
propagation, residual correction, and scheduler behavior for compiled kernel
graphs.

## Non-goals

- no authority promotion
- no hidden best effort when changed-node input is missing
- no compensation for compiler or oracle uncertainty

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`ContentDigest`, `ConvergenceReportId`, `RegionId`, `SyndromeId`)
- `constraint-compiler` -- compiled graph artifacts
- `recursive-kernel-core` -- authority classes and syndrome types

**Depended on by:**
- `kernel-oracles`
- `kernel-conformance`
- `knowledge-runtime`
- `forge-pilot`
- `contract-schema-gen`

## stack-ids integration

Uses `ContentDigest`, `ConvergenceReportId`, `RegionId`, and `SyndromeId`
from `stack-ids` for execution report identifiers and region tracking.
