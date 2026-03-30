# attestation-exchange

Typed surface crate for attestation-envelope, trust-root, transparency, and vendor-profile artifacts.

## Usage

```rust
use attestation_exchange::{AttestationEnvelopeV1, TrustRootSetV1, TransparencyReceiptV1};
```

## Owns

- attestation envelopes
- trust-root sets
- transparency receipts
- vendor translation profiles

## Does not own

- network transport
- silent trust promotion
- hidden cross-runtime authority

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (envelope, trust-root, transparency receipt, and disclosure policy IDs)

**Depended on by:**
- `remote-oracle-admission`
- `contract-schema-gen`
- `forge-pilot` (optional, via `governance` feature)

## stack-ids integration

Uses `AttestationEnvelopeId`, `TrustRootSetId`, `TransparencyReceiptId`,
`ArtifactAdmissionPolicyId`, `ContentDigest`, and `DisclosurePolicyId` from
`stack-ids` for all artifact identifiers.
