# P30-01 Report

## Scope

Phase slice: executable tool-call parser boundary and strict structured-output law in `crates/aidens-runner`.

Issue IDs addressed from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- `P30-ABSORB-0002`: malformed parser-fallback tool-call entries must not be silently dropped.
- `P30-ABSORB-0003`: tool-result serialization must not silently become an empty provider message.
- `P30-ABSORB-0004`: successful parser fallback must carry degradation evidence into downstream receipts.
- `P30-ABSORB-0005`: executable tool-call fallback must not use permissive JSON repair.

Related issue quarantined:

- `P30-ABSORB-0029`: substring-based tool-call detection remains. Attempted local blocking in `NoTools` mode caused permit/approval receipt regressions because `NoTools` also represents registered tools awaiting explicit approval/permit. This should be handled with a richer mode or exposure-state distinction, not a parser-only change.

## Changed Files

- `crates/aidens-runner/src/tests.rs`
  - Added `parser_fallback_valid_call_carries_degradation_on_request`.
  - Added `completion_request_serializes_tool_results_without_empty_substitution`.
  - Added `empty_tool_exposure` fixture helper.

Observed existing implementation evidence:

- `crates/aidens-runner/src/provider_tool.rs:17` maps tool-result serialization errors to an error instead of `unwrap_or_default`.
- `crates/aidens-runner/src/provider_tool.rs:89` uses `BoundaryRepairPolicyV1::default()` for executable parser fallback.
- `crates/aidens-runner/src/provider_tool.rs:146` iterates malformed calls explicitly and records rejected-call reason strings with raw digests.
- `crates/aidens-runner/src/provider_tool.rs:165` attaches parser-fallback reason codes to `ToolCallRequestV1`.

## Tests Added Or Updated

- `parser_fallback_valid_call_carries_degradation_on_request`: proves valid parser-fallback calls are degraded and carry `parser-fallback-tool-call` plus `parser-fallback-degraded`.
- `completion_request_serializes_tool_results_without_empty_substitution`: proves tool result content is serialized into the provider request and is not empty.

Existing relevant tests retained:

- `parser_fallback_blocks_repaired_tool_call_payloads`
- `parser_fallback_rejects_malformed_entries_without_dropping_them`

## Commands Run

- `cargo test --manifest-path Cargo.toml -p aidens-runner parser_fallback -- --nocapture`
  - Result: pass, 3 parser-fallback unit tests passed.
- `cargo test --manifest-path Cargo.toml -p aidens-runner completion_request_serializes_tool_results_without_empty_substitution -- --nocapture`
  - Result: pass, 1 targeted unit test passed.
- `cargo check --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `cargo test --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass, 38 unit tests and 8 integration tests passed for `aidens-runner`.
- `python3 scripts/p30_guard.py --repo .`
  - Result: exit 0, `findings=1836 hard=0`.

Command note: one malformed Cargo invocation was attempted with two test filters in one command; Cargo rejected it before running tests. The tests were then run separately and passed.

## Unresolved Risks And Quarantines

- `P30-ABSORB-0029` remains unresolved. The current `looks_like_tool_call_payload` fallback still uses quoted-marker detection after strict JSON parse fails. A safe fix needs to preserve approval/permit request generation for registered but unexposed tools.
- `p30_guard.py` reports broad existing warning debt across tests, contracts, integration tests, and scaffold code. It reports no hard failures for this phase.
- This phase did not run full workspace test/clippy/doc gates. Evidence is limited to the touched crate plus the P30 guard.

## Invariant Revalidation Checklist

- No `filter_map` remains in `provider_tool.rs` tool-call extraction.
- No `unwrap_or_default` remains in `provider_tool.rs` tool-result serialization.
- Parser fallback uses strict/default boundary policy, not permissive degraded repair.
- Malformed parser-fallback entries produce blocking degradation reason evidence.
- Valid parser-fallback entries carry degraded request-level reason codes.
- Tool-result serialization produces provider tool message content or returns an error.
- Approval/permit blocking behavior remains intact after reverting parser-layer no-tools blocking.

## Proceed Statement

P30-01 P0 parser-boundary blockers can proceed based on the targeted code evidence and passing `aidens-runner` validation. Proceed with `P30-ABSORB-0029` recorded as explicit remaining P1 debt.
