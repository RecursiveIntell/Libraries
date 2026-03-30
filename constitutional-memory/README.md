# constitutional-memory

Typed surface crate for v19 charter, amendment, archive, compaction, deprecation, and retirement artifacts.

## Usage

```rust
use constitutional_memory::{CharterBundleV1, AmendmentProposalV1, ArchiveManifestV1};
```

## Owns

- charter bundles
- doctrine snapshots
- amendment proposals and decisions
- archive manifests
- compaction receipts
- historical query guarantees
- deprecation and retirement bundles

## Does not own

- informal repo notes as constitutional truth
- hidden governance side agreements
- irreversible constitutional change with no rollback story

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (charter, amendment, archive, compaction, and deprecation IDs)

**Depended on by:**
- `kernel-conformance`
- `contract-schema-gen`
- `forge-pilot` (optional, via `governance` feature)

## stack-ids integration

Uses `CharterBundleId`, `DoctrineSnapshotId`, `AmendmentProposalId`,
`AmendmentDecisionId`, `ArchiveManifestId`, `CompactionReceiptId`,
`DeprecationBundleId`, `RetirementBundleId`, and `HistoricalQueryGuaranteeId`
from `stack-ids`.
