# mechanism-runtime

Typed surface crate for v17 mechanism, theory, simulation, fit, and refuter/stability artifacts.

## Usage

```rust
use mechanism_runtime::{MechanismBundleV1, TheoryVersionV1, FitRunV1};
```

## Owns

- mechanism bundles
- theory versions and libraries
- simulation contracts
- fit runs
- theory refuter suites
- rollout stability reports

## Does not own

- autonomous search/planning
- portfolio scheduling
- authoritative promotion

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`MechanismBundleId`, `TheoryVersionId`, `FitRunId`, `SimulationContractId`, etc.)

**Depended on by:**
- `kernel-conformance`
- `contract-schema-gen`
- `forge-pilot` (optional, via `governance` feature)

## stack-ids integration

Uses `MechanismBundleId`, `TheoryVersionId`, `TheoryLibraryId`,
`HypothesisLibraryId`, `SimulationContractId`, `FitRunId`,
`TheoryRefuterSuiteId`, `RolloutStabilityReportId`, and `SurfaceStatus`
from `stack-ids`.
