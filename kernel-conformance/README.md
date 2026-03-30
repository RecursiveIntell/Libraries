# kernel-conformance

Cross-crate conformance harness for the canonical export -> bridge -> import -> runtime -> compile -> oracle lane.

## Usage

```rust
use kernel_conformance::{phase_1_gates, phase_2_gates, phase_3_plus_gates};
```

## Scope

This crate proves the supported kernel lane through deterministic fixtures
spanning authority boundaries, degradation behavior, replay, refutation, and
living-memory roundtrips.

## Non-goals

- no production authority
- no replacement for crate-local unit tests
- no benchmark or satellite policy decisions

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives
- `constraint-compiler`, `forge-memory-bridge`, `kernel-execution`, `kernel-oracles`
- `recursive-kernel-core`, `semantic-memory`, `semantic-memory-forge`
- `constitutional-memory`, `discovery-portfolio`, `federated-settlement`
- `knowledge-runtime`, `mechanism-runtime`, `spec-execution`
- `verification-adjudication`, `verification-calibration`, `verification-control`, `verification-policy`

**Depended on by:** (leaf crate -- no downstream consumers)

## stack-ids integration

Uses `ScopeKey`, `TraceCtx`, `EnvelopeId`, `ClaimId`, and other identity
primitives from `stack-ids` in conformance test fixtures.
