# attestation-exchange

Typed attestation exchange contracts for envelope, trust-root, and transparency artifacts.

## Example

```rust
use attestation_exchange::VendorCertificationAdapterV1;

let adapter: VendorCertificationAdapterV1 = serde_json::from_str(&json_str)?;
```

## Ecosystem

- **stack-ids**: All artifact IDs (`AttestationEnvelopeId`, `TrustRootSetId`, etc.)
- **remote-oracle-admission**: Consumes attestation-exchange types for oracle admission flows

## stack-ids integration

Fully integrated. All IDs are `stack-ids` newtypes.
