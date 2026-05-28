# P29 Phase 14 Report

## Phase

Phase 14 - Supported-local agent path v11A conformance.

## Scope

Validated the `AgentSpec`/config to local runner/tool/receipt/report path for `run-coding-agent`.

## Files changed

- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/tests.rs`
- `handoffs/p29/PHASE_14_REPORT.md`

## Evidence produced

- Existing `run_coding_agent` tests now assert the v11A evidence block includes artifact envelope, execution context, operator contract, manifests, completion gate, semantic state, and view disclosure.
- The completion gate reports `complete` only when tool receipt refs exist and proof debt does not block promotion.
- The report still blocks unapproved write/check actions and records approval requests rather than silently completing them.

## Tests/checks run

| Command | Result | Log |
|---|---|---|
| `cargo test -p aidens-cli run_coding_agent -- --nocapture` | pass | `target/p29/audit/phase14_aidens_cli_run_coding_agent_tests.log` |
| `python3 scripts/assert_p29_receipt_chain.py` | pass | `target/p29/audit/phase14_assert_p29_receipt_chain.log` |
| `cargo check --workspace --all-targets` | pass | `target/p29/audit/phase14_aidens_cargo_check.log` |

## Claims changed

The declared supported-local coding-agent path has local release-candidate evidence. Full pass completion remains blocked on later v11B seed phases, final command bar, package generation, and extracted replay.

## Risks / limitations

This phase does not claim production-cloud readiness, broad autonomy, or v11B/v11C completion.

## Gate status

- [x] pass
- [ ] fail

## Next phase notes

Proceed to Phase 15 containment and ownership cleanup, then stop for the manual gate.
