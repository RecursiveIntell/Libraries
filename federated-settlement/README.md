# federated-settlement

Typed surface crate for v16 treaty, equivalence, settlement, replay, divergence, and suspension artifacts.

## Usage

```rust
use federated_settlement::{TreatyBundleV1, SettlementCaseV1, SharedDispositionV1};
```

## Owns

- treaty artifacts
- runtime identity/equivalence artifacts
- settlement case/receipt artifacts
- local dissent and downgrade artifacts
- bounded shared replay / divergence / suspension artifacts

## Does not own

- network transport
- trust-root exchange infrastructure
- authoritative truth mutation
- hidden cross-runtime promotion

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (treaty, settlement, dissent, replay, divergence, and suspension IDs)

**Depended on by:**
- `kernel-conformance`
- `contract-schema-gen`

## stack-ids integration

Uses `TreatyBundleId`, `SettlementCaseId`, `SettlementReceiptId`,
`SharedDispositionId`, `LocalDissentId`, `SharedReplaySliceId`,
`SharedDivergenceReportId`, `TreatySuspensionId`, and others from `stack-ids`.
Also re-exports `SurfaceStatus` and `V25ConstitutionCitation`.
