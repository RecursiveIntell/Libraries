# remote-oracle-admission

Typed remote oracle admission contracts for lease, result, replay, and re-admission artifacts.

## Example

```rust
use remote_oracle_admission::{RemoteOracleLeaseV1, V25ConstitutionCitation};

let lease = RemoteOracleLeaseV1::new(
    lease_id, "oracle:trusted-remote", citation,
    &["artifact:v1"], trust_root_set_id,
)?;
lease.validate()?;
```

## Ecosystem

- **stack-ids**: All artifact IDs plus `V25ConstitutionCitation`
- **attestation-exchange**: Provides attestation types used in admission flows

## stack-ids integration

Fully integrated. `V25ConstitutionCitation` is re-exported from `stack-ids`.
