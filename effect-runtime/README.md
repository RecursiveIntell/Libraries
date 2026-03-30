# effect-runtime

Typed surface crate for v21 live effect intents, preflight, commit, execution, observation, and compensation artifacts.

## Usage

```rust
use effect_runtime::{EffectIntentV1, EffectCommitDecisionV1, CompensationPlanV1};
```

## Owns

- effect intents and windows,
- preflight and commit decisions,
- execution receipts and observation bundles,
- compensation plans and compensation receipts,
- external effect-ledger artifacts.

## Non-goals

- direct domain-truth mutation,
- ambient authority inference,
- release-readiness adjudication,
- incident command.

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (effect intent, window, commit decision, compensation IDs)

**Depended on by:**
- `contract-schema-gen`
- `forge-pilot` (optional, via `governance` feature)

## stack-ids integration

Uses effect-family IDs (`EffectIntentId`, `EffectWindowId`,
`EffectCommitDecisionId`, `EffectExecutionReceiptId`,
`EffectObservationBundleId`, `CompensationPlanId`, etc.) from `stack-ids`.
