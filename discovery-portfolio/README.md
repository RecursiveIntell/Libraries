# discovery-portfolio

Typed surface crate for v18 discovery-program, campaign-selection, and review-budget artifacts.

## Usage

```rust
use discovery_portfolio::{DiscoveryProgramV1, PortfolioPlanV1, ExperimentCampaignV1};
```

## Owns

- discovery programs
- program hypothesis sets
- experiment campaigns
- information value estimates
- portfolio plans
- verification-load budgets
- campaign decision traces

## Does not own

- ground-truth mutation
- a full execution scheduler
- autonomous experiment promotion

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (`DiscoveryProgramId`, `ExperimentCampaignId`, `PortfolioPlanId`, etc.)

**Depended on by:**
- `kernel-conformance`
- `contract-schema-gen`

## stack-ids integration

Uses `DiscoveryProgramId`, `ExperimentCampaignId`, `InformationValueEstimateId`,
`PortfolioPlanId`, `VerificationLoadBudgetId`, `CampaignDecisionTraceId`, and
`SurfaceStatus` from `stack-ids`.
