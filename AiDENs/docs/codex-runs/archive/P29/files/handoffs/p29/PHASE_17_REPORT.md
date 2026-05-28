# P29 Phase 17 Report

## Phase

Phase 17 - v11B right-graph declarations and misuse tests.

## Scope

Added explicit P29 right-graph misuse coverage for storage graphs and unbounded regions.

## Files changed

- `crates/aidens-contracts/src/tests.rs`
- `handoffs/p29/PHASE_17_REPORT.md`

## Evidence produced

- `p29_right_graph_misuse_is_blocked_for_storage_or_unbounded_regions` verifies the seed graph law blocks storage-as-runtime and over-budget region use.
- Existing P28 reserved/advisory v11B test remains green.

## Tests/checks run

| Command | Result | Log |
|---|---|---|
| `cargo test -p aidens-contracts p29_ -- --nocapture` | pass | `target/p29/audit/phase19_aidens_contracts_p29_v11b_tests.log` |
| `cargo test -p aidens-contracts p28_v11b -- --nocapture` | pass | `target/p29/audit/phase17_aidens_contracts_p28_v11b_tests.log` |

## Claims changed

v11B right-graph support remains executable seed/advisory only.

## Risks / limitations

The graph seed does not activate a full regional runtime.

## Gate status

- [x] pass
- [ ] fail

## Next phase notes

Proceed to region boundary message/receipt seed.
