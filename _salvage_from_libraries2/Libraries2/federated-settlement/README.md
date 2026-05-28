# federated-settlement

Typed treaty and settlement surface crate with bounded shared-view evaluators for cross-runtime settlement, replay, divergence, and suspension artifacts.

## Example

```rust
use federated_settlement::{evaluate_settlement, SettlementDisposition};

let receipt = evaluate_settlement(&case, "2026-03-14T00:00:00Z");
match receipt.disposition {
    SettlementDisposition::SharedDispositionIssued => println!("Settlement lawful"),
    SettlementDisposition::DegradedSharedView => println!("Degraded: {}", receipt.downgrade.unwrap().detail),
    SettlementDisposition::AdvisoryOnly => println!("Advisory only"),
}
```

## Ecosystem

- **stack-ids**: All artifact IDs plus `SurfaceStatus` and `V25ConstitutionCitation`

## stack-ids integration

Fully integrated. `SurfaceStatus` and `V25ConstitutionCitation` are re-exported from `stack-ids`.
