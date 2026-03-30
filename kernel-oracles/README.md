# kernel-oracles

Bounded exact, conservative, temporal replay, and refutation paths for the recursive inference kernel.

## Usage

```rust
use kernel_oracles::{OracleAssessment, OracleMode, OracleParityCertificate};
```

## Scope

This crate evaluates supported oracle slices, delta parity, temporal replay,
and bounded refutation/minimal-perturbation behavior over compiled kernel
artifacts.

## Non-goals

- no string-time ordering shortcuts
- no unbounded search
- no authority over truth or promotion

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`ContentDigest`, `OracleSliceId`, `RegionId`)
- `constraint-compiler` -- compiled graph artifacts
- `forge-memory-bridge` -- projection import batch types
- `kernel-execution` -- execution baselines for oracle evaluation
- `recursive-kernel-core` -- kernel operator metadata

**Depended on by:**
- `kernel-conformance`
- `knowledge-runtime`
- `forge-pilot`
- `contract-schema-gen`

## stack-ids integration

Uses `ContentDigest`, `OracleSliceId`, and `RegionId` from `stack-ids` for
oracle slice identification and content addressing.
