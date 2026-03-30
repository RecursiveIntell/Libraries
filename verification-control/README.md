# verification-control

Canonical verification control-plane case, plan, receipt, and ledger artifacts.

## Usage

```sh
cargo add verification-control
```

```rust
use verification_control::{
    VerificationCase, CheckPlan, ControlReceipt, VerificationAttempt,
    PromotionClass, TerminalDisposition,
};
```

## Ecosystem

**Depends on:**
- `stack-ids` -- identity primitives (case, plan, receipt, region, and effect IDs)
- `llm-tool-runtime` -- tool receipt and retry-owner types
- `semantic-memory-forge` -- exactness budgets, evidence admissibility, degradation kinds

**Depended on by:**
- `verification-calibration`
- `verification-policy`
- `verification-adjudication`
- `contract-schema-gen`
- `forge-pilot`
- `kernel-conformance`

## stack-ids integration

This crate makes heavy use of `stack-ids` for all artifact identifiers
including `VerificationCaseId`, `CheckPlanId`, `ControlReceiptId`,
`LedgerEntryId`, `TraceCtx`, and many more. The `V25CitationContext`
struct embeds optional constitutional-context IDs from `stack-ids`.
