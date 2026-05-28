# spec-execution

Typed spec and proof surface crate with bounded generated-artifact evaluators for schema bundles, migration plans, and governance baselines.

## Example

```rust
use spec_execution::{generate_schema_bundle, generate_companion_bundles};

let (schema_bundle, receipt) = generate_schema_bundle(
    &spec, &ast, &obligations, "my-family", "2026-03-14T00:00:00Z",
);
assert!(receipt.admission_allowed);
```

## Ecosystem

- **stack-ids**: All artifact IDs (`SpecBundleId`, `NormativeAstId`, etc.) plus `SurfaceStatus` and `ContentDigest`

## stack-ids integration

Fully integrated. `SurfaceStatus` is re-exported. `ContentDigest::compute_json` is used for schema file hashing.
