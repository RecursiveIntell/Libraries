# AiDENs Supported-Lane Completion & Hardening Plan — 2026-07-13

## Completion contract

This pass closes every confirmed local supported-lane P0/P1 defect from the current hostile audit and produces a locally verified commit. Completion means the settled Rust workspace and local release gates are green; current documents distinguish verified local behavior, actually exercised live-provider behavior, and deferred/disabled horizons.

It does **not** imply federation, settlement, attestation, remote-oracle, production fault tolerance, or any provider behavior that lacks a fresh live receipt.

## Pre-flight gate

Before any new implementation edit:

1. Freeze/wait for editors and record parent-repository HEAD plus `AiDENs/` dirty scope.
2. Do not stage or edit sibling-repository paths.
3. Verify the environment-mutating tool test lock with Cargo.
4. Treat prior agent summaries and earlier green runs as leads only; re-run final gates on the settled tree.

Failure behavior: stop the affected batch, retain the dirty tree, repair the specific precondition, and retry. No commit may occur while a relevant editor is active.

## Issue matrix

| ID | Severity | Required invariant | Regression / proof | Focused gate |
|---|---:|---|---|---|
| C-001 | P0 | Approval ID is deterministically bound to tool, risk, sandbox, run, and attempt; substitute contexts are rejected. | Approve with request ID for context A and fields for context B must fail. | `cargo test -p aidens-cli --all-targets --locked` |
| C-002 | P1 | Exposure planning never authorizes a side-effect tool without exact run/attempt context; dispatch is the authorization boundary. | Side-effect exposure is approval-blocked pre-run; host-issued exact-context dispatch succeeds. | `cargo test -p aidens-tool-kit --all-targets --locked` |
| C-003 | P0 | Canonical append validates full chain under lock before deriving next link; malformed or digest/sequence tampered history blocks append. | Append after syntactically valid tampering fails. | `cargo test -p aidens-receipts --all-targets --locked` |
| C-004 | P0 | Durable queue completion succeeds before in-memory completion/active-job clearing. | Inject queue-completion failure and assert state remains non-complete. | `cargo test -p aidens-autonomous --all-targets --locked` |
| C-005 | P1 | Rollback preserves original existence: restore existing files, remove newly created files, disclose residual uncertainty. | Forced failure after new-file write leaves target absent and receipt says `removed_new`. | `cargo test -p aidens-tool-kit --all-targets --locked` |
| C-006 | P1 | Tool execution success is not causal verification success; post-dispatch governance is advisory/non-promotable unless verification ran. | Successful invocation emits `AdvisoryOnly`, not `Succeeded` causal refutation. | `cargo test -p aidens-runner --all-targets --locked` |
| C-007 | P1 | Provider implementation, route, readiness, capability, and doctor truth agree. | Unavailable boundary is not executable; mock HTTP/live boundary is truthfully classified. | `cargo test -p aidens-provider-kit -p aidens-cli --all-targets --locked` |
| C-008 | P1 | Cycle receipts capture the actual cycle mode/errors before transitions and per-cycle—not cumulative—metrics. | Failed subtractive cycle remains subtractive/degraded; nonzero delta gaps are recorded. | `cargo test -p aidens-autonomous --all-targets --locked` |
| C-009 | P0 | Cycle receipts used as an audit surface are retained and explicitly classified as durable or non-durable. | Receipt history is inspectable and persistence state is accurately disclosed. | `cargo test -p aidens-autonomous --all-targets --locked` |
| C-010 | P1 | Provider native-tool parsing preserves malformed-call evidence rather than dropping it. | Mixed valid/malformed arrays fail with indexed error. | `cargo test -p aidens-provider-kit --all-targets --locked` |
| C-011 | P1 | Corrupt configured receipt storage is degraded/unavailable, never healthy. | Corrupt log doctor fixture. | `cargo test -p aidens-cli --all-targets --locked` |
| C-012 | P2 | UI/example presentation never byte-slices UTF-8. | Emoji boundary tests and Clippy. | `cargo test -p aidens-tui --all-targets --locked` |
| C-013 | P1 | Tests mutating global environment are serialized without holding a blocking guard across await. | Parallel test execution and Clippy. | `cargo test -p aidens-tool-kit --test p30_tool_hardening --locked` |
| C-014 | P1 | Live provider claims are based on bounded, redacted Ollama/OpenAI-compatible evidence—not configuration inference. | Discovery + chat + tool-call probes where supported. | ignored/live-only probes, stored under ignored `.codex_evidence/live-provider/` |
| C-015 | P1 | README/status/current-run/audit are consistent with actual gates and external limitations. | Current truth gate plus manual diff review. | `bash scripts/verify_current.sh` |
| C-016 | P1 | No local release claim without a settled full bar and post-fix hostile review. | Fresh gate command receipts + read-only Codex review. | final bar below |

## Execution order

1. **Pre-flight:** stabilize the working tree, verify test isolation, and reconcile C-001–C-013 against current source.
2. **Local correctness revision:** repair any confirmed remaining C-001–C-013 defects serially with RED/GREEN tests. Never weaken permit, receipt, or truth invariants to satisfy a test.
3. **Live-provider certification:** run bounded Ollama discovery/chat and native tool calls only if supported; inspect OpenAI-compatible endpoint/credentials and certify it only if an authenticated endpoint is actually configured.
4. **Release truth revision:** update current audit, run ledger, status, support profile, and this plan from fresh receipts only.
5. **Final gate:** run the complete bar, then a read-only high-effort Codex hostile review on the settled tree. Controller verifies each new finding; confirmed P0/P1 findings loop back to step 2.
6. **Commit:** stage only paths under `AiDENs/` from `/home/sikmindz/Coding/Libraries`; review staged diff; commit locally; verify SHA and clean AiDENs scope. Never push without explicit instruction.

## Final release bar

```bash
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked --all-targets --no-fail-fast
bash scripts/assert_no_compatibility_ledgers.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_tool_runtime_delegation.sh
bash scripts/phase_verify_contract_ownership.sh final
python3 scripts/assert_current_run_truth.py
bash scripts/verify_current.sh
python3 scripts/p30_guard.py
git diff --check
```

## Claim boundary

The final documentation must state separately:

- locally certified supported-lane behavior;
- the exact live provider models/endpoints and behaviors exercised;
- provider behavior not tested (including unsupported native tool loops);
- deferred/disabled federation, remote-oracle, settlement, attestation, and production-network horizons.
