# verification-calibration

Canonical verification calibration snapshots and abstention artifacts.

## Usage

```sh
cargo add verification-calibration
```

```rust
use verification_calibration::{CalibrationSnapshot, NuisanceStateArtifact};
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`CalibrationSnapshotId`, `NuisanceStateId`, `RegionId`)
- `verification-control` -- control-plane case types

**Depended on by:**
- `verification-adjudication`
- `contract-schema-gen`
- `forge-pilot`
- `kernel-conformance`

## stack-ids integration

`CalibrationSnapshot` and `NuisanceStateArtifact` use `stack-ids` types for
all identifiers. The `CalibrationSnapshot::evaluate` constructor determines
whether a verification case is forced into advisory-only mode based on
comparability, oracle calibration, risk thresholds, and drift markers.
