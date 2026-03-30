# verification-adjudication

Canonical verification disposition, promotion, refutation, and rollback decisions.

## Usage

```sh
cargo add verification-adjudication
```

```rust
use verification_adjudication::{
    EffectAdjudicationReceiptV1, ReleaseRollbackDecisionV1,
    RollbackDecisionV1, RolloutDecisionV1,
};
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives
- `verification-calibration` -- calibration snapshots used during adjudication
- `verification-control` -- control-plane cases, plans, and receipts
- `verification-policy` -- policy decisions and citation context

**Depended on by:**
- `contract-schema-gen`
- `forge-pilot`
- `kernel-conformance`

## stack-ids integration

Adjudication receipt and decision IDs (`EffectAdjudicationReceiptId`,
`PromotionDecisionId`, `RefutationDecisionId`, `RollbackPlanId`, etc.) are
sourced from `stack-ids`.
