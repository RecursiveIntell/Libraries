# authority-delegation

Typed surface crate for v22 delegated-authority artifacts.

## Usage

```rust
use authority_delegation::{CapabilityClassV1, AuthorityLeaseV1, DelegationBundleV1};
```

## Owns

- capability classes and authority leases,
- delegation bundles and authority chains,
- separation-of-duties and dual-control artifacts,
- break-glass, revocation, and acting-on-behalf receipts.

## Non-goals

- ambient privilege,
- direct truth promotion,
- silent treaty or policy bypass.

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
