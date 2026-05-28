# Evaluator Reference

The reference evaluator entry point is:

```rust
evaluate_with_policy(input, canonical_policy) -> Result<ControlDecisionReceiptV1, ScrError>
```

Decision order:

1. Validate input shape.
2. Canonicalize and hash input JSON.
3. Use the supplied canonical policy hash.
4. Evaluate hard rules.
5. Apply hard vetoes and minimum action floors.
6. Derive integer score axes and pressures.
7. Resolve actions by deterministic precedence.
8. Emit and validate a decision receipt.

The `evaluate(input)` helper intentionally returns an explicit unavailable
error. This prevents hidden policy defaults; production use must pass a policy
explicitly.
