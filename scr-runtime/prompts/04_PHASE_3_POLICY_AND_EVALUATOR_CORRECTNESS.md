# Phase 3 — Policy and Evaluator Correctness

## Objective

Close semantic holes in policy evaluation and decision receipts.

## Required actions

1. Define a supported hard-rule registry, e.g. `SUPPORTED_HARD_RULE_IDS`.
   - Policy validation must reject any hard rule not in the registry.
   - Evaluation must check every enabled declared supported hard rule or fail.
2. Define a supported minimum-action/signal registry.
   - Unknown minimum-action keys must fail unless explicitly fixture-only and documented.
3. Enforce policy header compatibility:
   - domain must equal the SCR runtime domain;
   - algorithm version must match `EVALUATOR_ALGORITHM_ID` or an explicit compatibility table;
   - canonicalization profile must match the implemented canonicalization profile.
4. Stop opaque-ref token scanning.
   - `SignalSet::from_input` must not tokenize `input_id`, actor refs, permit refs, subject refs, environment refs, or arbitrary evidence refs.
   - Only explicit `ref_kind == "signal"` fixture signals or typed upstream adapter outputs may produce SCR signals.
5. Fix invalid input behavior.
   - Valid typed API must reject invalid input before normal evaluation, or route invalid raw input through a dedicated rejection/quarantine receipt.
   - Do not substitute empty time fields with synthetic strings in normal receipts.
6. Remove or replace `safe_ref` behavior that converts malformed refs to synthetic refs.
   - Preserve original invalid data in a parse/rejection artifact if raw input path exists.
7. Fix `evaluator_algorithm_hash`.
   - Either rename to `evaluator_algorithm_id_hash`, or compute a real source/build digest with honest semantics.
8. Fix candidate arbitration receipts.
   - Record every candidate considered.
   - Record every losing candidate and reason it lost: lower kind, lower precedence, same-action subsumed, hard-veto override, floor override, score override.
9. Fix the public `evaluate()` API.
   - Remove unusable API, or make explicit-policy evaluation the only public path.
10. Add tests for each above behavior.

## Acceptance gate

```bash
cargo test --workspace --all-targets
bash scripts/assert_no_opaque_signal_scanning.sh
```
