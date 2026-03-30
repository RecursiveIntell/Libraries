# continuity-runtime

Typed surface crate for v24 continuity and incident artifacts.

## Usage

```rust
use continuity_runtime::{IncidentCaseV1, RecoveryPlanV1, ServiceLevelProfileV1};
```

## Owns

- service-level profiles and error-budget ledgers,
- incident cases, containment decisions, and forensic freezes,
- recovery plans and replay slices,
- continuity exceptions, postmortems, and resilience exercises.

## Non-goals

- hidden emergency policy changes,
- untyped operator folklore,
- direct deploy approval.

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
