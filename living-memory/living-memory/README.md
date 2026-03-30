# forge-engine

Causal edit attribution and structured patch evaluation engine.

## Usage

```rust
use forge_engine::{ForgeConfig, ForgeError, MindState};
```

## What this crate is for

`forge-engine` owns the execution-heavy lane: compile mindstate, validate and
apply structured patches, run checks, score candidate outcomes, persist
operational evidence, and update causal edit-attribution state.

## Authority boundary

- `semantic-memory-forge` owns raw verification/export wire truth.
- `forge-memory-bridge` owns transformation only.
- `semantic-memory` owns durable projected/queryable truth.
- `forge-engine` owns operational verification work on top of those crates.

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`EnvelopeId`, `ClaimVersionId`, `AttemptId`, `TrialId`, etc.)
- `semantic-memory`, `semantic-memory-forge`, `forge-memory-bridge`
- `llm-tool-runtime`

**Depended on by:**
- `forge-pilot`
- `kernel-conformance` (dev-dependency)

## stack-ids integration

Uses `EnvelopeId`, `ClaimVersionId`, `RelationVersionId`, `AttemptId`,
`TrialId`, and `ContentDigest` from `stack-ids` for export provenance,
causal attribution, and content addressing.
