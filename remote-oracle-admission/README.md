# remote-oracle-admission

Typed remote oracle admission contracts for lease, result, replay, and re-admission artifacts.

## Usage

```sh
cargo add remote-oracle-admission
```

```rust
use remote_oracle_admission::{RemoteOracleLeaseV1, RemoteSliceRequestV1, RemoteSliceResultV1};
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (lease, slice, replay, and revocation IDs)
- `attestation-exchange` -- attestation envelope types

**Depended on by:**
- `contract-schema-gen`

## stack-ids integration

All artifact identifiers (`RemoteOracleLeaseId`, `RemoteSliceRequestId`,
`RemoteSliceResultId`, `CrossRuntimeReplayTicketId`, etc.) are sourced from
`stack-ids`. The `V25ConstitutionCitation` type is re-exported directly.
