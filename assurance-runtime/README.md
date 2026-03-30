# assurance-runtime

Typed surface crate for v23 deployability and certification artifacts.

## Usage

```rust
use assurance_runtime::{AssuranceCaseV1, CertificationBundleV1, DeploymentProfileV1};
```

## Owns

- deployment profiles and operating envelopes,
- assurance cases, hazard registers, and control mappings,
- residual-risk acceptance and release-readiness decisions,
- field monitoring and certification bundles.

## Non-goals

- score-only release approval,
- automatic truth promotion,
- incident command.

## Ecosystem

**Depends on:** (no sibling crate path dependencies)

**Depended on by:**
- `contract-schema-gen`
- `forge-pilot` (optional, via `governance` feature)
- `profile-runtime`

## stack-ids integration

This crate does not depend on `stack-ids` directly. It provides typed schema
surfaces that are composed by higher-level crates which do carry stack-ids
identity.
