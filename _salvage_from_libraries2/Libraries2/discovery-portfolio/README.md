# discovery-portfolio

Typed discovery portfolio surface crate with bounded budget and selection evaluators for experiment campaigns.

## Example

```rust
use discovery_portfolio::{evaluate_portfolio_plan, CampaignDecision};

let trace = evaluate_portfolio_plan(
    &program, &hypotheses, &plan,
    &campaigns, &value_estimates, &budget,
    "2026-03-14T00:00:00Z",
);

for line in &trace.decisions {
    match line.decision {
        CampaignDecision::Launch => println!("Launching: {}", line.campaign_id),
        CampaignDecision::PauseBudgetExhausted => println!("Paused: budget exhausted"),
        CampaignDecision::Defer => println!("Deferred: {}", line.rationale),
    }
}
```

## Ecosystem

- **stack-ids**: `DiscoveryProgramId`, `ExperimentCampaignId`, `SurfaceStatus`, and other typed IDs

## stack-ids integration

Fully integrated. All artifact IDs are `stack-ids` newtypes. `SurfaceStatus` is re-exported from `stack-ids`.
