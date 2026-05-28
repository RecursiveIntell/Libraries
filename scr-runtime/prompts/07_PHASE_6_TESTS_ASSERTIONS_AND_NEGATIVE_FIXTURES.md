# Phase 6 — Tests, Assertions, and Negative Fixtures

## Objective

Convert the hard audit into executable regression protection.

## Required tests

Add Rust unit/integration tests for:

1. unknown hard rule rejected;
2. missing supported hard rule rejected if policy requires full hard-rule set;
3. wrong policy domain rejected;
4. wrong algorithm version rejected;
5. wrong canonicalization rejected;
6. opaque refs do not trigger control signals;
7. explicit fixture signal refs do trigger control signals;
8. invalid typed input is rejected or raw input produces rejection receipt;
9. malformed refs are not silently replaced;
10. `ScoreBps` and `WeightBps` reject >10000;
11. unknown JSON fields fail deserialization;
12. generated schema contains max bounds and closed objects;
13. rejected/candidate action receipts include all losing candidates;
14. `explain-receipt` does not require input/policy re-evaluation;
15. fixture verification fails on changed output;
16. schema generation drift is detected.

## Required scripts/gates

All must pass:

```bash
python3 scripts/validate_strict_schemas.py
python3 scripts/assert_existing_crate_boundaries.py
python3 scripts/assert_no_stale_surfaces.py
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_unexplained_golden_changes.sh
```

## Acceptance gate

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
