# semantic-memory-forge

Forge verification truth: evidence bundles, export envelopes, and causal estimation substrate.

## Usage

```rust
use semantic_memory_forge::{ExportEnvelopeV3, EvidenceBundle, EvidenceBundleId};
```

## What this crate is for

`semantic-memory-forge` owns raw verification truth and the export envelope
contract. It provides evidence bundle schemas, causal estimation metadata, and
refutation structures for the canonical verification pipeline.

## Authority boundary

- Forge is authoritative for raw verification truth and export envelopes.
- Memory (`semantic-memory`) is authoritative for queryable projected truth.
- Bridge (`forge-memory-bridge`) is authoritative only for transformation.
- Runtime (`knowledge-runtime`) is authoritative only for query planning/merge.

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`EnvelopeId`, `ClaimId`, `EvidenceBundleId`, `TraceCtx`, `ContentDigest`, etc.)

**Depended on by:**
- `forge-memory-bridge`
- `semantic-memory` (dev-dependency)
- `constraint-compiler`
- `verification-control`
- `llm-tool-runtime`
- `forge-engine` (living-memory)
- `contract-schema-gen`
- `kernel-oracles` (dev-dependency)

## stack-ids integration

Uses `EnvelopeId`, `ClaimId`, `ClaimVersionId`, `EntityId`, `EpisodeId`,
`TraceCtx`, `ContentDigest`, `EvidenceBundleId`, and many more typed IDs
from `stack-ids` for all wire-visible artifact identifiers.
