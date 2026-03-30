# forge-pilot

Closed-loop orchestrator that sits on top of Forge export, bridge/import, runtime advisories, and kernel oracles.

## Usage

```rust
use forge_pilot::{execute_plan, ActionOutcome, LoopConfig, PatchPlanSeed};
```

## Scope

This crate scores targets, builds plans, performs canonical roundtrips, and
records bounded loop execution without taking ownership of truth.

## Non-goals

- no direct authority writes as the normal path
- no compensation for unresolved kernel uncertainty
- no promotion of pilot output into supported truth on its own

## Running

```bash
cargo run -p forge-pilot --
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`AttemptId`, `TraceCtx`, `TrialId`, `ClaimVersionId`, etc.)
- `constraint-compiler`, `forge-memory-bridge`, `kernel-execution`, `kernel-oracles`
- `recursive-kernel-core`, `semantic-memory`, `semantic-memory-forge`
- `knowledge-runtime`, `forge-engine` (living-memory)
- `verification-adjudication`, `verification-calibration`, `verification-control`, `verification-policy`
- Optional governance: `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `constitutional-memory`, `continuity-runtime`, `effect-runtime`, `mechanism-runtime`

**Depended on by:**
- `contract-schema-gen`
- `kernel-conformance` (dev-dependency)

## stack-ids integration

Uses `AttemptId`, `TraceCtx`, `TrialId`, `EnvelopeId`, `ScopeKey`,
`ClaimVersionId`, `ContentDigest`, and `OracleSliceId` from `stack-ids` for
loop execution tracing, content addressing, and target identification.
