# P21 Phase 02 Report — Test-Agent CLI

Status: PASS

Run timestamp: 2026-04-30T23:00:18-05:00

## Phase scope

Phase 02 focused only on the operator-facing `aidens run-test-agent` command.

Touched files/crates:

- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/Cargo.toml`
- `handoffs/p21/PHASE_02_REPORT.md`
- `target/p21/phase02/` logs
- `target/p21/test-agent/aidens-basic-test-agent/` generated proof bundle

No runner, provider, tool, agency, memory, kernel, verification, repair, fixture, eval, or scanner files were changed.

## Invariant revalidation

Before Phase 02 implementation, these checks passed:

- `bash scripts/assert_stack_paths.sh .`
  - Log: `target/p21/phase02/invariant_stack_paths.log`
  - Result: PASS
- `bash scripts/assert_no_local_substitute_dependencies.sh`
  - Log: `target/p21/phase02/invariant_no_local_substitute_dependencies.log`
  - Output: `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .`
  - Log: `target/p21/phase02/invariant_compat_is_finite.log`
  - Result: PASS
- `bash scripts/assert_no_shadow_truth.sh .`
  - Log: `target/p21/phase02/invariant_no_shadow_truth.log`
  - Result: PASS
- `bash scripts/p21_verify.sh`
  - Log: `target/p21/phase02/invariant_p21_verify.log`
  - Output ended with `P21 verify completed`

After implementation, `bash scripts/p21_verify.sh` passed again.

Log: `target/p21/phase02/p21_verify_after_change.log`

## Implementation summary

Added `aidens run-test-agent <config> [--prompt <prompt>] [--out <dir>]`.

The command:

- parses the test-agent TOML fixture;
- resolves fixture paths relative to the AiDENs repository root inferred from the config path;
- builds an effective AiDENs app config with mock provider fixture text, sandbox root, receipt root, enabled tool bundles, and full receipt level;
- runs through `AiDENsApp::from_config(...).build().await?.run_once(...)`, which uses the real `AiDENsRunner` path;
- fails if `agency.enabled = true` but agency policy reports or canonical agency receipt records are missing;
- writes the required bundle files:
  - `final.txt`
  - `run-report.json`
  - `turn-report.json`
  - `tool-exposure.json`
  - `agency-policy-reports.json`
  - `event-log.ndjson`
  - `summary.md`

The CLI manifest now directly depends on `aidens-runner` for the output-bundle helper type and `toml` for parsing test-agent TOML.

## Required proof

### `cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml`

Log: `target/p21/phase02/run_test_agent_command.log`

Result: PASS

Output bundle:

```text
target/p21/test-agent/aidens-basic-test-agent/final.txt
target/p21/test-agent/aidens-basic-test-agent/run-report.json
target/p21/test-agent/aidens-basic-test-agent/turn-report.json
target/p21/test-agent/aidens-basic-test-agent/tool-exposure.json
target/p21/test-agent/aidens-basic-test-agent/agency-policy-reports.json
target/p21/test-agent/aidens-basic-test-agent/event-log.ndjson
target/p21/test-agent/aidens-basic-test-agent/summary.md
target/p21/test-agent/aidens-basic-test-agent/receipts/canonical-receipts.ndjson
```

Inspection log: `target/p21/phase02/output_bundle_inspection.log`

Key inspection output:

```text
agency_reports=2
provider=mock route=mock native_tool_loop=false tool_invocations=1
event_log_events=13
canonical_receipt_records=5
```

The generated event log includes:

- `provider_route_selected`
- `tool_exposure_plan_created`
- `permit_checked`
- `tool_invocation_recorded`
- `agency_policy_evaluated`
- `final_response_recorded`

### `cargo test -p aidens-integration-tests test_agent_vertical_slice -- --nocapture`

Log: `target/p21/phase02/cargo_test_integration_test_agent_vertical_slice.log`

Result: PASS. The existing integration vertical slice still passes.

## Additional validation

- `cargo test -p aidens-cli run_test_agent_writes_bundle_and_receipts_through_runner -- --nocapture`
  - Log: `target/p21/phase02/cargo_test_aidens_cli_run_test_agent.log`
  - Result: PASS
- `cargo fmt --all --check`
  - Log: `target/p21/phase02/cargo_fmt_check.log`
  - Result: PASS
- `cargo check -p aidens-cli --all-targets --all-features`
  - Log: `target/p21/phase02/cargo_check_aidens_cli.log`
  - Result: PASS
- `cargo test -p aidens-cli --all-targets --all-features`
  - Log: `target/p21/phase02/cargo_test_aidens_cli_all_targets.log`
  - Result: PASS
- `cargo clippy -p aidens-cli --all-targets --all-features -- -D warnings`
  - Log: `target/p21/phase02/cargo_clippy_aidens_cli.log`
  - Result: PASS
- `cargo check --workspace --all-targets --all-features`
  - Log: `target/p21/phase02/cargo_check_workspace_all_targets_all_features.log`
  - Result: PASS

Development notes:

- `target/p21/phase02/cargo_check_aidens_cli_initial.log` records the expected first compile failure after adding TOML parsing without the direct `toml` dependency.
- `target/p21/phase02/cargo_fmt_check_pre.log` records pre-format diffs; `cargo fmt --all` was run, and the final fmt check passed.

## Invariant checks performed

- No shadow truth: existing scanner passed; implementation only translates test-agent fixture data into an effective AiDENs app config and output bundle.
- No local canonical ownership: command delegates execution to `AiDENsApp`/`AiDENsRunner` and does not define memory/evidence/kernel/repair/verification/federation/mechanism semantics.
- Canonical IDs: `stack-ids` path scanner passed and no stack-id dependency was changed.
- No silent parser fallback: parser-fallback tool calls still run through the existing runner and boundary repair/degradation receipt path.
- Execution is evidence: output bundle includes run report, turn report, tool exposure, logical event log, agency policy reports, and canonical receipt log.
- Agency runtime gate: command fails if `agency.enabled = true` and agency reports or canonical agency records are absent.
- Provider/tool truth: mock route is executable; output proves `native_tool_loop=false`; unsupported provider behavior was not changed.
- Tests/fixtures/evals/scanners were not deleted or weakened.

## Repairs

- Added the CLI command and focused unit coverage.
- Added direct CLI dependencies on `aidens-runner` and `toml`.
- Fixed relative fixture resolution so test-agent fixture paths are resolved against the AiDENs repository root inferred from the config path, not the process working directory.
- Ran `cargo fmt --all` after pre-format check reported formatting diffs.

## Stop condition

Per P21 phase protocol, Codex must stop here and wait for the operator to paste the next global plus Phase 03-specific injection before touching code or proceeding to generated-agent project proof.
