# P30-04 Report

## Scope

Phase slice: execution evidence defaults, durable failure receipts, retry/provider/tool evidence.

Issue IDs addressed from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv` by observed current implementation and tests:

- `P30-ABSORB-0011`: `PlanActVerifyLoopV1` now defaults canonical receipt logging to a durable default config.
- `P30-ABSORB-0018`: runner default receipt level is `ReportLevelV1::Full`, not `Minimal`.
- `P30-ABSORB-0020`: public runner constructors open canonical receipts by default; failure paths reached through public constructors produce durable failure control records.

Issue IDs quarantined as remaining P1/P2 debt:

- `P30-ABSORB-0112`: provider retry attempts still record shallow warning strings rather than a full retry-attempt family receipt.
- `P30-ABSORB-0146`: `safe_coding_registry_for_current_dir` still has a declaration-only fallback if dispatcher registry construction fails.
- `P30-ABSORB-0147`: read-only tool descriptors still declare ephemeral canonical runtime receipt persistence.
- `P30-ABSORB-0298`: provider HTTP error-body read failure evidence remains outside this runner-only slice.
- `P30-ABSORB-0522` through `P30-ABSORB-0551`: `println!` observability sweep spans app/CLI/primitives/forge-pilot and is deferred as broad P2 observability debt.

## Changed Files

No additional P30-04 code changes were made in this slice. The phase result is based on observed current implementation plus existing targeted tests.

Observed implementation evidence:

- `crates/aidens-runner/src/lib.rs:90`: `PlanActVerifyLoopV1::new` sets `canonical_receipt_log_config: Some(default_plan_act_verify_receipt_log_config())`.
- `crates/aidens-runner/src/lib.rs:93`: `PlanActVerifyLoopV1::new` sets `receipt_level: ReportLevelV1::Full`.
- `crates/aidens-runner/src/lib.rs:1776`: `AiDENsRunnerBuilder::default` sets `receipt_level: ReportLevelV1::Full`.
- `crates/aidens-runner/src/lib.rs:1877`: `AiDENsRunnerBuilder::build` installs `default_runner_receipt_log_config` when no explicit config is provided.
- `crates/aidens-runner/src/lib.rs:1571`: `failure_control_records` emits verification-control records when canonical receipts are present.

## Tests Added Or Updated

No new tests were added in this slice.

Existing relevant tests:

- `default_runner_failure_is_report_rich_with_durable_store`
- `canonical_log_records_provider_unavailable_report`
- `canonical_log_records_tool_and_boundary_failure_report`
- `canonical_log_reopens_runner_report_after_process_boundary`

## Commands Run

- `cargo check --manifest-path Cargo.toml -p aidens-contracts -p aidens-cli --all-targets --locked`
  - Result: pass. This was run after P30-03 edits and before this report; no P30-04 code was changed.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo .`
  - Result: exit 0, `findings=1838 hard=0`.

Previously in this session, relevant runner validation passed:

- `cargo test --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass, 38 unit tests and 8 integration tests passed for `aidens-runner`.

## Unresolved Risks And Quarantines

- Retry evidence remains too thin: retries are warning strings, not first-class attempt-family receipts.
- Read-only tool runtime receipt persistence remains ephemeral in canonical descriptors.
- Declaration-only safe registry fallback remains possible in convenience constructors.
- Broad `println!` observability findings are not addressed in this phase.
- The `failure_control_records` `None` branch still exists internally; current public constructors make canonical receipts present, but the branch is not removed.

## Invariant Revalidation Checklist

- Default runner receipt level is full.
- Public runner construction opens a canonical receipt log by default.
- PlanActVerifyLoop construction installs a durable receipt log by default.
- Provider-unavailable and parser-fallback failure paths have existing durable-control-record tests.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-04 can proceed for the P0/default durable evidence blockers already present in the code. Remaining P1/P2 retry/tool/observability items are explicit debt and must limit final release claims.
