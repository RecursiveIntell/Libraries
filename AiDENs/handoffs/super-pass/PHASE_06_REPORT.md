# PHASE 06 REPORT — Provider honesty

## Scope

- Backlog rows selected: 57 rows, `AHD-0231` through `AHD-0285` plus `CLAUDE-F-014` and `CLAUDE-F-019`, all with `Suggested_Phase = Phase 06 provider honesty`.
- Files/crates touched: `crates/aidens-provider-kit/src/lib.rs`, `crates/aidens-runner/src/lib.rs`, `crates/aidens-runner/src/tests.rs`, `crates/aidens-cli/src/agent.rs`, `matrices/SUPER_PASS_BACKLOG_1020.csv`, `matrices/SUPER_PASS_BACKLOG_1020.json`.
- Non-goals: no cloud provider boundary was promoted; Ollama remains local chat without native tool-loop support.

## Changes made

| Area | Files | Summary |
|---|---|---|
| Local route honesty | `crates/aidens-provider-kit/src/lib.rs` | `kind = local` now resolves to unavailable with `local-provider-boundary-unavailable`; it is not a mock alias in provider construction/readiness/route reporting. |
| Explicit fixture disclosure | `crates/aidens-runner/src/lib.rs` | Existing `AgentProviderModeV1::Local` fixture execution through mock now emits `provider-policy-local-routed-to-explicit-mock-fixture` in route and finalization receipts. |
| Degradation disclosure | `crates/aidens-cli/src/agent.rs` | CLI run-bundle degradation markers include the explicit local-policy/mock-fixture route reason. |
| Hostile fixtures | `crates/aidens-provider-kit/src/lib.rs`, `crates/aidens-runner/src/tests.rs` | Added tests proving provider `local` is unavailable, and runner local-policy fixture routing is receipt-disclosed. |

## Tests/commands run

| Command | Result | Evidence/log path |
|---|---|---|
| `cargo test -p aidens-provider-kit local_provider_is_unavailable_not_mock_alias` | pass | `target/super-pass/audit/phase06-cargo-test-provider-local.log` |
| `cargo test -p aidens-runner phase06_local_policy_mock_fixture_is_receipt_disclosed` | pass | `target/super-pass/audit/phase06-cargo-test-runner-local-disclosure.log` |
| `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` | pass | `target/super-pass/audit/phase06-cargo-test-cli-agent-run.log` |
| `cargo check -p aidens-provider-kit -p aidens-runner -p aidens-cli` | pass | `target/super-pass/audit/phase06-cargo-check-provider-runner-cli.log` |

## Issue matrix updates

| Status | Count | IDs |
|---|---:|---|
| fixed | 57 | `AHD-0231` through `AHD-0285`, `CLAUDE-F-014`, `CLAUDE-F-019` |
| quarantined | 0 |  |
| deferred | 0 |  |
| superseded | 0 |  |
| open-blocking | 0 |  |

## Gate result

- Phase gate: Provider honesty gate.
- Result: Pass for scoped local/mock route honesty. `Local` no longer means mock at provider construction, and legacy local-agent mock fixture execution is explicit and degradation-bearing.
- Remaining risk: Ollama native tool-result loop parity is still not claimed; it remains `ollama-chat` with `ollama-native-tool-loop-unimplemented`.

## Notes for next phase

Phase 07 should focus on queue leasing and completion race behavior.
